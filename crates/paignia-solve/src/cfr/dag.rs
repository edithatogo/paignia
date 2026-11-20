//! Indexed traversals of an admitted DAG. Shared continuations are evaluated once.

use std::collections::{BTreeMap, VecDeque};

use paignia_core::{
    GameSpecification, InformationSetGraph, InformationSetId, Node, PayoffVector, PlayerId,
};

use super::{finite, Error, Variant};

/// Neumaier summation retains small contributions when large terms cancel.
#[derive(Default)]
struct Sum {
    running: f64,
    correction: f64,
}

impl Sum {
    fn add(&mut self, term: f64) -> Result<(), Error> {
        let next = finite(self.running + term)?;
        let residual = if self.running.abs() >= term.abs() {
            (self.running - next) + term
        } else {
            (term - next) + self.running
        };
        self.correction = finite(self.correction + residual)?;
        self.running = next;
        Ok(())
    }

    fn finish(self) -> Result<f64, Error> {
        finite(self.running + self.correction)
    }
}

fn product(left: f64, right: f64) -> Result<f64, Error> {
    let result = finite(left * right)?;
    if result == 0.0 && left != 0.0 && right != 0.0 {
        return Err(Error::Underflow);
    }
    Ok(result)
}

fn weighted_sum(terms: impl Iterator<Item = (f64, f64)>) -> Result<f64, Error> {
    let mut sum = Sum::default();
    for (weight, value) in terms {
        sum.add(product(weight, value)?)?;
    }
    sum.finish()
}

enum Kind<'a> {
    Terminal(&'a PayoffVector),
    Chance(Vec<f64>),
    Decision(usize),
}

struct Vertex<'a> {
    kind: Kind<'a>,
    children: Vec<usize>,
}

impl Vertex<'_> {
    fn probabilities<'s>(&'s self, strategy: &'s [Vec<f64>]) -> &'s [f64] {
        match &self.kind {
            Kind::Terminal(_) => &[],
            Kind::Chance(p) => p,
            Kind::Decision(set) => &strategy[*set],
        }
    }

    fn value(
        &self,
        values: &[f64],
        player: &PlayerId,
        strategy: &[Vec<f64>],
        scale: f64,
    ) -> Result<f64, Error> {
        if let Kind::Terminal(payoffs) = self.kind {
            return product(payoffs.get(player).copied().unwrap_or(0.0), scale.recip());
        }
        weighted_sum(
            self.children
                .iter()
                .zip(self.probabilities(strategy))
                .map(|(&target, &p)| (p, values[target])),
        )
    }
}

struct Set<'a> {
    id: &'a InformationSetId,
    owner: &'a PlayerId,
    members: Vec<usize>,
}

pub(super) struct IndexedGame<'a> {
    nodes: Vec<Vertex<'a>>,
    sets: Vec<Set<'a>>,
    order: Vec<usize>,
    payoff_scales: BTreeMap<&'a PlayerId, f64>,
    root: usize,
}

impl<'a> IndexedGame<'a> {
    pub(super) fn new(source: &'a GameSpecification, graph: &'a InformationSetGraph) -> Self {
        let indices: BTreeMap<_, _> = source
            .game
            .nodes
            .keys()
            .enumerate()
            .map(|(i, id)| (id, i))
            .collect();
        let set_indices: BTreeMap<_, _> = graph
            .sets
            .keys()
            .enumerate()
            .map(|(i, id)| (id, i))
            .collect();
        let sets = graph
            .sets
            .iter()
            .map(|(id, set)| Set {
                id,
                owner: &set.player,
                members: set.members.iter().map(|member| indices[member]).collect(),
            })
            .collect();
        let nodes: Vec<_> = source
            .game
            .nodes
            .iter()
            .map(|(id, node)| match node {
                Node::Terminal { payoffs } => Vertex {
                    kind: Kind::Terminal(payoffs),
                    children: Vec::new(),
                },
                Node::Chance { edges } => Vertex {
                    kind: Kind::Chance(edges.iter().map(|edge| edge.probability).collect()),
                    children: edges.iter().map(|edge| indices[&edge.target]).collect(),
                },
                Node::Decision { edges, .. } => Vertex {
                    kind: Kind::Decision(set_indices[&graph.assignments[id]]),
                    children: edges.iter().map(|edge| indices[&edge.target]).collect(),
                },
            })
            .collect();
        // Positive utility rescaling leaves each player's strategy and best
        // replies invariant. A power of two preserves the payoff significand.
        let mut payoff_scales: BTreeMap<_, f64> =
            graph.sets.values().map(|set| (&set.player, 1.0)).collect();
        for node in &nodes {
            if let Kind::Terminal(payoffs) = node.kind {
                for (player, payoff) in payoffs {
                    let power = f64::from_bits(payoff.abs().to_bits() & (0x7ff_u64 << 52));
                    let scale = payoff_scales.entry(player).or_insert(1.0);
                    *scale = scale.max(power);
                }
            }
        }
        let root = indices[&source.game.root];
        let mut incoming = vec![0_usize; nodes.len()];
        for vertex in &nodes {
            for &target in &vertex.children {
                incoming[target] += 1;
            }
        }
        let mut order = vec![root];
        let mut index = 0;
        while index < order.len() {
            for &target in &nodes[order[index]].children {
                incoming[target] -= 1;
                if incoming[target] == 0 {
                    order.push(target);
                }
            }
            index += 1;
        }
        debug_assert_eq!(order.len(), nodes.len());
        Self {
            nodes,
            sets,
            order,
            payoff_scales,
            root,
        }
    }

    // Excluding a player's action probabilities gives counterfactual reach.
    // Summation over incoming edges preserves shared-path multiplicity.
    fn reaches(&self, strategy: &[Vec<f64>], player: &PlayerId) -> Result<Vec<f64>, Error> {
        let mut reach = vec![0.0; self.nodes.len()];
        reach[self.root] = 1.0;
        for &index in &self.order {
            let node = &self.nodes[index];
            let exclude_action =
                matches!(node.kind, Kind::Decision(set) if self.sets[set].owner == player);
            for (&target, &p) in node.children.iter().zip(node.probabilities(strategy)) {
                let weight = if exclude_action {
                    reach[index]
                } else {
                    product(reach[index], p)?
                };
                reach[target] = finite(reach[target] + weight)?;
            }
        }
        Ok(reach)
    }

    pub(super) fn profile_values(&self, strategy: &[Vec<f64>]) -> Result<PayoffVector, Error> {
        let mut payoffs = PayoffVector::new();
        let mut values = vec![0.0; self.nodes.len()];
        for (&player, &scale) in &self.payoff_scales {
            for &i in self.order.iter().rev() {
                values[i] = self.nodes[i].value(&values, player, strategy, scale)?;
            }
            payoffs.insert(player.clone(), product(values[self.root], scale)?);
        }
        Ok(payoffs)
    }

    pub(super) fn update(
        &self,
        strategy: &[Vec<f64>],
        player: &PlayerId,
        regrets: &mut [Vec<f64>],
        sums: &mut [Vec<f64>],
        variant: Variant,
    ) -> Result<(), Error> {
        let reach = self.reaches(strategy, player)?;
        let scale = self.payoff_scales.get(player).copied().unwrap_or(1.0);
        let mut values = vec![0.0; self.nodes.len()];
        for &i in self.order.iter().rev() {
            values[i] = self.nodes[i].value(&values, player, strategy, scale)?;
        }
        // At every subsequent own decision, perfect recall makes the product
        // of own action probabilities identical on all incoming histories.
        let mut own = vec![0.0; self.nodes.len()];
        own[self.root] = 1.0;
        for &i in &self.order {
            let node = &self.nodes[i];
            let acting = matches!(node.kind, Kind::Decision(set) if self.sets[set].owner == player);
            for (&target, &p) in node.children.iter().zip(node.probabilities(strategy)) {
                own[target] = if acting { product(own[i], p)? } else { own[i] };
            }
        }
        for (s, set) in self
            .sets
            .iter()
            .enumerate()
            .filter(|(_, set)| set.owner == player)
        {
            let mut changes: Vec<_> = strategy[s].iter().map(|_| Sum::default()).collect();
            for &i in &set.members {
                if reach[i] == 0.0 {
                    continue;
                }
                let children = &self.nodes[i].children;
                let baseline = values[children[0]];
                let expected_difference = weighted_sum(
                    strategy[s]
                        .iter()
                        .zip(children)
                        .map(|(&p, &child)| (p, values[child] - baseline)),
                )?;
                for (change, &child) in changes.iter_mut().zip(children) {
                    let advantage = (values[child] - baseline) - expected_difference;
                    change.add(product(reach[i], advantage)?)?;
                }
            }
            for (a, change) in changes.into_iter().enumerate() {
                let next = finite(regrets[s][a] + change.finish()?)?;
                regrets[s][a] = if variant == Variant::CfrPlus {
                    next.max(0.0)
                } else {
                    next
                };
                sums[s][a] = finite(sums[s][a] + product(own[set.members[0]], strategy[s][a])?)?;
            }
        }
        Ok(())
    }

    pub(super) fn best_response(
        &self,
        strategy: &[Vec<f64>],
        player: &PlayerId,
    ) -> Result<f64, Error> {
        let reach = self.reaches(strategy, player)?;
        let scale = self.payoff_scales.get(player).copied().unwrap_or(1.0);
        // All members of an owned information set wait for all their children,
        // then choose one common action. Perfect recall orders successive own
        // information sets even if members occur at different tree depths.
        let group: Vec<_> = self
            .nodes
            .iter()
            .enumerate()
            .map(|(i, node)| match node.kind {
                Kind::Decision(set) if self.sets[set].owner == player => self.sets[set].members[0],
                _ => i,
            })
            .collect();
        let mut parents = vec![Vec::new(); self.nodes.len()];
        let mut pending = vec![0_usize; self.nodes.len()];
        for (i, node) in self.nodes.iter().enumerate() {
            for &target in &node.children {
                parents[group[target]].push(group[i]);
                pending[group[i]] += 1;
            }
        }
        let mut ready: VecDeque<_> = (0..self.nodes.len())
            .filter(|&i| group[i] == i && pending[i] == 0)
            .collect();
        let mut values = vec![0.0; self.nodes.len()];
        while let Some(i) = ready.pop_front() {
            let node = &self.nodes[i];
            match node.kind {
                Kind::Decision(s) if self.sets[s].owner == player => {
                    let set = &self.sets[s];
                    let mut selected = 0;
                    for candidate in 1..node.children.len() {
                        // Compare alternatives within each hidden state before
                        // summing. A large state-specific common payoff cancels.
                        let advantage = weighted_sum(set.members.iter().map(|&m| {
                            let actions = &self.nodes[m].children;
                            (
                                reach[m],
                                values[actions[candidate]] - values[actions[selected]],
                            )
                        }))?;
                        if advantage > 0.0 {
                            selected = candidate;
                        }
                    }
                    for &m in &set.members {
                        values[m] = values[self.nodes[m].children[selected]];
                    }
                }
                _ => values[i] = node.value(&values, player, strategy, scale)?,
            }
            for &parent in &parents[i] {
                pending[parent] -= 1;
                if pending[parent] == 0 {
                    ready.push_back(parent);
                }
            }
        }
        if let Some(set) = self
            .sets
            .iter()
            .find(|set| set.owner == player && pending[set.members[0]] != 0)
        {
            return Err(Error::RecallCycle(set.id.clone()));
        }
        product(values[self.root], scale)
    }
}
