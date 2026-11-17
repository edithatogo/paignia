//! Public solver emission controls for the declared legacy static profile.

use std::collections::BTreeMap;

use paignia_core::{
    ActionEdge, ActionId, CatalogueClaim, ChanceEdge, Game, GameInstanceId,
    GameSpecification, GameSpecificationId, Node, NodeId, PlayerId, ResultIdentityRecord,
    ResultSelectionRule, ResultStatus, SolverRunId, STATIC_GAME_PROFILE_V1,
};
use paignia_solve::{
    backward_induction_result, backward_induction_with_config, ResultSolveError, SolverConfig,
};

fn specification() -> GameSpecification {
    GameSpecification {
        schema_version: "1.0.0".into(),
        id: "urn:test:specification".into(),
        game: Game {
            root: NodeId("root".into()),
            nodes: BTreeMap::from([
                (NodeId("root".into()), Node::Decision {
                    player: PlayerId("p".into()),
                    edges: vec![
                        ActionEdge { action: ActionId("a".into()), target: NodeId("a".into()) },
                        ActionEdge { action: ActionId("b".into()), target: NodeId("b".into()) },
                    ],
                }),
                (NodeId("a".into()), Node::Terminal {
                    payoffs: BTreeMap::from([(PlayerId("p".into()), 2.0)]),
                }),
                (NodeId("b".into()), Node::Terminal {
                    payoffs: BTreeMap::new(),
                }),
            ]),
        },
    }
}

fn identities() -> ResultIdentityRecord {
    ResultIdentityRecord {
        schema_version: "1.0.0".into(),
        specification_id: GameSpecificationId("urn:test:specification".into()),
        game_instance_id: GameInstanceId("urn:test:instance".into()),
        solver_run_id: SolverRunId("urn:test:solver-run".into()),
        session_id: None, policy_artifact_id: None, solver_trace_id: None,
    }
}

#[test]
fn generated_record_is_admitted_and_preserves_the_native_result() {
    let source = specification();
    let expected = backward_induction_with_config(&source.game, SolverConfig::default()).unwrap();
    let result = backward_induction_result(&source, STATIC_GAME_PROFILE_V1, identities(), SolverConfig::default()).unwrap();
    assert_eq!(result.envelope.status, ResultStatus::Completed);
    assert_eq!(result.envelope.catalogue, CatalogueClaim::OneFound);
    assert_eq!(result.envelope.selection_rule, ResultSelectionRule::InputOrderPairwiseToleranceThenLexicographic);
    assert_eq!(result.envelope.solutions[0].choices, expected.choices);
    assert_eq!(result.envelope.solutions[0].expected_payoffs, expected.expected_payoffs);
    assert_eq!(result.diagnostic_trace, expected.trace);
    assert_eq!(result.solver_diagnostics, expected.diagnostics);
    assert!(result.envelope.diagnostics.is_empty());
    assert!(result.envelope.identities.session_id.is_none());
    result.envelope.validate_against(&source, &identities().game_instance_id).unwrap();
}

#[test]
fn unsupported_input_profile_fails_before_solving() {
    assert!(matches!(backward_induction_result(&specification(), "unknown", identities(), SolverConfig::default()), Err(ResultSolveError::Input(_))));
}

#[test]
fn invalid_identity_is_not_replaced_or_generated() {
    let mut ids = identities();
    ids.solver_run_id.0.clear();
    assert!(matches!(backward_induction_result(&specification(), STATIC_GAME_PROFILE_V1, ids, SolverConfig::default()), Err(ResultSolveError::Identity(_))));
}

#[test]
fn mismatched_specification_id_fails_before_solving() {
    let mut ids = identities();
    ids.specification_id.0 = "urn:other:specification".into();
    assert!(matches!(backward_induction_result(&specification(), STATIC_GAME_PROFILE_V1, ids, SolverConfig::default()), Err(ResultSolveError::SpecificationIdentityMismatch)));
}

#[test]
fn invalid_configuration_is_a_typed_failure_not_a_zero_result() {
    let config = SolverConfig { tolerance: f64::NAN, ..SolverConfig::default() };
    assert!(matches!(backward_induction_result(&specification(), STATIC_GAME_PROFILE_V1, identities(), config), Err(ResultSolveError::Solve(_))));
}

#[test]
fn finite_inputs_with_overflowed_expectations_do_not_emit_a_completed_record() {
    let mut source = specification();
    source.game = Game {
        root: NodeId("chance".into()),
        nodes: BTreeMap::from([
            (NodeId("chance".into()), Node::Chance { edges: vec![ChanceEdge {
                probability: 1.0 + 5e-10, target: NodeId("terminal".into()),
            }] }),
            (NodeId("terminal".into()), Node::Terminal {
                payoffs: BTreeMap::from([(PlayerId("p".into()), f64::MAX)]),
            }),
        ]),
    };
    source.validate_for_profile(STATIC_GAME_PROFILE_V1).unwrap();
    assert!(matches!(backward_induction_result(&source, STATIC_GAME_PROFILE_V1, identities(), SolverConfig::default()), Err(ResultSolveError::Output(_))));
}

#[test]
fn solver_budget_exhaustion_is_not_an_equilibrium_claim() {
    let config = SolverConfig { max_visited_nodes: 0, ..SolverConfig::default() };
    assert!(matches!(backward_induction_result(&specification(), STATIC_GAME_PROFILE_V1, identities(), config), Err(ResultSolveError::Solve(_))));
}

#[test]
fn signed_zero_tolerance_and_full_role_strings_survive_emission() {
    let config = SolverConfig { tolerance: -0.0, ..SolverConfig::default() };
    let mut ids = identities();
    ids.game_instance_id.0 = " https://example.test/ns/instance\0 ".into();
    let result = backward_induction_result(&specification(), STATIC_GAME_PROFILE_V1, ids.clone(), config).unwrap();
    assert_eq!(result.envelope.identities, ids);
    assert_eq!(result.envelope.tolerance.to_bits(), (-0.0_f64).to_bits());
}
