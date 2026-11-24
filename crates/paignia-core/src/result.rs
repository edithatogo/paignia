//! Versioned structural admission for bounded static solver result records.
//!
//! Admission checks declared metadata and its connection to a supplied graph.
//! It does not execute a solver or certify an equilibrium, a measured residual,
//! a content hash, or a causal simulation trace.

use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;

use thiserror::Error;

use crate::{
    ActionId, GameInstanceId, GameSpecification, Node, NodeId, PayoffVector,
    ResultDiagnosticRecord, ResultIdentityRecord, STATIC_GAME_PROFILE_V1,
};

/// Exact native-envelope schema; no serialized wire format is defined here.
pub const RESULT_ENVELOPE_SCHEMA_VERSION: &str = "1.0.0";
/// Current input-order, pairwise-tolerance method with compensated chance sums.
pub const BACKWARD_INDUCTION_ALGORITHM_VERSION: &str = "1.0.1";
const LEGACY_BACKWARD_INDUCTION_ALGORITHM_VERSION: &str = "1.0.0";

/// Declared representation; only the acyclic extensive form is admitted here.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GameClass {
    FiniteAcyclicExtensive,
    FiniteNormalForm,
}

/// Declared information structure, independently of algorithm naming.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InformationStructure {
    PerfectInformation,
    ImperfectInformation,
    Simultaneous,
}

/// The claimed result kind, not an inferred mathematical certificate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SolutionConcept {
    ToleranceAwareBackwardInductionPolicy,
    NashEquilibrium,
    CorrelatedEquilibrium,
}

/// Method identity. A known name alone does not imply profile support.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Algorithm {
    BackwardInduction,
    PureNashEnumeration,
}

/// Exact method and output-semantics version, separate from envelope version.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AlgorithmRecord {
    pub algorithm: Algorithm,
    pub version: String,
}

/// Execution outcome, separate from the claimed solution concept.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Status {
    Completed,
    Unsupported,
    NotConverged,
    NoEquilibrium,
}

/// The seed's selection is pairwise and can depend on action traversal order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectionRule {
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
pub struct StaticPolicy {
    pub choices: BTreeMap<NodeId, ActionId>,
    pub expected_payoffs: PayoffVector,
}

/// Caller-constructed, versioned result metadata and a bounded native payload.
///
/// Public mutation requires revalidation. Existing solver APIs do not construct
/// this envelope automatically. No field adds live state or hidden execution.
#[derive(Clone, Debug, PartialEq)]
pub struct Envelope {
    pub schema_version: String,
    pub semantic_profile: String,
    pub identities: ResultIdentityRecord,
    pub game_class: GameClass,
    pub information_structure: InformationStructure,
    pub solution_concept: SolutionConcept,
    pub algorithm: AlgorithmRecord,
    pub status: Status,
    pub status_detail: Option<String>,
    pub tolerance: f64,
    pub selection_rule: SelectionRule,
    pub catalogue: CatalogueClaim,
    pub diagnostics: Vec<DeclaredDiagnostic>,
    pub solutions: Vec<StaticPolicy>,
}

/// Stable structural failure with a field path and no inferred repair.
#[derive(Clone, Debug, Eq, PartialEq, Error)]
#[error("{code} at {path}: {detail}")]
pub struct EnvelopeError {
    pub code: &'static str,
    pub path: String,
    pub detail: String,
}

impl EnvelopeError {
    fn new(code: &'static str, path: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            code,
            path: path.into(),
            detail: detail.into(),
        }
    }
}

impl FromStr for Algorithm {
    type Err = EnvelopeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "paignia/backward-induction" => Ok(Self::BackwardInduction),
            "paignia/pure-nash-enumeration" => Ok(Self::PureNashEnumeration),
            _ => Err(EnvelopeError::new(
                "unsupported_result_algorithm",
                "algorithm.algorithm",
                value,
            )),
        }
    }
}

impl FromStr for SolutionConcept {
    type Err = EnvelopeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "tolerance-aware-backward-induction-policy" => {
                Ok(Self::ToleranceAwareBackwardInductionPolicy)
            }
            "nash-equilibrium" => Ok(Self::NashEquilibrium),
            "correlated-equilibrium" => Ok(Self::CorrelatedEquilibrium),
            _ => Err(EnvelopeError::new(
                "unsupported_result_solution_concept",
                "solution_concept",
                value,
            )),
        }
    }
}

impl Envelope {
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
        specification: &GameSpecification,
        expected_instance: &GameInstanceId,
    ) -> Result<(), EnvelopeError> {
        if self.schema_version != RESULT_ENVELOPE_SCHEMA_VERSION {
            return Err(EnvelopeError::new(
                "unsupported_result_schema",
                "schema_version",
                &self.schema_version,
            ));
        }
        if self.semantic_profile != STATIC_GAME_PROFILE_V1 {
            return Err(EnvelopeError::new(
                "unsupported_semantic_profile",
                "semantic_profile",
                &self.semantic_profile,
            ));
        }
        self.identities.validate().map_err(|error| {
            EnvelopeError::new(
                error.code(),
                format!("identities.{}", error.path()),
                error.to_string(),
            )
        })?;
        specification
            .validate_for_profile(&self.semantic_profile)
            .map_err(|error| {
                EnvelopeError::new(
                    error.code(),
                    format!("specification.{}", error.path()),
                    error.to_string(),
                )
            })?;
        for (matches, path) in [
            (
                self.identities.specification_id.0 == specification.id,
                "identities.specification_id",
            ),
            (
                &self.identities.game_instance_id == expected_instance,
                "identities.game_instance_id",
            ),
        ] {
            if !matches {
                return Err(EnvelopeError::new(
                    "result_source_mismatch",
                    path,
                    "identity differs from the supplied source binding",
                ));
            }
        }
        self.validate_claims()?;
        if !self.tolerance.is_finite() || self.tolerance < 0.0 {
            return Err(EnvelopeError::new(
                "invalid_result_tolerance",
                "tolerance",
                "expected a finite, nonnegative tolerance",
            ));
        }
        if self.status != Status::Completed {
            return self.validate_failure();
        }
        if self.catalogue != CatalogueClaim::OneFound {
            return Err(EnvelopeError::new(
                "unsupported_result_catalogue",
                "catalogue",
                "this method returns one contingent policy, not an equilibrium catalogue",
            ));
        }
        if self.solutions.len() != 1 {
            return Err(EnvelopeError::new(
                "invalid_result_catalogue",
                "solutions",
                "one_found requires exactly one policy",
            ));
        }
        self.validate_diagnostics()?;
        self.solutions[0].validate_against(specification)?;
        Ok(())
    }

    fn validate_claims(&self) -> Result<(), EnvelopeError> {
        for (supported, path) in [
            (
                self.game_class == GameClass::FiniteAcyclicExtensive,
                "game_class",
            ),
            (
                self.information_structure == InformationStructure::PerfectInformation,
                "information_structure",
            ),
            (
                self.solution_concept == SolutionConcept::ToleranceAwareBackwardInductionPolicy,
                "solution_concept",
            ),
            (
                self.algorithm.algorithm == Algorithm::BackwardInduction,
                "algorithm.algorithm",
            ),
            (
                self.selection_rule == SelectionRule::InputOrderPairwiseToleranceThenLexicographic,
                "selection_rule",
            ),
        ] {
            if !supported {
                return Err(EnvelopeError::new(
                    "unsupported_result_claim",
                    path,
                    "claim is outside the admitted static backward-induction profile",
                ));
            }
        }
        if !matches!(
            self.algorithm.version.as_str(),
            LEGACY_BACKWARD_INDUCTION_ALGORITHM_VERSION | BACKWARD_INDUCTION_ALGORITHM_VERSION
        ) {
            return Err(EnvelopeError::new(
                "unsupported_result_algorithm_version",
                "algorithm.version",
                &self.algorithm.version,
            ));
        }
        Ok(())
    }

    fn validate_failure(&self) -> Result<(), EnvelopeError> {
        if self.status == Status::NoEquilibrium {
            return Err(EnvelopeError::new(
                "unsupported_result_claim",
                "status",
                "no_equilibrium is not a conclusion of this finite graph method",
            ));
        }
        for (consistent, path) in [
            (
                self.status_detail
                    .as_ref()
                    .is_some_and(|detail| !detail.trim().is_empty()),
                "status_detail",
            ),
            (self.catalogue == CatalogueClaim::NotApplicable, "catalogue"),
            (self.solutions.is_empty(), "solutions"),
            (self.diagnostics.is_empty(), "diagnostics"),
        ] {
            if !consistent {
                return Err(EnvelopeError::new(
                    "inconsistent_result_status",
                    path,
                    "failure needs a reason and has no solution or computed diagnostic payload",
                ));
            }
        }
        Ok(())
    }

    fn validate_diagnostics(&self) -> Result<(), EnvelopeError> {
        let mut definitions = BTreeSet::new();
        for (index, diagnostic) in self.diagnostics.iter().enumerate() {
            let path = format!("diagnostics.{index}");
            if diagnostic.definition_id.trim().is_empty() {
                return Err(EnvelopeError::new(
                    "empty_result_diagnostic_definition",
                    format!("{path}.definition_id"),
                    "a measurement needs an explicit definition reference",
                ));
            }
            if !definitions.insert(&diagnostic.definition_id) {
                return Err(EnvelopeError::new(
                    "duplicate_result_diagnostic",
                    format!("{path}.definition_id"),
                    &diagnostic.definition_id,
                ));
            }
            diagnostic.measurement.validate().map_err(|error| {
                EnvelopeError::new(
                    error.code(),
                    format!("{path}.measurement.{}", error.path()),
                    error.to_string(),
                )
            })?;
            // +0 and -0 mean the same numeric tolerance; neither is rewritten.
            if diagnostic
                .measurement
                .tolerance
                .partial_cmp(&self.tolerance)
                != Some(std::cmp::Ordering::Equal)
            {
                return Err(EnvelopeError::new(
                    "result_tolerance_mismatch",
                    format!("{path}.measurement.tolerance"),
                    "diagnostic tolerance differs from the envelope tolerance",
                ));
            }
        }
        Ok(())
    }
}

impl StaticPolicy {
    fn validate_against(&self, specification: &GameSpecification) -> Result<(), EnvelopeError> {
        let mut players = BTreeSet::new();
        for (node_id, node) in &specification.game.nodes {
            match node {
                Node::Decision { player, edges } => {
                    players.insert(player);
                    let action = self.choices.get(node_id).ok_or_else(|| {
                        EnvelopeError::new(
                            "missing_result_choice",
                            format!("solutions.0.choices.{}", node_id.0),
                            "a contingent policy needs a choice at every decision node",
                        )
                    })?;
                    if !edges.iter().any(|edge| &edge.action == action) {
                        return Err(EnvelopeError::new(
                            "invalid_result_action",
                            format!("solutions.0.choices.{}", node_id.0),
                            &action.0,
                        ));
                    }
                }
                Node::Terminal { payoffs } => players.extend(payoffs.keys()),
                Node::Chance { .. } => {}
            }
        }
        for node in self.choices.keys() {
            if !matches!(
                specification.game.nodes.get(node),
                Some(Node::Decision { .. })
            ) {
                return Err(EnvelopeError::new(
                    "invalid_result_choice_node",
                    format!("solutions.0.choices.{}", node.0),
                    "policy choices can reference only supplied decision nodes",
                ));
            }
        }
        for (player, value) in &self.expected_payoffs {
            let path = format!("solutions.0.expected_payoffs.{}", player.0);
            if !players.contains(player) {
                return Err(EnvelopeError::new(
                    "unknown_result_player",
                    path,
                    "player is absent from the supplied graph",
                ));
            }
            if !value.is_finite() {
                return Err(EnvelopeError::new(
                    "invalid_result_payoff",
                    path,
                    "expected a finite payoff; omitted entries retain sparse-zero semantics",
                ));
            }
        }
        Ok(())
    }
}
