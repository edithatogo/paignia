//! Opt-in production of the versioned static result envelope.

use paignia_core::{
    AlgorithmRecord, CatalogueClaim, GameSpecification, ProfileValidationError, ResultAlgorithm,
    ResultEnvelope, ResultEnvelopeError, ResultGameClass, ResultIdentityRecord,
    ResultIdentityValidationError, ResultInformationStructure, ResultSelectionRule,
    ResultSolutionConcept, ResultStatus, StaticPolicyResult, BACKWARD_INDUCTION_ALGORITHM_VERSION,
    RESULT_ENVELOPE_SCHEMA_VERSION,
};
use thiserror::Error;

use crate::{
    backward_induction_with_config, SolveError, SolverConfig, SolverDiagnostics, TraceStep,
};

/// One computed policy and its separate hypothetical solver traversal.
///
/// `diagnostic_trace` is not a realized event stream. Missing caller context
/// remains absent, and no residual or convergence statistic is fabricated.
#[derive(Clone, Debug, PartialEq)]
pub struct ComputedPolicy {
    pub envelope: ResultEnvelope,
    pub diagnostic_trace: Vec<TraceStep>,
    pub solver_diagnostics: SolverDiagnostics,
}

/// Failure of input admission, native computation, or generated-record admission.
///
/// Legacy [`SolveError`] variants remain unchanged. Failure is never replaced
/// with an economic zero, a completed envelope or a no-equilibrium claim.
#[derive(Clone, Debug, PartialEq, Error)]
pub enum EmissionError {
    #[error("invalid semantic input: {0}")]
    Input(#[source] ProfileValidationError),
    #[error("invalid result identity: {0}")]
    Identity(#[source] ResultIdentityValidationError),
    #[error("result specification identity differs from the supplied specification")]
    SpecificationIdentityMismatch,
    #[error("native solver failed: {0}")]
    Solve(#[source] SolveError),
    #[error("generated result is outside the admitted output contract: {0}")]
    Output(#[source] ResultEnvelopeError),
}

/// Compute and admit a versioned result for the explicit static input profile.
///
/// This calls the existing Rust backward-induction implementation exactly once.
/// It preserves that algorithm's input-order pairwise-tolerance selection;
/// neither exact SPE optimality nor reordered-input invariance is asserted.
/// The caller supplies instance/run/context IDs; source identity is compared to
/// `specification.id`. This is explicit binding, not historical attestation.
///
/// # Errors
///
/// Returns [`EmissionError`] for an unsupported/invalid input, invalid or
/// mismatched identities, failed computation, or a structurally invalid computed
/// record, including non-finite aggregate payoffs from otherwise finite inputs.
pub fn solve_specification(
    specification: &GameSpecification,
    semantic_profile: &str,
    identities: ResultIdentityRecord,
    config: SolverConfig,
) -> Result<ComputedPolicy, EmissionError> {
    specification
        .validate_for_profile(semantic_profile)
        .map_err(EmissionError::Input)?;
    identities.validate().map_err(EmissionError::Identity)?;
    if identities.specification_id.0 != specification.id {
        return Err(EmissionError::SpecificationIdentityMismatch);
    }
    let solution = backward_induction_with_config(&specification.game, config)
        .map_err(EmissionError::Solve)?;
    for step in &solution.trace {
        for (player, payoff) in &step.expected_payoffs {
            if !payoff.is_finite() {
                return Err(EmissionError::Output(ResultEnvelopeError {
                    code: "invalid_result_payoff",
                    path: format!(
                        "diagnostic_trace.{}.expected_payoffs.{}",
                        step.sequence, player.0
                    ),
                    detail: "a visited hypothetical branch produced a non-finite payoff".into(),
                }));
            }
        }
    }
    let envelope = ResultEnvelope {
        schema_version: RESULT_ENVELOPE_SCHEMA_VERSION.into(),
        semantic_profile: semantic_profile.into(),
        identities,
        game_class: ResultGameClass::FiniteAcyclicExtensive,
        information_structure: ResultInformationStructure::PerfectInformation,
        solution_concept: ResultSolutionConcept::ToleranceAwareBackwardInductionPolicy,
        algorithm: AlgorithmRecord {
            algorithm: ResultAlgorithm::BackwardInduction,
            version: BACKWARD_INDUCTION_ALGORITHM_VERSION.into(),
        },
        status: ResultStatus::Completed,
        status_detail: None,
        tolerance: solution.tolerance,
        selection_rule: ResultSelectionRule::InputOrderPairwiseToleranceThenLexicographic,
        catalogue: CatalogueClaim::OneFound,
        diagnostics: Vec::new(),
        solutions: vec![StaticPolicyResult {
            choices: solution.choices,
            expected_payoffs: solution.expected_payoffs,
        }],
    };
    envelope
        .validate_against(specification, &envelope.identities.game_instance_id)
        .map_err(EmissionError::Output)?;
    Ok(ComputedPolicy {
        envelope,
        diagnostic_trace: solution.trace,
        solver_diagnostics: solution.diagnostics,
    })
}
