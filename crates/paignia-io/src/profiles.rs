//! Versioned information-set, normal-form and discounted game projections.

use std::collections::{BTreeMap, BTreeSet};

use paignia_core::discounted::{Game as DiscountedGame, Outcome, State, StationaryPolicy};
use paignia_core::{
    build_information_set_graph, ActionId, Game, GameSpecification, InformationSetDeclaration,
    InformationSetDefinition, InformationSetId, NodeId, PlayerId, GAME_SPEC_SCHEMA_VERSION,
    INFORMATION_SET_PROFILE_V1,
};
use paignia_solve::discounted::{self, Config as DiscountedConfig, Evaluation, Status};
use paignia_solve::{
    cfr_nash_conv, pure_nash_equilibria, solve_mixed_nash, train_counterfactual_regret, CfrConfig,
    CfrVariant, MixedNashAlgorithm, MixedNashConfig, MixedNashStatus, NormalFormGame, SolverConfig,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{
    admit_source_id, build_nodes, is_hex, NamedSource, NodeRecord, Payoff, Source,
    MAX_REQUEST_BYTES, WIRE_VERSION,
};

pub const NORMAL_FORM_PROFILE: &str = "paignia/finite-two-player-normal-form/1.0.0";
pub const DISCOUNTED_PROFILE: &str = "paignia/discounted-zero-sum-2x2/1.0.0";

#[derive(Debug, thiserror::Error)]
pub enum ProfileError {
    #[error("profile request exceeds {MAX_REQUEST_BYTES} bytes")]
    RequestTooLarge,
    #[error("invalid profile JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("unsupported wire version {0}")]
    Version(String),
    #[error("unsupported profile {0}")]
    Profile(String),
    #[error("invalid source or run identity")]
    Identity,
    #[error("invalid or incomplete two-player payoff table")]
    PayoffTable,
    #[error(transparent)]
    Topology(#[from] crate::Error),
    #[error(transparent)]
    InformationSet(#[from] paignia_core::InformationSetError),
    #[error("CFR: {0}")]
    Cfr(String),
    #[error("Nash: {0}")]
    Nash(String),
    #[error(transparent)]
    Discounted(#[from] discounted::Error),
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    version: String,
    semantic_profile: String,
    source: Source,
    specification: String,
    instance: String,
    run: String,
    game: Input,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Input {
    PerfectRecall {
        root: String,
        nodes: Vec<NodeRecord>,
        declaration: Partition,
        method: RegretMethod,
        iterations: u32,
    },
    NormalForm {
        players: [Seat; 2],
        payoffs: Vec<Vec<NormalCell>>,
        method: NashMethod,
    },
    Discounted(DiscountedRequest),
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DiscountedRequest {
    players: [DiscountedSeat; 2],
    states: Vec<DiscountedState>,
    initial: Vec<f64>,
    discount: f64,
    method: DiscountedMethod,
    max_iterations: u32,
    value_tolerance: f64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DiscountedSeat {
    id: String,
    actions: [String; 2],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DiscountedState {
    id: String,
    outcomes: [[DiscountedOutcome; 2]; 2],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DiscountedOutcome {
    reward: f64,
    transition: Vec<f64>,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum DiscountedMethod {
    Evaluate { policy: Vec<[[f64; 2]; 2]> },
    Minimax {},
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RegretMethod {
    Cfr,
    CfrPlus,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum NashMethod {
    Pure,
    Mixed,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Partition {
    schema_version: String,
    semantic_profile: String,
    sets: Vec<SetRecord>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SetRecord {
    id: String,
    player: String,
    members: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Seat {
    id: String,
    strategies: Vec<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct NormalCell {
    profile: String,
    payoff_profile: String,
    payoffs: [Payoff; 2],
}

#[derive(Serialize)]
struct Response {
    version: &'static str,
    semantic_profile: String,
    source: Source,
    specification: String,
    instance: String,
    run: String,
    method: &'static str,
    method_version: &'static str,
    status: &'static str,
    result: Value,
}

type MethodResult = (&'static str, &'static str, &'static str, Value);

/// Solve a versioned perfect-recall, normal-form or discounted game request.
///
/// The RDF caller is responsible for the asserted graph and its digest. This
/// boundary checks the projected topology, partition or payoff table and
/// retains source and result identities without normalizing their strings.
///
/// # Errors
/// Rejects malformed or unbounded wire input, invalid model data and failures
/// of native admission or calculation.
pub fn solve_profile_json(input: &[u8]) -> Result<Vec<u8>, ProfileError> {
    if input.len() > MAX_REQUEST_BYTES {
        return Err(ProfileError::RequestTooLarge);
    }
    let request: Request = serde_json::from_slice(input)?;
    if request.version != WIRE_VERSION {
        return Err(ProfileError::Version(request.version));
    }
    if !is_hex(&request.source.ontology_revision, 40)
        || !is_hex(&request.source.rdf_sha256, 64)
        || [
            &request.source.representation,
            &request.specification,
            &request.instance,
            &request.run,
        ]
        .into_iter()
        .any(|id| id.trim().is_empty())
    {
        return Err(ProfileError::Identity);
    }
    let expected = match &request.game {
        Input::PerfectRecall { .. } => INFORMATION_SET_PROFILE_V1,
        Input::NormalForm { .. } => NORMAL_FORM_PROFILE,
        Input::Discounted(_) => DISCOUNTED_PROFILE,
    };
    if request.semantic_profile != expected {
        return Err(ProfileError::Profile(request.semantic_profile));
    }
    let (method, method_version, status, result) = match request.game {
        Input::PerfectRecall {
            root,
            nodes,
            declaration,
            method,
            iterations,
        } => regret(
            request.specification.clone(),
            Game {
                root: NodeId(root),
                nodes: build_nodes(nodes)?,
            },
            declaration,
            method,
            iterations,
        )?,
        Input::NormalForm {
            players,
            payoffs,
            method,
        } => {
            let matrix = native_payoff_table(&players, &payoffs)?;
            let (algorithm, version, status, mut result) = nash(&players, &matrix, method)?;
            result["source_cells"] = json!(payoffs);
            (algorithm, version, status, result)
        }
        Input::Discounted(game) => discounted_result(game)?,
    };
    Ok(serde_json::to_vec_pretty(&Response {
        version: WIRE_VERSION,
        semantic_profile: request.semantic_profile,
        source: request.source,
        specification: request.specification,
        instance: request.instance,
        run: request.run,
        method,
        method_version,
        status,
        result,
    })?)
}

fn evaluation_status(status: Status) -> &'static str {
    match status {
        Status::ToleranceReached => "value_tolerance_reached",
        Status::IterationLimit => "iteration_limit",
    }
}

fn evaluation_json(evaluation: &Evaluation) -> Value {
    let state_values: Vec<_> = evaluation
        .state_values
        .iter()
        .map(|(state, value)| json!({ "state": state.0, "value": value }))
        .collect();
    json!({
        "status": evaluation_status(evaluation.status),
        "state_values": state_values,
        "initial_value": evaluation.initial_value,
        "iterations": evaluation.iterations,
        "residual_bound": evaluation.residual_bound,
        "value_error_bound": evaluation.value_error_bound,
        "initial_error_bound": evaluation.initial_error_bound,
    })
}

fn discounted_result(request: DiscountedRequest) -> Result<MethodResult, ProfileError> {
    let [row, column] = request.players;
    let game = DiscountedGame {
        players: [PlayerId(row.id), PlayerId(column.id)],
        actions: [row.actions.map(ActionId), column.actions.map(ActionId)],
        states: request
            .states
            .into_iter()
            .map(|state| State {
                id: NodeId(state.id),
                outcomes: state.outcomes.map(|actions| {
                    actions.map(|outcome| Outcome {
                        reward: outcome.reward,
                        transition: outcome.transition,
                    })
                }),
            })
            .collect(),
        initial: request.initial,
        discount: request.discount,
    };
    let config = DiscountedConfig {
        max_iterations: request.max_iterations,
        value_tolerance: request.value_tolerance,
    };
    let (method, status, policy, mut result) = match request.method {
        DiscountedMethod::Evaluate { policy } => {
            let policy = StationaryPolicy { weights: policy };
            let evaluation = discounted::evaluate(&game, &policy, config)?;
            (
                "stationary_policy_evaluation",
                evaluation_status(evaluation.status),
                policy,
                json!({ "policy_values": evaluation_json(&evaluation) }),
            )
        }
        DiscountedMethod::Minimax {} => {
            let equilibrium = discounted::solve_zero_sum(&game, config)?;
            let evaluations = [
                &equilibrium.game_values,
                &equilibrium.policy_values,
                &equilibrium.row_best_response,
                &equilibrium.column_best_response,
            ];
            let status = if evaluations
                .iter()
                .all(|evaluation| evaluation.status == Status::ToleranceReached)
            {
                Status::ToleranceReached
            } else {
                Status::IterationLimit
            };
            (
                "discounted_zero_sum_minimax",
                evaluation_status(status),
                equilibrium.policy,
                json!({
                    "game_values": evaluation_json(evaluations[0]),
                    "policy_values": evaluation_json(evaluations[1]),
                    "row_best_response": evaluation_json(evaluations[2]),
                    "column_best_response": evaluation_json(evaluations[3]),
                    "deviation_bounds": equilibrium.deviation_bounds,
                }),
            )
        }
    };
    result["players"] = json!([
        { "id": game.players[0].0, "actions": [&game.actions[0][0].0, &game.actions[0][1].0] },
        { "id": game.players[1].0, "actions": [&game.actions[1][0].0, &game.actions[1][1].0] },
    ]);
    result["policy"] = json!(policy.weights);
    Ok((method, "1.0.0", status, result))
}

fn regret(
    specification: String,
    game: Game,
    declaration: Partition,
    method: RegretMethod,
    iterations: u32,
) -> Result<MethodResult, ProfileError> {
    let source = GameSpecification {
        schema_version: GAME_SPEC_SCHEMA_VERSION.into(),
        id: specification,
        game,
    };
    let declaration = InformationSetDeclaration {
        schema_version: declaration.schema_version,
        semantic_profile: declaration.semantic_profile,
        sets: declaration
            .sets
            .into_iter()
            .map(|set| InformationSetDefinition {
                id: InformationSetId(set.id),
                player: PlayerId(set.player),
                members: set.members.into_iter().map(NodeId).collect(),
            })
            .collect(),
    };
    let graph = build_information_set_graph(&source, &declaration)?;
    let variant = match method {
        RegretMethod::Cfr => CfrVariant::Cfr,
        RegretMethod::CfrPlus => CfrVariant::CfrPlus,
    };
    let trained = train_counterfactual_regret(
        &source,
        &declaration,
        &CfrConfig {
            variant,
            iterations,
        },
    )
    .map_err(|error| ProfileError::Cfr(error.to_string()))?;
    let policy: BTreeMap<_, Vec<_>> = trained
        .average_strategy
        .action_order
        .iter()
        .map(|(set, actions)| {
            let probabilities = &trained.average_strategy.probabilities[set];
            (
                set.0.as_str(),
                actions
                    .iter()
                    .map(|action| action.0.as_str())
                    .zip(probabilities.iter().copied())
                    .collect(),
            )
        })
        .collect();
    let values: BTreeMap<_, _> = trained
        .root_values
        .iter()
        .map(|(player, value)| (player.0.as_str(), value))
        .collect();
    let nash_conv = cfr_nash_conv(&source, &graph, &trained.average_strategy)
        .map_err(|error| ProfileError::Cfr(error.to_string()))?;
    Ok((
        variant.id(),
        paignia_solve::CFR_VERSION,
        "training_completed",
        json!({
            "iterations": trained.iterations, "policy": policy,
            "expected_payoffs": values, "nash_conv": nash_conv,
        }),
    ))
}

fn native_payoff_table(
    players: &[Seat; 2],
    payoffs: &[Vec<NormalCell>],
) -> Result<Vec<Vec<[f64; 2]>>, ProfileError> {
    let [row, column] = players;
    if row.id == column.id
        || players.iter().any(|seat| {
            seat.id.trim().is_empty()
                || !(1..=paignia_solve::MAX_MIXED_NASH_ACTIONS).contains(&seat.strategies.len())
                || seat
                    .strategies
                    .iter()
                    .any(|action| action.trim().is_empty())
                || seat.strategies.iter().collect::<BTreeSet<_>>().len() != seat.strategies.len()
        })
        || payoffs.len() != row.strategies.len()
        || payoffs
            .iter()
            .any(|cells| cells.len() != column.strategies.len())
    {
        return Err(ProfileError::PayoffTable);
    }
    let mut profiles = BTreeSet::new();
    let mut payoff_profiles = BTreeMap::new();
    let mut links = BTreeMap::new();
    payoffs
        .iter()
        .map(|cells| {
            cells
                .iter()
                .map(|cell| {
                    if cell.profile.trim().is_empty()
                        || !profiles.insert(cell.profile.as_str())
                        || cell.payoff_profile.trim().is_empty()
                    {
                        return Err(ProfileError::PayoffTable);
                    }
                    let payoff_ids = [cell.payoffs[0].id.as_str(), cell.payoffs[1].id.as_str()];
                    if payoff_profiles
                        .insert(cell.payoff_profile.as_str(), payoff_ids)
                        .is_some_and(|prior| prior != payoff_ids)
                    {
                        return Err(ProfileError::PayoffTable);
                    }
                    for (payoff, player) in cell.payoffs.iter().zip(players) {
                        if payoff.player != player.id {
                            return Err(ProfileError::PayoffTable);
                        }
                        admit_source_id(
                            &mut links,
                            payoff.id.clone(),
                            NamedSource::Payoff {
                                player: payoff.player.clone(),
                                value_bits: payoff.value.to_bits(),
                            },
                        )?;
                    }
                    Ok([cell.payoffs[0].value, cell.payoffs[1].value])
                })
                .collect()
        })
        .collect()
}

fn nash(
    players: &[Seat; 2],
    payoffs: &[Vec<[f64; 2]>],
    method: NashMethod,
) -> Result<MethodResult, ProfileError> {
    let [row, column] = players;
    match method {
        NashMethod::Pure => pure_nash(row, column, payoffs),
        NashMethod::Mixed => mixed_nash(row, column, payoffs),
    }
}

fn pure_nash(
    row: &Seat,
    column: &Seat,
    payoffs: &[Vec<[f64; 2]>],
) -> Result<MethodResult, ProfileError> {
    let strategies = [&row.strategies, &column.strategies];
    let ids = [PlayerId(row.id.clone()), PlayerId(column.id.clone())];
    let config = SolverConfig::default();
    let mut table = BTreeMap::new();
    for (i, row_payoffs) in payoffs.iter().enumerate() {
        for (j, cell) in row_payoffs.iter().enumerate() {
            table.insert(
                BTreeMap::from([
                    (ids[0].clone(), ActionId(row.strategies[i].clone())),
                    (ids[1].clone(), ActionId(column.strategies[j].clone())),
                ]),
                ids.iter().cloned().zip(cell.iter().copied()).collect(),
            );
        }
    }
    let game = NormalFormGame {
        players: ids.to_vec(),
        strategies: ids
            .iter()
            .cloned()
            .zip(strategies.map(|actions| actions.iter().cloned().map(ActionId).collect()))
            .collect(),
        payoffs: table,
    };
    let profiles = pure_nash_equilibria(&game, config)
        .map_err(|error| ProfileError::Nash(error.to_string()))?
        .iter()
        .map(|profile| {
            profile
                .iter()
                .map(|(player, action)| (player.0.clone(), action.0.clone()))
                .collect::<BTreeMap<_, _>>()
        })
        .collect::<Vec<_>>();
    Ok((
        "paignia/pure-nash",
        "1.0.0",
        "completed",
        json!({
            "profiles": profiles, "tolerance": config.tolerance,
        }),
    ))
}

fn mixed_nash(
    row: &Seat,
    column: &Seat,
    payoffs: &[Vec<[f64; 2]>],
) -> Result<MethodResult, ProfileError> {
    let [a, b]: [Vec<Vec<f64>>; 2] = [0, 1].map(|player| {
        payoffs
            .iter()
            .map(|cells| cells.iter().map(|payoff| payoff[player]).collect())
            .collect()
    });
    let native = solve_mixed_nash(
        &a,
        &b,
        MixedNashAlgorithm::SupportEnumeration,
        MixedNashConfig::default(),
    )
    .map_err(|error| ProfileError::Nash(error.to_string()))?;
    let status = match native.status {
        MixedNashStatus::CompletedWithinScope => "completed_within_scope",
        MixedNashStatus::UnresolvedSupports => "unresolved_supports",
        MixedNashStatus::BudgetExhausted => "budget_exhausted",
        MixedNashStatus::NoVerifiedWitness => "no_verified_witness",
        MixedNashStatus::UnsupportedAlgorithm => "unsupported_algorithm",
    };
    let candidates: Vec<_> = native
        .candidates
        .iter()
        .map(|candidate| {
            let by_action = |seat: &Seat, probs: &[f64]| {
                seat.strategies
                    .iter()
                    .cloned()
                    .zip(probs.iter().copied())
                    .collect::<Vec<_>>()
            };
            json!({
                "strategy": {
                    &row.id: by_action(row, &candidate.row_strategy),
                    &column.id: by_action(column, &candidate.column_strategy),
                },
                "expected_payoffs": {
                    &row.id: candidate.diagnostics.expected_payoffs[0],
                    &column.id: candidate.diagnostics.expected_payoffs[1],
                },
                "max_gain": candidate.diagnostics.max_gain,
            })
        })
        .collect();
    Ok((
        native.algorithm.id(),
        native
            .algorithm_version
            .ok_or_else(|| ProfileError::Nash("native result omitted algorithm version".into()))?,
        status,
        json!({
            "candidates": candidates,
            "visited_support_pairs": native.visited_support_pairs,
            "unresolved_support_pairs": native.unresolved_support_pairs,
            "equilibrium_tolerance": native.config.equilibrium_tolerance,
        }),
    ))
}
