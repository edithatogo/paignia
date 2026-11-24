//! Opt-in JSON interchange for bounded, finite game profiles.
//!
//! The request carries declared source identities. The caller checks those
//! bindings against its source. The solver and graph validation are native.

mod profiles;
pub use profiles::{solve_profile_json, ProfileError, NORMAL_FORM_PROFILE};

use std::collections::{btree_map::Entry, BTreeMap};

use paignia_core::{
    ActionEdge, ActionId, ChanceEdge, Game, GameInstanceId, GameSpecification, GameSpecificationId,
    Node, NodeId, PlayerId, ResultAlgorithm, ResultIdentityRecord, ResultStatus, SolverRunId,
    GAME_SPEC_SCHEMA_VERSION, RESULT_IDENTITY_SCHEMA_VERSION,
};
use paignia_solve::{backward_induction_result, ResultSolveError, SolverConfig};
use serde::{Deserialize, Serialize};

pub const WIRE_VERSION: &str = "1.0.0";
pub const MAX_REQUEST_BYTES: usize = 1_048_576;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("request exceeds {MAX_REQUEST_BYTES} bytes")]
    RequestTooLarge,
    #[error("invalid JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("unsupported interchange version {0}")]
    Version(String),
    #[error("invalid source provenance")]
    Provenance,
    #[error("duplicate node identifier {0}")]
    DuplicateNode(String),
    #[error("conflicting, repeated transition, or empty source identifier {0}")]
    SourceIdentity(String),
    #[error("duplicate payoff for player {player} at {node}")]
    DuplicatePayoff { player: String, node: String },
    #[error("native result is outside the completed static policy profile")]
    UnsupportedResult,
    #[error(transparent)]
    Native(#[from] ResultSolveError),
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Source {
    ontology_revision: String,
    rdf_sha256: String,
    representation: String,
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
    root: String,
    nodes: Vec<NodeRecord>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NodeRecord {
    id: String,
    node: NodeRecordKind,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum NodeRecordKind {
    Decision {
        player: String,
        actions: Vec<DecisionEdge>,
    },
    Chance {
        outcomes: Vec<ChanceOutcome>,
    },
    Terminal {
        payoffs: Vec<Payoff>,
    },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DecisionEdge {
    id: String,
    action: String,
    target: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ChanceOutcome {
    id: String,
    probability: f64,
    target: String,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Payoff {
    id: String,
    player: String,
    value: f64,
}

#[derive(Serialize)]
struct Response<'a> {
    version: &'static str,
    native_result_version: &'a str,
    semantic_profile: &'a str,
    source: Source,
    specification: &'a str,
    instance: &'a str,
    run: &'a str,
    method: &'static str,
    method_version: &'a str,
    status: &'static str,
    tolerance: f64,
    choices: BTreeMap<&'a str, &'a str>,
    expected_payoffs: BTreeMap<&'a str, f64>,
}

/// Solve a versioned static game request and emit its completed JSON policy.
///
/// # Errors
/// Rejects oversized or malformed requests, unknown wire versions, duplicate
/// identities, invalid games and failures of native computation or admission.
pub fn solve_static_json(input: &[u8]) -> Result<Vec<u8>, Error> {
    if input.len() > MAX_REQUEST_BYTES {
        return Err(Error::RequestTooLarge);
    }
    let request: Request = serde_json::from_slice(input)?;
    if request.version != WIRE_VERSION {
        return Err(Error::Version(request.version));
    }
    if !is_hex(&request.source.ontology_revision, 40)
        || !is_hex(&request.source.rdf_sha256, 64)
        || request.source.representation.trim().is_empty()
    {
        return Err(Error::Provenance);
    }
    let identities = ResultIdentityRecord {
        schema_version: RESULT_IDENTITY_SCHEMA_VERSION.into(),
        specification_id: GameSpecificationId(request.specification.clone()),
        game_instance_id: GameInstanceId(request.instance),
        solver_run_id: SolverRunId(request.run),
        session_id: None,
        policy_artifact_id: None,
        solver_trace_id: None,
    };
    let specification = GameSpecification {
        schema_version: GAME_SPEC_SCHEMA_VERSION.into(),
        id: request.specification,
        game: Game {
            root: NodeId(request.root),
            nodes: build_nodes(request.nodes)?,
        },
    };
    let envelope = backward_induction_result(
        &specification,
        &request.semantic_profile,
        identities,
        SolverConfig::default(),
    )?
    .envelope;
    let [policy] = envelope.solutions.as_slice() else {
        return Err(Error::UnsupportedResult);
    };
    if envelope.status != ResultStatus::Completed
        || envelope.algorithm.algorithm != ResultAlgorithm::BackwardInduction
    {
        return Err(Error::UnsupportedResult);
    }
    let response = Response {
        version: WIRE_VERSION,
        native_result_version: &envelope.schema_version,
        semantic_profile: &envelope.semantic_profile,
        source: request.source,
        specification: &envelope.identities.specification_id.0,
        instance: &envelope.identities.game_instance_id.0,
        run: &envelope.identities.solver_run_id.0,
        method: "backward_induction",
        method_version: &envelope.algorithm.version,
        status: "completed",
        tolerance: envelope.tolerance,
        choices: policy
            .choices
            .iter()
            .map(|(node, action)| (node.0.as_str(), action.0.as_str()))
            .collect(),
        expected_payoffs: policy
            .expected_payoffs
            .iter()
            .map(|(player, payoff)| (player.0.as_str(), *payoff))
            .collect(),
    };
    Ok(serde_json::to_vec_pretty(&response)?)
}

fn is_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[derive(PartialEq)]
enum NamedSource {
    Transition,
    Payoff { player: String, value_bits: u64 },
}

fn admit_source_id(
    seen: &mut BTreeMap<String, NamedSource>,
    id: String,
    role: NamedSource,
) -> Result<(), Error> {
    if id.trim().is_empty() {
        return Err(Error::SourceIdentity(id));
    }
    match seen.entry(id) {
        Entry::Vacant(slot) => {
            slot.insert(role);
            Ok(())
        }
        Entry::Occupied(slot)
            if matches!(role, NamedSource::Payoff { .. }) && slot.get() == &role =>
        {
            Ok(())
        }
        Entry::Occupied(slot) => Err(Error::SourceIdentity(slot.key().clone())),
    }
}

fn build_nodes(records: Vec<NodeRecord>) -> Result<BTreeMap<NodeId, Node>, Error> {
    let mut nodes = BTreeMap::new();
    let mut source_ids = BTreeMap::new();
    for NodeRecord { id, node } in records {
        let slot = match nodes.entry(NodeId(id)) {
            Entry::Vacant(slot) => slot,
            Entry::Occupied(slot) => return Err(Error::DuplicateNode(slot.key().0.clone())),
        };
        let native = match node {
            NodeRecordKind::Decision { player, actions } => {
                let mut edges = Vec::with_capacity(actions.len());
                for edge in actions {
                    admit_source_id(&mut source_ids, edge.id, NamedSource::Transition)?;
                    edges.push(ActionEdge {
                        action: ActionId(edge.action),
                        target: NodeId(edge.target),
                    });
                }
                Node::Decision {
                    player: PlayerId(player),
                    edges,
                }
            }
            NodeRecordKind::Chance { outcomes } => {
                let mut edges = Vec::with_capacity(outcomes.len());
                for outcome in outcomes {
                    admit_source_id(&mut source_ids, outcome.id, NamedSource::Transition)?;
                    edges.push(ChanceEdge {
                        probability: outcome.probability,
                        target: NodeId(outcome.target),
                    });
                }
                Node::Chance { edges }
            }
            NodeRecordKind::Terminal { payoffs } => {
                let mut values = BTreeMap::new();
                for payoff in payoffs {
                    admit_source_id(
                        &mut source_ids,
                        payoff.id,
                        NamedSource::Payoff {
                            player: payoff.player.clone(),
                            value_bits: payoff.value.to_bits(),
                        },
                    )?;
                    if values
                        .insert(PlayerId(payoff.player.clone()), payoff.value)
                        .is_some()
                    {
                        return Err(Error::DuplicatePayoff {
                            node: slot.key().0.clone(),
                            player: payoff.player,
                        });
                    }
                }
                Node::Terminal { payoffs: values }
            }
        };
        slot.insert(native);
    }
    Ok(nodes)
}
