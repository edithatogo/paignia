//! Explicit reversible identity conversion for one neutral static profile.

use std::collections::{BTreeMap, BTreeSet};

use thiserror::Error;

use crate::{
    ActionEdge, ActionId, ChanceEdge, Game, GameInstanceId, GameSessionId, GameSpecification, Node,
    NodeId, PlayerId, PolicyArtifactId, ResultInformationStructure, ResultSelectionRule,
    SolverDiagnosticTraceId, SolverRunId, STATIC_GAME_PROFILE_V1,
};

/// Version of the neutral static record and its role-specific mapping ledger.
pub const STATIC_CONVERSION_SCHEMA_VERSION: &str = "1.0.0";

/// Identity roles remain distinct even when two full strings are equal.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum IdentityRole {
    Specification,
    Instance,
    Session,
    PolicyArtifact,
    SolverRun,
    SolverDiagnosticTrace,
    Player,
    Node,
    Action,
}

/// Explicit chance-zero declaration; only the existing static rule is admitted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChanceRule {
    AllowZeroAndValidateEveryBranch,
    StrictlyPositiveOnly,
}

/// Explicit payoff interpretation; omission is never inferred as accidental.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PayoffRule {
    IntentionalSparseZero,
    CompleteVectors,
}

/// Neutral, ECS-independent static game plus explicitly supplied context.
#[derive(Clone, Debug, PartialEq)]
pub struct Record {
    pub schema_version: String,
    pub semantic_profile: String,
    pub information_structure: ResultInformationStructure,
    pub chance_rule: ChanceRule,
    pub payoff_rule: PayoffRule,
    pub selection_rule: ResultSelectionRule,
    pub selection_tolerance: f64,
    pub specification: GameSpecification,
    pub instance_id: GameInstanceId,
    pub session_id: Option<GameSessionId>,
    pub policy_artifact_id: Option<PolicyArtifactId>,
    pub solver_run_id: Option<SolverRunId>,
    pub solver_trace_id: Option<SolverDiagnosticTraceId>,
}

/// One explicit full-string identity binding, never a local-name inference.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Binding {
    pub role: IdentityRole,
    pub source: String,
    pub target: String,
}

/// Complete bijection for the identities actually present in a static record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityMap {
    pub schema_version: String,
    pub bindings: Vec<Binding>,
}

/// Converted native record and its complete reversible identity ledger.
#[derive(Clone, Debug, PartialEq)]
pub struct Conversion {
    pub record: Record,
    pub mapping: IdentityMap,
}

/// A rejected conversion, without repairing source values or guessing identity.
#[derive(Clone, Debug, Eq, PartialEq, Error)]
#[error("{code} at {path}: {detail}")]
pub struct AdmissionError {
    pub code: &'static str,
    pub path: String,
    pub detail: String,
}

impl AdmissionError {
    fn new(code: &'static str, path: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            code,
            path: path.into(),
            detail: detail.into(),
        }
    }
}

type IdentityKey = (IdentityRole, String);
type Bindings = BTreeMap<IdentityKey, String>;

impl Record {
    /// Admit the complete declared neutral record without changing it.
    ///
    /// # Errors
    ///
    /// Rejects unknown record/profile versions, unsupported semantics, invalid
    /// selection tolerance or graph, and blank supplied contextual identities.
    pub fn validate(&self) -> Result<(), AdmissionError> {
        if self.schema_version != STATIC_CONVERSION_SCHEMA_VERSION {
            return Err(AdmissionError::new(
                "unsupported_static_record_schema",
                "schema_version",
                &self.schema_version,
            ));
        }
        if self.semantic_profile != STATIC_GAME_PROFILE_V1 {
            return Err(AdmissionError::new(
                "unsupported_semantic_profile",
                "semantic_profile",
                &self.semantic_profile,
            ));
        }
        let semantic_checks = [
            (
                self.information_structure == ResultInformationStructure::PerfectInformation,
                "unsupported_conversion_information",
                "information_structure",
            ),
            (
                self.chance_rule == ChanceRule::AllowZeroAndValidateEveryBranch,
                "unsupported_conversion_chance_rule",
                "chance_rule",
            ),
            (
                self.payoff_rule == PayoffRule::IntentionalSparseZero,
                "unsupported_conversion_payoff_rule",
                "payoff_rule",
            ),
            (
                self.selection_rule
                    == ResultSelectionRule::InputOrderPairwiseToleranceThenLexicographic,
                "unsupported_conversion_selection",
                "selection_rule",
            ),
        ];
        for (supported, code, path) in semantic_checks {
            if !supported {
                return Err(AdmissionError::new(
                    code,
                    path,
                    "the declared rule is not the selected static profile",
                ));
            }
        }
        if !self.selection_tolerance.is_finite() || self.selection_tolerance < 0.0 {
            return Err(AdmissionError::new(
                "invalid_conversion_tolerance",
                "selection_tolerance",
                "a finite nonnegative selection tolerance is required",
            ));
        }
        self.specification
            .validate_for_profile(&self.semantic_profile)
            .map_err(|error| {
                AdmissionError::new(
                    error.code(),
                    format!("specification.{}", error.path()),
                    error.to_string(),
                )
            })?;
        let contextual = [
            ("instance_id", Some(self.instance_id.0.as_str())),
            (
                "session_id",
                self.session_id.as_ref().map(|id| id.0.as_str()),
            ),
            (
                "policy_artifact_id",
                self.policy_artifact_id.as_ref().map(|id| id.0.as_str()),
            ),
            (
                "solver_run_id",
                self.solver_run_id.as_ref().map(|id| id.0.as_str()),
            ),
            (
                "solver_trace_id",
                self.solver_trace_id.as_ref().map(|id| id.0.as_str()),
            ),
        ];
        for (path, value) in contextual {
            if value.is_some_and(|id| id.trim().is_empty()) {
                return Err(AdmissionError::new(
                    "empty_conversion_identity",
                    path,
                    "supplied context must be nonblank; absence remains None",
                ));
            }
        }
        Ok(())
    }

    fn identities(&self) -> BTreeSet<IdentityKey> {
        let mut known = BTreeSet::from([
            (IdentityRole::Specification, self.specification.id.clone()),
            (IdentityRole::Instance, self.instance_id.0.clone()),
        ]);
        for (role, value) in [
            (
                IdentityRole::Session,
                self.session_id.as_ref().map(|id| &id.0),
            ),
            (
                IdentityRole::PolicyArtifact,
                self.policy_artifact_id.as_ref().map(|id| &id.0),
            ),
            (
                IdentityRole::SolverRun,
                self.solver_run_id.as_ref().map(|id| &id.0),
            ),
            (
                IdentityRole::SolverDiagnosticTrace,
                self.solver_trace_id.as_ref().map(|id| &id.0),
            ),
        ] {
            if let Some(id) = value {
                known.insert((role, id.clone()));
            }
        }
        for (id, node) in &self.specification.game.nodes {
            known.insert((IdentityRole::Node, id.0.clone()));
            match node {
                Node::Decision { player, edges } => {
                    known.insert((IdentityRole::Player, player.0.clone()));
                    known.extend(
                        edges
                            .iter()
                            .map(|edge| (IdentityRole::Action, edge.action.0.clone())),
                    );
                }
                Node::Chance { .. } => {}
                Node::Terminal { payoffs } => {
                    known.extend(
                        payoffs
                            .keys()
                            .map(|id| (IdentityRole::Player, id.0.clone())),
                    );
                }
            }
        }
        known
    }
}

impl IdentityMap {
    fn admit(&self, known: &BTreeSet<IdentityKey>) -> Result<Bindings, AdmissionError> {
        if self.schema_version != STATIC_CONVERSION_SCHEMA_VERSION {
            return Err(AdmissionError::new(
                "unsupported_identity_map_schema",
                "mapping.schema_version",
                &self.schema_version,
            ));
        }
        let mut forward = Bindings::new();
        let mut targets = BTreeSet::new();
        for (index, binding) in self.bindings.iter().enumerate() {
            let path = format!("mapping.bindings[{index}]");
            if binding.source.trim().is_empty() || binding.target.trim().is_empty() {
                return Err(AdmissionError::new(
                    "empty_identity_mapping",
                    path,
                    "source and target strings must be nonblank",
                ));
            }
            let key = (binding.role, binding.source.clone());
            if forward.contains_key(&key) {
                return Err(AdmissionError::new(
                    "duplicate_identity_source",
                    path,
                    "each role/source identity requires exactly one binding",
                ));
            }
            if !targets.insert((binding.role, binding.target.clone())) {
                return Err(AdmissionError::new(
                    "identity_target_collision",
                    path,
                    "distinct identities of one role cannot share a target",
                ));
            }
            if !known.contains(&key) {
                return Err(AdmissionError::new(
                    "unknown_identity_mapping",
                    path,
                    "the complete role/source string is absent from this record",
                ));
            }
            forward.insert(key, binding.target.clone());
        }
        for key in known {
            if !forward.contains_key(key) {
                return Err(AdmissionError::new(
                    "missing_identity_mapping",
                    "mapping.bindings",
                    format!("missing {:?} binding for {}", key.0, key.1),
                ));
            }
        }
        Ok(forward)
    }
}

fn rename(bindings: &Bindings, role: IdentityRole, source: &str) -> Result<String, AdmissionError> {
    bindings
        .get(&(role, source.into()))
        .cloned()
        .ok_or_else(|| {
            AdmissionError::new(
                "missing_identity_mapping",
                "mapping.bindings",
                format!("missing {role:?} binding for {source}"),
            )
        })
}

fn check_selection(record: &Record, bindings: &Bindings) -> Result<(), AdmissionError> {
    for (id, node) in &record.specification.game.nodes {
        let Node::Decision { edges, .. } = node else {
            continue;
        };
        let mut ordered: Vec<_> = edges.iter().map(|edge| &edge.action.0).collect();
        ordered.sort();
        let translated: Vec<_> = ordered
            .iter()
            .map(|action| rename(bindings, IdentityRole::Action, action))
            .collect::<Result<_, _>>()?;
        if translated.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(AdmissionError::new(
                "selection_order_changed",
                format!("specification.game.nodes.{}.edges", id.0),
                "renaming changes the lexical order of alternatives at this decision",
            ));
        }
    }
    Ok(())
}

fn convert_graph(game: &Game, bindings: &Bindings) -> Result<Game, AdmissionError> {
    let mut nodes = BTreeMap::new();
    for (id, node) in &game.nodes {
        let converted = match node {
            Node::Decision { player, edges } => Node::Decision {
                player: PlayerId(rename(bindings, IdentityRole::Player, &player.0)?),
                edges: edges
                    .iter()
                    .map(|edge| {
                        Ok(ActionEdge {
                            action: ActionId(rename(
                                bindings,
                                IdentityRole::Action,
                                &edge.action.0,
                            )?),
                            target: NodeId(rename(bindings, IdentityRole::Node, &edge.target.0)?),
                        })
                    })
                    .collect::<Result<_, AdmissionError>>()?,
            },
            Node::Chance { edges } => Node::Chance {
                edges: edges
                    .iter()
                    .map(|edge| {
                        Ok(ChanceEdge {
                            probability: edge.probability,
                            target: NodeId(rename(bindings, IdentityRole::Node, &edge.target.0)?),
                        })
                    })
                    .collect::<Result<_, AdmissionError>>()?,
            },
            Node::Terminal { payoffs } => Node::Terminal {
                payoffs: payoffs
                    .iter()
                    .map(|(player, value)| {
                        Ok((
                            PlayerId(rename(bindings, IdentityRole::Player, &player.0)?),
                            *value,
                        ))
                    })
                    .collect::<Result<_, AdmissionError>>()?,
            },
        };
        nodes.insert(
            NodeId(rename(bindings, IdentityRole::Node, &id.0)?),
            converted,
        );
    }
    Ok(Game {
        root: NodeId(rename(bindings, IdentityRole::Node, &game.root.0)?),
        nodes,
    })
}

/// Convert the declared neutral record with an explicit, complete bijection.
///
/// # Errors
///
/// Rejects unsupported profiles/semantics, invalid source data, incomplete or
/// colliding identities, and a mapping that changes lexical action selection.
pub fn convert(
    record: &Record,
    mapping: &IdentityMap,
    target_profile: &str,
) -> Result<Conversion, AdmissionError> {
    if target_profile != STATIC_GAME_PROFILE_V1 {
        return Err(AdmissionError::new(
            "unsupported_target_profile",
            "target_profile",
            target_profile,
        ));
    }
    record.validate()?;
    let bindings = mapping.admit(&record.identities())?;
    check_selection(record, &bindings)?;
    let mut converted = record.clone();
    converted.specification.id = rename(
        &bindings,
        IdentityRole::Specification,
        &record.specification.id,
    )?;
    converted.specification.game = convert_graph(&record.specification.game, &bindings)?;
    converted.instance_id = GameInstanceId(rename(
        &bindings,
        IdentityRole::Instance,
        &record.instance_id.0,
    )?);
    converted.session_id = record
        .session_id
        .as_ref()
        .map(|id| rename(&bindings, IdentityRole::Session, &id.0).map(GameSessionId))
        .transpose()?;
    converted.policy_artifact_id = record
        .policy_artifact_id
        .as_ref()
        .map(|id| rename(&bindings, IdentityRole::PolicyArtifact, &id.0).map(PolicyArtifactId))
        .transpose()?;
    converted.solver_run_id = record
        .solver_run_id
        .as_ref()
        .map(|id| rename(&bindings, IdentityRole::SolverRun, &id.0).map(SolverRunId))
        .transpose()?;
    converted.solver_trace_id = record
        .solver_trace_id
        .as_ref()
        .map(|id| {
            rename(&bindings, IdentityRole::SolverDiagnosticTrace, &id.0)
                .map(SolverDiagnosticTraceId)
        })
        .transpose()?;
    converted.validate()?;
    Ok(Conversion {
        record: converted,
        mapping: mapping.clone(),
    })
}

/// Restore the source identity representation using the retained mapping.
///
/// # Errors
///
/// Rejects invalid public mutation, unsupported semantics or a broken bijection.
pub fn restore(conversion: &Conversion) -> Result<Record, AdmissionError> {
    let reverse = IdentityMap {
        schema_version: conversion.mapping.schema_version.clone(),
        bindings: conversion
            .mapping
            .bindings
            .iter()
            .map(|binding| Binding {
                role: binding.role,
                source: binding.target.clone(),
                target: binding.source.clone(),
            })
            .collect(),
    };
    convert(&conversion.record, &reverse, STATIC_GAME_PROFILE_V1).map(|converted| converted.record)
}
