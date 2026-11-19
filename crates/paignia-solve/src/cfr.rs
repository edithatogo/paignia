//! Vanilla CFR and CFR+ on an admitted perfect-recall information-set graph.
//!
//! Regret updates use the current information-set strategy and the counterfactual
//! reach of every other player and of chance. The returned strategy is the
//! reach-weighted average of the strategies that were actually played. This is
//! not linear averaging, and it does not by itself prove a Nash equilibrium.
//! Kuhn's closed form is an external check. Leduc is not solved or claimed here.

use std::collections::{BTreeMap, BTreeSet};

use thiserror::Error;

use paignia_core::{
    build_information_set_graph, ActionId, GameSpecification, InformationSetDeclaration,
    InformationSetError, InformationSetGraph, InformationSetId, Node, NodeId, PlayerId,
};

/// Version of the regret-matching, averaging and admission rules in this module.
pub const VERSION: &str = "1.0.0";
/// Inclusive iteration budget. Callers choose a smaller training length.
pub const MAX_ITERATIONS: u32 = 100_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Variant {
    /// Both players' regrets are updated from one frozen strategy snapshot.
    Cfr,
    /// Players update in `PlayerId` order. Each update is visible to the next
    /// player, and cumulative regret is floored at zero.
    CfrPlus,
}

impl Variant {
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Cfr => "paignia/vanilla-cfr",
            Self::CfrPlus => "paignia/cfr-plus",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Config {
    pub variant: Variant,
    pub iterations: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Strategy {
    /// Declared legal-action order for each admitted information set.
    pub action_order: BTreeMap<InformationSetId, Vec<ActionId>>,
    /// Probabilities aligned with [`Self::action_order`]. Each vector sums to one.
    pub probabilities: BTreeMap<InformationSetId, Vec<f64>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Training {
    pub variant: Variant,
    pub iterations: u32,
    pub average_strategy: Strategy,
    /// Expected payoff of the average strategy for every player who owns a
    /// decision or appears in a terminal payoff.
    pub root_values: BTreeMap<PlayerId, f64>,
}

#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error(transparent)]
    InformationSet(#[from] InformationSetError),
    #[error("CFR iteration count {iterations} is outside 1..={MAX_ITERATIONS}")]
    IterationBudget { iterations: u32 },
    #[error("decision node {node:?} has action {action:?} outside its admitted information set")]
    InconsistentActionOrder { node: NodeId, action: ActionId },
    #[error("information set {0:?} is reachable from itself under perfect-recall traversal")]
    RecallCycle(InformationSetId),
    #[error("best response reached information set {0:?} before its action was selected")]
    UnsolvedInformationSet(InformationSetId),
}

/// Train an average strategy on the unique graph admitted by `declaration`.
///
/// Imperfect recall, partial partitions and action mismatches fail in
/// information-set admission before any regret is stored.
///
/// # Errors
///
/// Returns an information-set admission error, or [`Error::IterationBudget`]
/// when the requested length is zero or above [`MAX_ITERATIONS`].
pub fn train(
    source: &GameSpecification,
    declaration: &InformationSetDeclaration,
    config: &Config,
) -> Result<Training, Error> {
    if config.iterations == 0 || config.iterations > MAX_ITERATIONS {
        return Err(Error::IterationBudget {
            iterations: config.iterations,
        });
    }
    let graph = build_information_set_graph(source, declaration)?;
    let players = players_of(&graph);
    let mut regrets = zero_tables(&graph);
    let mut strategy_sum = zero_tables(&graph);

    for _ in 0..config.iterations {
        match config.variant {
            Variant::Cfr => {
                let snapshot = regret_matching(&graph, &regrets);
                let mut delta = zero_tables(&graph);
                for player in &players {
                    Traversal {
                        source,
                        graph: &graph,
                        strategy: &snapshot,
                        updating: player,
                        delta: &mut delta,
                        strategy_sum: &mut strategy_sum,
                    }
                    .walk()?;
                }
                add_regret(&mut regrets, &delta, false);
            }
            Variant::CfrPlus => {
                for player in &players {
                    let snapshot = regret_matching(&graph, &regrets);
                    let mut delta = zero_tables(&graph);
                    Traversal {
                        source,
                        graph: &graph,
                        strategy: &snapshot,
                        updating: player,
                        delta: &mut delta,
                        strategy_sum: &mut strategy_sum,
                    }
                    .walk()?;
                    add_regret(&mut regrets, &delta, true);
                }
            }
        }
    }

    let average_strategy = average_strategy(&graph, &strategy_sum);
    let root_values = profile_values(source, &graph, &average_strategy)?;
    Ok(Training {
        variant: config.variant,
        iterations: config.iterations,
        average_strategy,
        root_values,
    })
}

/// Expected payoffs of `strategy` for every participating player.
///
/// # Errors
///
/// Returns [`Error::InconsistentActionOrder`] when a node edge does not follow
/// the strategy's admitted action order.
pub fn profile_values(
    source: &GameSpecification,
    graph: &InformationSetGraph,
    strategy: &Strategy,
) -> Result<BTreeMap<PlayerId, f64>, Error> {
    let players = participating_players(source, graph);
    let mut memo = BTreeMap::new();
    values_at(
        source,
        graph,
        strategy,
        &players,
        &source.game.root,
        &mut memo,
    )
}

/// Expected payoff for `player` when that player unilaterally switches to a
/// pure best response and everyone else keeps `strategy`.
///
/// # Errors
///
/// Returns [`Error::RecallCycle`] when an information set can reach itself, or
/// [`Error::UnsolvedInformationSet`] when a downstream action is missing.
pub fn best_response_value(
    source: &GameSpecification,
    graph: &InformationSetGraph,
    strategy: &Strategy,
    player: &PlayerId,
) -> Result<f64, Error> {
    let reaches = counterfactual_reaches(source, graph, strategy, player);
    let order = best_response_order(source, graph, player)?;
    let mut policy = BTreeMap::new();
    let mut memo = BTreeMap::new();
    for set_id in order {
        let set = &graph.sets[&set_id];
        let mut best_index = 0_usize;
        let mut best_value = f64::NEG_INFINITY;
        for action_index in 0..set.legal_actions.len() {
            let mut value = 0.0;
            for member in &set.members {
                let target = action_target(source, graph, member, action_index)?;
                value += reaches.get(member).copied().unwrap_or(0.0)
                    * response_value(source, graph, strategy, &policy, player, &target, &mut memo)?;
            }
            if value > best_value {
                best_value = value;
                best_index = action_index;
            }
        }
        policy.insert(set_id, best_index);
    }
    response_value(
        source,
        graph,
        strategy,
        &policy,
        player,
        &source.game.root,
        &mut memo,
    )
}

/// Sum of unilateral pure best-response gains against `strategy`.
///
/// For a two-player zero-sum game this is the usual exploitability. It is not
/// a certificate that the strategy is a Nash equilibrium.
///
/// # Errors
///
/// Propagates the errors of [`profile_values`] and [`best_response_value`].
pub fn nash_conv(
    source: &GameSpecification,
    graph: &InformationSetGraph,
    strategy: &Strategy,
) -> Result<f64, Error> {
    let values = profile_values(source, graph, strategy)?;
    let mut total = 0.0;
    for player in players_of(graph) {
        let current = values.get(&player).copied().unwrap_or(0.0);
        total += best_response_value(source, graph, strategy, &player)? - current;
    }
    Ok(total)
}

fn players_of(graph: &InformationSetGraph) -> Vec<PlayerId> {
    graph
        .sets
        .values()
        .map(|set| set.player.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn participating_players(source: &GameSpecification, graph: &InformationSetGraph) -> Vec<PlayerId> {
    let mut players = players_of(graph).into_iter().collect::<BTreeSet<_>>();
    for node in source.game.nodes.values() {
        if let Node::Terminal { payoffs } = node {
            players.extend(payoffs.keys().cloned());
        }
    }
    players.into_iter().collect()
}

fn zero_tables(graph: &InformationSetGraph) -> BTreeMap<InformationSetId, Vec<f64>> {
    graph
        .sets
        .iter()
        .map(|(id, set)| (id.clone(), vec![0.0; set.legal_actions.len()]))
        .collect()
}

fn regret_matching(
    graph: &InformationSetGraph,
    regrets: &BTreeMap<InformationSetId, Vec<f64>>,
) -> BTreeMap<InformationSetId, Vec<f64>> {
    graph
        .sets
        .iter()
        .map(|(id, set)| {
            let regret = &regrets[id];
            let positive = regret
                .iter()
                .map(|value| value.max(0.0))
                .collect::<Vec<_>>();
            let total: f64 = positive.iter().sum();
            let probabilities = if total > 0.0 {
                positive.into_iter().map(|value| value / total).collect()
            } else {
                uniform(set.legal_actions.len())
            };
            (id.clone(), probabilities)
        })
        .collect()
}

fn uniform(len: usize) -> Vec<f64> {
    let width = u32::try_from(len).expect("an information set has a finite action list");
    vec![1.0 / f64::from(width); len]
}

fn add_regret(
    regrets: &mut BTreeMap<InformationSetId, Vec<f64>>,
    delta: &BTreeMap<InformationSetId, Vec<f64>>,
    floor_at_zero: bool,
) {
    for (id, change) in delta {
        let slot = regrets.get_mut(id).expect("regret table covers every set");
        for (index, value) in change.iter().enumerate() {
            slot[index] += value;
            if floor_at_zero {
                slot[index] = slot[index].max(0.0);
            }
        }
    }
}

fn average_strategy(
    graph: &InformationSetGraph,
    strategy_sum: &BTreeMap<InformationSetId, Vec<f64>>,
) -> Strategy {
    let mut action_order = BTreeMap::new();
    let mut probabilities = BTreeMap::new();
    for (id, set) in &graph.sets {
        let weights = &strategy_sum[id];
        let total: f64 = weights.iter().sum();
        let probs = if total > 0.0 {
            weights.iter().map(|value| value / total).collect()
        } else {
            uniform(set.legal_actions.len())
        };
        action_order.insert(id.clone(), set.legal_actions.clone());
        probabilities.insert(id.clone(), probs);
    }
    Strategy {
        action_order,
        probabilities,
    }
}

struct Traversal<'a> {
    source: &'a GameSpecification,
    graph: &'a InformationSetGraph,
    strategy: &'a BTreeMap<InformationSetId, Vec<f64>>,
    updating: &'a PlayerId,
    delta: &'a mut BTreeMap<InformationSetId, Vec<f64>>,
    strategy_sum: &'a mut BTreeMap<InformationSetId, Vec<f64>>,
}

impl Traversal<'_> {
    fn walk(&mut self) -> Result<(), Error> {
        let root = self.source.game.root.clone();
        self.visit(&root, 1.0, 1.0)?;
        Ok(())
    }

    fn visit(
        &mut self,
        node_id: &NodeId,
        player_reach: f64,
        counterfactual: f64,
    ) -> Result<f64, Error> {
        match &self.source.game.nodes[node_id] {
            Node::Terminal { payoffs } => Ok(payoffs.get(self.updating).copied().unwrap_or(0.0)),
            Node::Chance { edges } => {
                let steps = edges
                    .iter()
                    .map(|edge| (edge.probability, edge.target.clone()))
                    .collect::<Vec<_>>();
                let mut value = 0.0;
                for (probability, target) in steps {
                    value += probability
                        * self.visit(&target, player_reach, counterfactual * probability)?;
                }
                Ok(value)
            }
            Node::Decision { player, edges } => {
                let updating_player = player == self.updating;
                let player_for_update = player.clone();
                let set_id = self.graph.assignments[node_id].clone();
                let legal_actions = self.graph.sets[&set_id].legal_actions.clone();
                let probabilities = self.strategy[&set_id].clone();
                let steps = edges
                    .iter()
                    .map(|edge| (edge.action.clone(), edge.target.clone()))
                    .collect::<Vec<_>>();
                let mut action_values = Vec::with_capacity(steps.len());
                let mut value = 0.0;
                for (index, (action, target)) in steps.into_iter().enumerate() {
                    if legal_actions.get(index) != Some(&action) {
                        return Err(Error::InconsistentActionOrder {
                            node: node_id.clone(),
                            action,
                        });
                    }
                    let probability = probabilities[index];
                    let next_player_reach = if updating_player {
                        player_reach * probability
                    } else {
                        player_reach
                    };
                    let next_counterfactual = if updating_player {
                        counterfactual
                    } else {
                        counterfactual * probability
                    };
                    let action_value =
                        self.visit(&target, next_player_reach, next_counterfactual)?;
                    value += probability * action_value;
                    action_values.push(action_value);
                }
                if player_for_update == *self.updating {
                    let regret = self.delta.get_mut(&set_id).expect("delta covers every set");
                    for (index, action_value) in action_values.iter().enumerate() {
                        regret[index] += counterfactual * (action_value - value);
                    }
                    let sum = self
                        .strategy_sum
                        .get_mut(&set_id)
                        .expect("strategy sum covers every set");
                    for (index, probability) in probabilities.iter().enumerate() {
                        sum[index] += player_reach * probability;
                    }
                }
                Ok(value)
            }
        }
    }
}

fn values_at(
    source: &GameSpecification,
    graph: &InformationSetGraph,
    strategy: &Strategy,
    players: &[PlayerId],
    node_id: &NodeId,
    memo: &mut BTreeMap<NodeId, BTreeMap<PlayerId, f64>>,
) -> Result<BTreeMap<PlayerId, f64>, Error> {
    if let Some(existing) = memo.get(node_id) {
        return Ok(existing.clone());
    }
    let value = match &source.game.nodes[node_id] {
        Node::Terminal { payoffs } => players
            .iter()
            .map(|player| (player.clone(), payoffs.get(player).copied().unwrap_or(0.0)))
            .collect(),
        Node::Chance { edges } => {
            let mut totals = players
                .iter()
                .map(|player| (player.clone(), 0.0))
                .collect::<BTreeMap<_, _>>();
            for edge in edges {
                let child = values_at(source, graph, strategy, players, &edge.target, memo)?;
                for player in players {
                    *totals.get_mut(player).expect("player totals exist") +=
                        edge.probability * child.get(player).copied().unwrap_or(0.0);
                }
            }
            totals
        }
        Node::Decision { edges, .. } => {
            let set_id = &graph.assignments[node_id];
            let probabilities = &strategy.probabilities[set_id];
            let mut totals = players
                .iter()
                .map(|player| (player.clone(), 0.0))
                .collect::<BTreeMap<_, _>>();
            for (index, edge) in edges.iter().enumerate() {
                if strategy.action_order[set_id].get(index) != Some(&edge.action) {
                    return Err(Error::InconsistentActionOrder {
                        node: node_id.clone(),
                        action: edge.action.clone(),
                    });
                }
                let child = values_at(source, graph, strategy, players, &edge.target, memo)?;
                for player in players {
                    *totals.get_mut(player).expect("player totals exist") +=
                        probabilities[index] * child.get(player).copied().unwrap_or(0.0);
                }
            }
            totals
        }
    };
    memo.insert(node_id.clone(), value.clone());
    Ok(value)
}

fn action_target(
    source: &GameSpecification,
    graph: &InformationSetGraph,
    node_id: &NodeId,
    action_index: usize,
) -> Result<NodeId, Error> {
    match &source.game.nodes[node_id] {
        Node::Decision { edges, .. } => {
            let edge = &edges[action_index];
            let set_id = &graph.assignments[node_id];
            if graph.sets[set_id].legal_actions.get(action_index) != Some(&edge.action) {
                return Err(Error::InconsistentActionOrder {
                    node: node_id.clone(),
                    action: edge.action.clone(),
                });
            }
            Ok(edge.target.clone())
        }
        Node::Chance { .. } | Node::Terminal { .. } => {
            unreachable!("best response only targets admitted decision nodes")
        }
    }
}

fn counterfactual_reaches(
    source: &GameSpecification,
    graph: &InformationSetGraph,
    strategy: &Strategy,
    player: &PlayerId,
) -> BTreeMap<NodeId, f64> {
    let mut reach = BTreeMap::new();
    reach.insert(source.game.root.clone(), 1.0);
    for node_id in topological_order(source) {
        let weight = reach.get(&node_id).copied().unwrap_or(0.0);
        match &source.game.nodes[&node_id] {
            Node::Terminal { .. } => {}
            Node::Chance { edges } => {
                for edge in edges {
                    *reach.entry(edge.target.clone()).or_insert(0.0) += weight * edge.probability;
                }
            }
            Node::Decision {
                player: owner,
                edges,
            } if owner == player => {
                for edge in edges {
                    *reach.entry(edge.target.clone()).or_insert(0.0) += weight;
                }
            }
            Node::Decision { edges, .. } => {
                let set_id = &graph.assignments[&node_id];
                for (index, edge) in edges.iter().enumerate() {
                    let probability = strategy.probabilities[set_id][index];
                    *reach.entry(edge.target.clone()).or_insert(0.0) += weight * probability;
                }
            }
        }
    }
    reach
}

fn best_response_order(
    source: &GameSpecification,
    graph: &InformationSetGraph,
    player: &PlayerId,
) -> Result<Vec<InformationSetId>, Error> {
    let owned = graph
        .sets
        .iter()
        .filter(|(_, set)| set.player == *player)
        .map(|(id, _)| id.clone())
        .collect::<Vec<_>>();
    let mut dependencies = BTreeMap::new();
    for set_id in &owned {
        dependencies.insert(
            set_id.clone(),
            reachable_infosets(source, graph, player, set_id)?,
        );
    }
    let mut remaining = dependencies;
    let mut ready = remaining
        .iter()
        .filter(|(_, deps)| deps.is_empty())
        .map(|(id, _)| id.clone())
        .collect::<BTreeSet<_>>();
    let mut ordered = Vec::new();
    let mut done = BTreeSet::new();
    while let Some(set_id) = ready.pop_first() {
        if !done.insert(set_id.clone()) {
            continue;
        }
        ordered.push(set_id.clone());
        for (other, deps) in &mut remaining {
            if done.contains(other) {
                continue;
            }
            deps.remove(&set_id);
            if deps.is_empty() {
                ready.insert(other.clone());
            }
        }
    }
    if ordered.len() != owned.len() {
        let unresolved = owned
            .into_iter()
            .find(|id| !done.contains(id))
            .expect("a missing set");
        return Err(Error::RecallCycle(unresolved));
    }
    Ok(ordered)
}

fn reachable_infosets(
    source: &GameSpecification,
    graph: &InformationSetGraph,
    player: &PlayerId,
    origin: &InformationSetId,
) -> Result<BTreeSet<InformationSetId>, Error> {
    let mut stack = Vec::new();
    for member in &graph.sets[origin].members {
        match &source.game.nodes[member] {
            Node::Decision { edges, .. } => {
                for edge in edges {
                    stack.push(edge.target.clone());
                }
            }
            Node::Chance { .. } | Node::Terminal { .. } => {}
        }
    }
    let mut seen = BTreeSet::new();
    let mut found = BTreeSet::new();
    while let Some(node_id) = stack.pop() {
        if !seen.insert(node_id.clone()) {
            continue;
        }
        match &source.game.nodes[&node_id] {
            Node::Terminal { .. } => {}
            Node::Chance { edges } => {
                for edge in edges {
                    stack.push(edge.target.clone());
                }
            }
            Node::Decision {
                player: owner,
                edges,
            } => {
                if owner == player {
                    let set_id = &graph.assignments[&node_id];
                    if set_id == origin {
                        return Err(Error::RecallCycle(origin.clone()));
                    }
                    found.insert(set_id.clone());
                }
                for edge in edges {
                    stack.push(edge.target.clone());
                }
            }
        }
    }
    Ok(found)
}

fn response_value(
    source: &GameSpecification,
    graph: &InformationSetGraph,
    strategy: &Strategy,
    policy: &BTreeMap<InformationSetId, usize>,
    player: &PlayerId,
    node_id: &NodeId,
    memo: &mut BTreeMap<NodeId, f64>,
) -> Result<f64, Error> {
    if let Some(value) = memo.get(node_id) {
        return Ok(*value);
    }
    let value = match &source.game.nodes[node_id] {
        Node::Terminal { payoffs } => payoffs.get(player).copied().unwrap_or(0.0),
        Node::Chance { edges } => {
            let mut total = 0.0;
            for edge in edges {
                total += edge.probability
                    * response_value(source, graph, strategy, policy, player, &edge.target, memo)?;
            }
            total
        }
        Node::Decision {
            player: owner,
            edges,
        } if owner == player => {
            let set_id = &graph.assignments[node_id];
            let Some(index) = policy.get(set_id).copied() else {
                return Err(Error::UnsolvedInformationSet(set_id.clone()));
            };
            response_value(
                source,
                graph,
                strategy,
                policy,
                player,
                &edges[index].target,
                memo,
            )?
        }
        Node::Decision { edges, .. } => {
            let set_id = &graph.assignments[node_id];
            let probabilities = &strategy.probabilities[set_id];
            let mut total = 0.0;
            for (index, edge) in edges.iter().enumerate() {
                total += probabilities[index]
                    * response_value(source, graph, strategy, policy, player, &edge.target, memo)?;
            }
            total
        }
    };
    memo.insert(node_id.clone(), value);
    Ok(value)
}

fn topological_order(source: &GameSpecification) -> Vec<NodeId> {
    let mut indegree = source
        .game
        .nodes
        .keys()
        .map(|node| (node.clone(), 0_usize))
        .collect::<BTreeMap<_, _>>();
    for node in source.game.nodes.values() {
        for target in targets(node) {
            *indegree.get_mut(&target).expect("validated targets exist") += 1;
        }
    }
    let mut ready = indegree
        .iter()
        .filter(|(_, degree)| **degree == 0)
        .map(|(node, _)| node.clone())
        .collect::<BTreeSet<_>>();
    let mut order = Vec::with_capacity(source.game.nodes.len());
    while let Some(node_id) = ready.pop_first() {
        order.push(node_id.clone());
        for target in targets(&source.game.nodes[&node_id]) {
            let degree = indegree.get_mut(&target).expect("validated targets exist");
            *degree -= 1;
            if *degree == 0 {
                ready.insert(target);
            }
        }
    }
    order
}

fn targets(node: &Node) -> Vec<NodeId> {
    match node {
        Node::Decision { edges, .. } => edges.iter().map(|edge| edge.target.clone()).collect(),
        Node::Chance { edges } => edges.iter().map(|edge| edge.target.clone()).collect(),
        Node::Terminal { .. } => Vec::new(),
    }
}
