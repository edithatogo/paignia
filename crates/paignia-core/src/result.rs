//! Versioned structural admission for bounded static solver result records.
//!
//! Admission checks declared metadata and its connection to a supplied graph.
//! It does not execute a solver or certify an equilibrium, a measured residual,
//! a content hash, or a causal simulation trace.

use std::collections::BTreeMap;
use std::str::FromStr;

use thiserror::Error;

use crate::{
    ActionId, GameInstanceId, GameSpecification, NodeId, PayoffVector, ResultDiagnosticRecord,
    ResultIdentityRecord,
};

/// Exact native-envelope schema; no serialized wire format is defined here.
pub const RESULT_ENVELOPE_SCHEMA_VERSION: &str = "1.0.0";
/// Version of the seed's input-order, pairwise-tolerance selection algorithm.
pub const BACKWARD_INDUCTION_ALGORITHM_VERSION: &str = "1.0.0";

/// Declared representation; only the acyclic extensive form is admitted here.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResultGameClass {
    FiniteAcyclicExtensive,
    FiniteNormalForm,
}

/// Declared information structure, independently of algorithm naming.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResultInformationStructure {
    PerfectInformation,
    ImperfectInformation,
    Simultaneous,
}

/// The claimed result kind, not an inferred mathematical certificate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResultSolutionConcept {
    ToleranceAwareBackwardInductionPolicy,
    NashEquilibrium,
    CorrelatedEquilibrium,
}

/// Method identity. A known name alone does not imply profile support.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResultAlgorithm {
    BackwardInduction,
    PureNashEnumeration,
}

/// Exact method and output-semantics version, separate from envelope version.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AlgorithmRecord {
    pub algorithm: ResultAlgorithm,
    pub version: String,
}

/// Execution outcome, separate from the claimed solution concept.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResultStatus {
    Completed,
    Unsupported,
    NotConverged,
    NoEquilibrium,
}

/// The seed's selection is pairwise and can depend on action traversal order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResultSelectionRule {
    InputOrderPairwiseToleranceThenLexicographic,
    Unspecified,
}

/// Completeness of the returned catalogue, not a claim about all game solutions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CatalogueClaim {
    NotApplicable,
    OneFound,
    EnumeratedWithinDeclaredScope {
        scope_id: String,
        completeness_evidence_id: Option<String>,
    },
    Approximate,
}

/// A named, caller-supplied diagnostic. Its definition and value are not proved.
#[derive(Clone, Debug, PartialEq)]
pub struct DeclaredDiagnostic {
    pub definition_id: String,
    pub measurement: ResultDiagnosticRecord,
}

/// A complete contingent policy over graph decision nodes and sparse payoffs.
///
/// Choices include hypothetical, unchosen branches. This is not a causal trace.
#[derive(Clone, Debug, PartialEq)]
pub struct StaticPolicyResult {
    pub choices: BTreeMap<NodeId, ActionId>,
    pub expected_payoffs: PayoffVector,
}

/// Caller-constructed, versioned result metadata and a bounded native payload.
///
/// Public mutation requires revalidation. Existing solver APIs do not construct
/// this envelope automatically. No field adds live state or hidden execution.
#[derive(Clone, Debug, PartialEq)]
pub struct ResultEnvelope {
    pub schema_version: String,
    pub semantic_profile: String,
    pub identities: ResultIdentityRecord,
    pub game_class: ResultGameClass,
    pub information_structure: ResultInformationStructure,
    pub solution_concept: ResultSolutionConcept,
    pub algorithm: AlgorithmRecord,
    pub status: ResultStatus,
    pub status_detail: Option<String>,
    pub tolerance: f64,
    pub selection_rule: ResultSelectionRule,
    pub catalogue: CatalogueClaim,
    pub diagnostics: Vec<DeclaredDiagnostic>,
    pub solutions: Vec<StaticPolicyResult>,
}

/// Stable structural failure with a field path and no inferred repair.
#[derive(Clone, Debug, Eq, PartialEq, Error)]
#[error("{code} at {path}: {detail}")]
pub struct ResultEnvelopeError {
    pub code: &'static str,
    pub path: String,
    pub detail: String,
}

impl ResultEnvelopeError {
    fn new(code: &'static str, path: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            code,
            path: path.into(),
            detail: detail.into(),
        }
    }
}

impl FromStr for ResultAlgorithm {
    type Err = ResultEnvelopeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "paignia/backward-induction" => Ok(Self::BackwardInduction),
            "paignia/pure-nash-enumeration" => Ok(Self::PureNashEnumeration),
            _ => Err(ResultEnvelopeError::new(
                "unsupported_result_algorithm",
                "algorithm.algorithm",
                value,
            )),
        }
    }
}

impl FromStr for ResultSolutionConcept {
    type Err = ResultEnvelopeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "tolerance-aware-backward-induction-policy" => {
                Ok(Self::ToleranceAwareBackwardInductionPolicy)
            }
            "nash-equilibrium" => Ok(Self::NashEquilibrium),
            "correlated-equilibrium" => Ok(Self::CorrelatedEquilibrium),
            _ => Err(ResultEnvelopeError::new(
                "unsupported_result_solution_concept",
                "solution_concept",
                value,
            )),
        }
    }
}

impl ResultEnvelope {
    /// Check the envelope against a caller-supplied specification and instance.
    ///
    /// Binding compares complete ID strings and checks policy references in the
    /// supplied graph. It is not cryptographic or historical source attestation.
    ///
    /// # Errors
    ///
    /// Returns a structured error for unsupported versions or semantic claims,
    /// invalid identities/numbers, inconsistent status/catalogue, or an illegal
    /// or incomplete policy. Successful admission is not a solver certificate.
    pub fn validate_against(
        &self,
        _specification: &GameSpecification,
        _expected_instance: &GameInstanceId,
    ) -> Result<(), ResultEnvelopeError> {
        // Explicit tests-first prototype: no admission checks yet.
        Ok(())
    }
}
