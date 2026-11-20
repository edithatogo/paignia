//! Vanilla CFR and CFR+ on an admitted perfect-recall information-set graph.
//!
//! Regret updates use the current information-set strategy and the counterfactual
//! reach of every other player and of chance. The returned strategy is the
//! reach-weighted average of the strategies that were actually played. This is
//! not linear averaging, and it does not by itself prove a Nash equilibrium.
//! Kuhn's closed form is an external check. Leduc is compared with the published
//! two-player value of the `OpenSpiel` rule set; that comparison is not a proof
//! that every equilibrium has been found.

mod dag;

use std::collections::{BTreeMap, BTreeSet};

use dag::{IndexedGame, ResponseMeasure};

use thiserror::Error;

use paignia_core::{
    build_information_set_graph, ActionId, GameSpecification, InformationSetDeclaration,
    InformationSetDefinition, InformationSetError, InformationSetGraph, InformationSetId, NodeId,
    PlayerId,
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
    #[error("information-set graph does not match its source and declared partition")]
    GraphMismatch,
    #[error("strategy domains do not match the admitted information sets")]
    StrategyDomain,
    #[error("invalid action probabilities at information set {0:?}")]
    InvalidDistribution(InformationSetId),
    #[error("counterfactual calculation exceeded finite arithmetic")]
    NonFiniteArithmetic,
    #[error("a nonzero counterfactual quantity underflowed the available precision")]
    Underflow,
    #[error("floating-point evaluation gave a negative unilateral gain")]
    NegativeGain,
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
/// when the requested length is zero or above [`MAX_ITERATIONS`]. Returns an
/// arithmetic error if a finite payoff, reach or regret cannot be represented.
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
    let game = IndexedGame::new(source, &graph);
    let players = players_of(&graph);
    let mut regrets = zero_table(&graph);
    let mut sums = zero_table(&graph);

    for _ in 0..config.iterations {
        let mut strategy = normalized(&regrets)?;
        for (index, player) in players.iter().enumerate() {
            if index != 0 && config.variant == Variant::CfrPlus {
                strategy = normalized(&regrets)?;
            }
            // Only the updating player's sets change. Vanilla keeps the
            // snapshot fixed until every player has applied this iteration.
            game.update(&strategy, player, &mut regrets, &mut sums, config.variant)?;
        }
    }

    let rows = normalized(&sums)?;
    let root_values = game.profile_values(&rows)?;
    let average_strategy = Strategy {
        action_order: graph
            .sets
            .iter()
            .map(|(id, set)| (id.clone(), set.legal_actions.clone()))
            .collect(),
        probabilities: graph.sets.keys().cloned().zip(rows).collect(),
    };
    Ok(Training {
        variant: config.variant,
        iterations: config.iterations,
        average_strategy,
        root_values,
    })
}

/// Expected payoffs of a validated behavioral strategy for all participants.
///
/// # Errors
///
/// Rejects inconsistent source/partition data, invalid action distributions
/// and out-of-range arithmetic.
pub fn profile_values(
    source: &GameSpecification,
    graph: &InformationSetGraph,
    strategy: &Strategy,
) -> Result<BTreeMap<PlayerId, f64>, Error> {
    let (game, rows) = checked_profile(source, graph, strategy)?;
    game.profile_values(&rows)
}

/// Expected payoff of a unilateral pure best response at the player's information sets.
///
/// # Errors
///
/// Propagates admission, distribution and finite arithmetic errors.
pub fn best_response_value(
    source: &GameSpecification,
    graph: &InformationSetGraph,
    strategy: &Strategy,
    player: &PlayerId,
) -> Result<f64, Error> {
    let (game, rows) = checked_profile(source, graph, strategy)?;
    game.best_response(&rows, player, ResponseMeasure::Payoff)
}

/// Sum of unilateral best-response gains. In the usual two-player zero-sum
/// convention exploitability is half this sum.
///
/// # Errors
///
/// Propagates admission, distribution and arithmetic errors, including a
/// negative unilateral gain due to floating-point evaluation.
pub fn nash_conv(
    source: &GameSpecification,
    graph: &InformationSetGraph,
    strategy: &Strategy,
) -> Result<f64, Error> {
    let (game, rows) = checked_profile(source, graph, strategy)?;
    let mut total = 0.0;
    for player in players_of(graph) {
        let gain = game.best_response(&rows, player, ResponseMeasure::Gain)?;
        if gain < 0.0 {
            return Err(Error::NegativeGain);
        }
        total = finite(total + gain)?;
    }
    Ok(total)
}

fn checked_profile<'a>(
    source: &'a GameSpecification,
    graph: &'a InformationSetGraph,
    strategy: &Strategy,
) -> Result<(IndexedGame<'a>, Vec<Vec<f64>>), Error> {
    // Graphs are public DTOs. Validate even when a caller retained a graph and
    // subsequently changed its membership, assignments, or source.
    let declaration = InformationSetDeclaration {
        schema_version: graph.schema_version.clone(),
        semantic_profile: graph.semantic_profile.clone(),
        sets: graph
            .sets
            .iter()
            .map(|(id, set)| InformationSetDefinition {
                id: id.clone(),
                player: set.player.clone(),
                members: set.members.clone(),
            })
            .collect(),
    };
    if build_information_set_graph(source, &declaration)? != *graph {
        return Err(Error::GraphMismatch);
    }
    if !strategy.action_order.keys().eq(graph.sets.keys())
        || !strategy.probabilities.keys().eq(graph.sets.keys())
    {
        return Err(Error::StrategyDomain);
    }
    for (id, set) in &graph.sets {
        let order = &strategy.action_order[id];
        let row = &strategy.probabilities[id];
        if order.len() != set.legal_actions.len() || row.len() != order.len() {
            return Err(Error::InvalidDistribution(id.clone()));
        }
        for (declared, supplied) in set.legal_actions.iter().zip(order) {
            if supplied != declared {
                return Err(Error::InconsistentActionOrder {
                    node: set.members[0].clone(),
                    action: supplied.clone(),
                });
            }
        }
        if row.iter().any(|p| !(0.0..=1.0).contains(p))
            || (row.iter().sum::<f64>() - 1.0).abs() > 1e-12
        {
            return Err(Error::InvalidDistribution(id.clone()));
        }
    }
    Ok((
        IndexedGame::new(source, graph),
        strategy.probabilities.values().cloned().collect(),
    ))
}

fn players_of(graph: &InformationSetGraph) -> BTreeSet<&PlayerId> {
    graph.sets.values().map(|set| &set.player).collect()
}

fn zero_table(graph: &InformationSetGraph) -> Vec<Vec<f64>> {
    graph
        .sets
        .values()
        .map(|set| vec![0.0; set.legal_actions.len()])
        .collect()
}

// Use the ordinary total when representable (exact for common small
// dyadic counts). Scale by the largest regret if the total overflows.
fn normalized(table: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, Error> {
    table
        .iter()
        .map(|row| {
            let mut weights: Vec<_> = row.iter().map(|r| r.max(0.0)).collect();
            let mut mass = weights.iter().sum::<f64>();
            if mass == 0.0 {
                let width = u32::try_from(row.len()).map_err(|_| Error::NonFiniteArithmetic)?;
                return Ok(vec![1.0 / f64::from(width); row.len()]);
            }
            if !mass.is_finite() {
                let scale = weights.iter().copied().fold(0.0, f64::max);
                for weight in &mut weights {
                    *weight = quotient(*weight, scale)?;
                }
                mass = finite(weights.iter().sum())?;
            }
            for weight in &mut weights {
                *weight = quotient(*weight, mass)?;
            }
            Ok(weights)
        })
        .collect()
}

fn quotient(value: f64, divisor: f64) -> Result<f64, Error> {
    let result = finite(value / divisor)?;
    if result == 0.0 && value != 0.0 {
        return Err(Error::Underflow);
    }
    Ok(result)
}

fn finite(value: f64) -> Result<f64, Error> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(Error::NonFiniteArithmetic)
    }
}
