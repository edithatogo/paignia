//! Explicit reversible identity conversion for one neutral static profile.

use thiserror::Error;

use crate::{
    GameInstanceId, GameSessionId, GameSpecification, PolicyArtifactId, ResultInformationStructure,
    ResultSelectionRule, SolverDiagnosticTraceId, SolverRunId,
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
pub struct ConversionError {
    pub code: &'static str,
    pub path: String,
    pub detail: String,
}

/// Convert the declared neutral record with an explicit, complete bijection.
///
/// # Errors
///
/// Rejects unsupported profiles/semantics, invalid source data, incomplete or
/// colliding identities, and a mapping that changes lexical action selection.
pub fn convert(
    _record: &Record,
    _mapping: &IdentityMap,
    _target_profile: &str,
) -> Result<Conversion, ConversionError> {
    Err(ConversionError {
        code: "not_implemented",
        path: "conversion".into(),
        detail: "tests-first prototype".into(),
    })
}

/// Restore the source identity representation using the retained mapping.
///
/// # Errors
///
/// Rejects invalid public mutation, unsupported semantics or a broken bijection.
pub fn restore(_conversion: &Conversion) -> Result<Record, ConversionError> {
    Err(ConversionError {
        code: "not_implemented",
        path: "conversion".into(),
        detail: "tests-first prototype".into(),
    })
}
