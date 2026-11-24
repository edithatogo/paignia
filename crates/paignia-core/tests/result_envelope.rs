//! Synthetic public-contract controls; not numerical solver qualification.

use std::collections::BTreeMap;

use paignia_core::{
    ActionEdge, ActionId, Game, GameInstanceId, GameSpecification, GameSpecificationId, Node,
    NodeId, PlayerId, ResultDiagnosticKind, ResultDiagnosticRecord, ResultIdentityRecord,
    SolverRunId,
};
use paignia_core::{
    AlgorithmRecord, CatalogueClaim, DeclaredDiagnostic, ResultAlgorithm, ResultEnvelope,
    ResultGameClass, ResultInformationStructure, ResultSelectionRule, ResultSolutionConcept,
    ResultStatus, StaticPolicyResult, BACKWARD_INDUCTION_ALGORITHM_VERSION,
    RESULT_ENVELOPE_SCHEMA_VERSION,
};

fn source() -> GameSpecification {
    GameSpecification {
        schema_version: "1.0.0".into(),
        id: "https://example.test/spec/game".into(),
        game: Game {
            root: NodeId("root".into()),
            nodes: BTreeMap::from([
                (
                    NodeId("root".into()),
                    Node::Decision {
                        player: PlayerId("p".into()),
                        edges: vec![
                            ActionEdge {
                                action: ActionId("a".into()),
                                target: NodeId("left".into()),
                            },
                            ActionEdge {
                                action: ActionId("b".into()),
                                target: NodeId("right".into()),
                            },
                        ],
                    },
                ),
                (
                    NodeId("left".into()),
                    Node::Terminal {
                        payoffs: BTreeMap::from([(PlayerId("p".into()), 2.0)]),
                    },
                ),
                (
                    NodeId("right".into()),
                    Node::Terminal {
                        payoffs: BTreeMap::new(),
                    },
                ),
            ]),
        },
    }
}

fn instance() -> GameInstanceId {
    GameInstanceId("urn:example:instance:1".into())
}

fn record() -> ResultEnvelope {
    ResultEnvelope {
        schema_version: "1.0.0".into(),
        semantic_profile: "paignia/static-finite-perfect-information-sparse-zero/1.0.0".into(),
        identities: ResultIdentityRecord {
            schema_version: "1.0.0".into(),
            specification_id: GameSpecificationId(source().id),
            game_instance_id: instance(),
            solver_run_id: SolverRunId("urn:example:run:1".into()),
            session_id: None,
            policy_artifact_id: None,
            solver_trace_id: None,
        },
        game_class: ResultGameClass::FiniteAcyclicExtensive,
        information_structure: ResultInformationStructure::PerfectInformation,
        solution_concept: ResultSolutionConcept::ToleranceAwareBackwardInductionPolicy,
        algorithm: AlgorithmRecord {
            algorithm: ResultAlgorithm::BackwardInduction,
            version: "1.0.0".into(),
        },
        status: ResultStatus::Completed,
        status_detail: None,
        tolerance: 1e-12,
        selection_rule: ResultSelectionRule::InputOrderPairwiseToleranceThenLexicographic,
        catalogue: CatalogueClaim::OneFound,
        diagnostics: Vec::new(),
        solutions: vec![StaticPolicyResult {
            choices: BTreeMap::from([(NodeId("root".into()), ActionId("a".into()))]),
            expected_payoffs: BTreeMap::from([(PlayerId("p".into()), 2.0)]),
        }],
    }
}

fn rejects(value: &ResultEnvelope, code: &str, path: &str) {
    let error = value.validate_against(&source(), &instance()).unwrap_err();
    assert_eq!(error.code, code);
    assert_eq!(error.path, path);
}

fn failed(status: ResultStatus) -> ResultEnvelope {
    let mut value = record();
    value.status = status;
    value.status_detail = Some("explicit inability to produce a completed result".into());
    value.catalogue = CatalogueClaim::NotApplicable;
    value.solutions.clear();
    value
}

fn diagnostic() -> DeclaredDiagnostic {
    DeclaredDiagnostic {
        definition_id: "urn:example:diagnostic:maximum-local-deviation/1".into(),
        measurement: ResultDiagnosticRecord {
            schema_version: "1.0.0".into(),
            kind: ResultDiagnosticKind::Residual,
            value: 0.0,
            tolerance: 1e-12,
        },
    }
}

#[test]
fn exact_versions_and_current_static_policy_are_admitted() {
    assert_eq!(RESULT_ENVELOPE_SCHEMA_VERSION, "1.0.0");
    assert_eq!(BACKWARD_INDUCTION_ALGORITHM_VERSION, "1.0.1");
    for version in ["1.0.0", "1.0.1"] {
        let mut value = record();
        value.algorithm.version = version.into();
        let before = value.clone();
        assert_eq!(value.validate_against(&source(), &instance()), Ok(()));
        assert_eq!(value, before);
    }
}

#[test]
fn unknown_envelope_schema_precedes_other_errors() {
    for schema in ["", "1.0.1", " 1.0.0", "1.0.0\n", "2.0.0"] {
        let mut value = record();
        value.schema_version = schema.into();
        value.semantic_profile.clear();
        value.tolerance = f64::NAN;
        rejects(&value, "unsupported_result_schema", "schema_version");
    }
}

#[test]
fn unknown_profiles_are_never_normalized() {
    for profile in [
        "",
        "1.0.0",
        "paignia/static-finite-perfect-information-sparse-zero/2.0.0",
    ] {
        let mut value = record();
        value.semantic_profile = profile.into();
        rejects(&value, "unsupported_semantic_profile", "semantic_profile");
    }
}

#[test]
fn nested_identity_version_and_blank_identifiers_are_checked() {
    let mut value = record();
    value.identities.schema_version = "2.0.0".into();
    rejects(
        &value,
        "unsupported_result_identity_schema",
        "identities.schema_version",
    );
    value.identities.schema_version = "1.0.0".into();
    value.identities.solver_run_id.0 = "\t".into();
    rejects(&value, "empty_result_identity", "identities.solver_run_id");
}

#[test]
fn specification_identity_must_match_the_supplied_source() {
    let mut value = record();
    value.identities.specification_id.0 = "https://different.test/spec/game".into();
    rejects(
        &value,
        "result_source_mismatch",
        "identities.specification_id",
    );
}

#[test]
fn explicit_instance_binding_is_required() {
    let error = record()
        .validate_against(&source(), &GameInstanceId("urn:example:other".into()))
        .unwrap_err();
    assert_eq!(error.code, "result_source_mismatch");
    assert_eq!(error.path, "identities.game_instance_id");
}

#[test]
fn invalid_source_graph_does_not_receive_a_result_receipt() {
    let mut specification = source();
    specification.game.root = NodeId("missing".into());
    let error = record()
        .validate_against(&specification, &instance())
        .unwrap_err();
    assert_eq!(error.code, "missing_root");
    assert!(error.path.starts_with("specification."));
}

#[test]
fn another_game_class_is_not_admitted_as_the_static_graph_profile() {
    let mut value = record();
    value.game_class = ResultGameClass::FiniteNormalForm;
    rejects(&value, "unsupported_result_claim", "game_class");
}

#[test]
fn unsupported_information_structures_reject() {
    for information in [
        ResultInformationStructure::ImperfectInformation,
        ResultInformationStructure::Simultaneous,
    ] {
        let mut value = record();
        value.information_structure = information;
        rejects(&value, "unsupported_result_claim", "information_structure");
    }
}

#[test]
fn a_policy_cannot_be_relabelled_as_nash_or_correlated_equilibrium() {
    for concept in [
        ResultSolutionConcept::NashEquilibrium,
        ResultSolutionConcept::CorrelatedEquilibrium,
    ] {
        let mut value = record();
        value.solution_concept = concept;
        rejects(&value, "unsupported_result_claim", "solution_concept");
    }
}

#[test]
fn method_and_method_version_are_checked_independently() {
    let mut value = record();
    value.algorithm.algorithm = ResultAlgorithm::PureNashEnumeration;
    rejects(&value, "unsupported_result_claim", "algorithm.algorithm");
    value.algorithm.algorithm = ResultAlgorithm::BackwardInduction;
    for version in ["", "0.1.0", "1.0.2", "1.0.1+unqualified", "2.0.0"] {
        value.algorithm.version = version.into();
        rejects(
            &value,
            "unsupported_result_algorithm_version",
            "algorithm.version",
        );
    }
}

#[test]
fn enum_parsing_rejects_unknown_or_normalized_literals() {
    assert_eq!(
        "paignia/backward-induction".parse(),
        Ok(ResultAlgorithm::BackwardInduction)
    );
    for value in [
        "BackwardInduction",
        "paignia/backward-induction ",
        "paignia/cfr",
        "",
    ] {
        assert_eq!(
            value.parse::<ResultAlgorithm>().unwrap_err().code,
            "unsupported_result_algorithm"
        );
    }
    assert_eq!(
        "tolerance-aware-backward-induction-policy".parse(),
        Ok(ResultSolutionConcept::ToleranceAwareBackwardInductionPolicy)
    );
    for value in [
        "Nash",
        "subgame-perfect-equilibrium",
        "nash-equilibrium\n",
        "",
    ] {
        assert_eq!(
            value.parse::<ResultSolutionConcept>().unwrap_err().code,
            "unsupported_result_solution_concept"
        );
    }
}

#[test]
fn unspecified_selection_does_not_acquire_a_default() {
    let mut value = record();
    value.selection_rule = ResultSelectionRule::Unspecified;
    rejects(&value, "unsupported_result_claim", "selection_rule");
}

#[test]
fn unsupported_and_nonconverged_are_empty_failures_not_zero_payoffs() {
    for status in [ResultStatus::Unsupported, ResultStatus::NotConverged] {
        let mut value = failed(status);
        assert_eq!(value.validate_against(&source(), &instance()), Ok(()));
        value.solutions = record().solutions;
        rejects(&value, "inconsistent_result_status", "solutions");
    }
}

#[test]
fn failure_requires_an_explicit_nonblank_reason() {
    for detail in [None, Some(String::new()), Some("\n \t".into())] {
        let mut value = failed(ResultStatus::Unsupported);
        value.status_detail = detail;
        rejects(&value, "inconsistent_result_status", "status_detail");
    }
}

#[test]
fn failure_cannot_claim_a_solution_catalogue_or_computed_diagnostic() {
    let mut value = failed(ResultStatus::NotConverged);
    value.catalogue = CatalogueClaim::OneFound;
    rejects(&value, "inconsistent_result_status", "catalogue");
    value.catalogue = CatalogueClaim::NotApplicable;
    value.diagnostics.push(diagnostic());
    rejects(&value, "inconsistent_result_status", "diagnostics");
}

#[test]
fn no_equilibrium_is_not_an_admitted_conclusion_for_this_profile() {
    let value = failed(ResultStatus::NoEquilibrium);
    rejects(&value, "unsupported_result_claim", "status");
}

#[test]
fn completed_policy_requires_exactly_one_solution() {
    let mut value = record();
    value.solutions.clear();
    rejects(&value, "invalid_result_catalogue", "solutions");
    value.solutions = vec![record().solutions[0].clone(); 2];
    rejects(&value, "invalid_result_catalogue", "solutions");
}

#[test]
fn one_policy_cannot_be_promoted_to_an_exhaustive_catalogue() {
    for evidence in [None, Some("urn:example:unverified-certificate".into())] {
        let mut value = record();
        value.catalogue = CatalogueClaim::EnumeratedWithinDeclaredScope {
            scope_id: "all-equilibria".into(),
            completeness_evidence_id: evidence,
        };
        rejects(&value, "unsupported_result_catalogue", "catalogue");
    }
}

#[test]
fn approximate_and_inapplicable_catalogues_are_not_completed_policy_catalogues() {
    for catalogue in [CatalogueClaim::Approximate, CatalogueClaim::NotApplicable] {
        let mut value = record();
        value.catalogue = catalogue;
        rejects(&value, "unsupported_result_catalogue", "catalogue");
    }
}

#[test]
fn invalid_tolerances_reject_without_clipping_or_defaulting() {
    for tolerance in [
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        -1.0,
        -f64::from_bits(1),
    ] {
        let mut value = record();
        value.tolerance = tolerance;
        rejects(&value, "invalid_result_tolerance", "tolerance");
    }
}

#[test]
fn finite_tolerance_boundaries_and_signed_zero_are_preserved() {
    for tolerance in [0.0, -0.0, f64::from_bits(1), f64::MIN_POSITIVE, f64::MAX] {
        let mut value = record();
        value.tolerance = tolerance;
        assert_eq!(value.validate_against(&source(), &instance()), Ok(()));
        assert_eq!(value.tolerance.to_bits(), tolerance.to_bits());
    }
}

#[test]
fn diagnostic_components_are_validated_and_need_a_definition() {
    let mut value = record();
    value.diagnostics.push(diagnostic());
    assert_eq!(value.validate_against(&source(), &instance()), Ok(()));
    value.diagnostics[0].definition_id = " ".into();
    rejects(
        &value,
        "empty_result_diagnostic_definition",
        "diagnostics.0.definition_id",
    );
    value.diagnostics[0] = diagnostic();
    value.diagnostics[0].measurement.value = f64::NAN;
    rejects(
        &value,
        "non_finite_result_diagnostic",
        "diagnostics.0.measurement.value",
    );
}

#[test]
fn each_named_diagnostic_is_unique_and_uses_the_declared_tolerance() {
    let mut value = record();
    value.diagnostics = vec![diagnostic(), diagnostic()];
    rejects(
        &value,
        "duplicate_result_diagnostic",
        "diagnostics.1.definition_id",
    );
    value.diagnostics.pop();
    value.diagnostics[0].measurement.tolerance = 1e-9;
    rejects(
        &value,
        "result_tolerance_mismatch",
        "diagnostics.0.measurement.tolerance",
    );
}

#[test]
fn named_diagnostics_are_not_a_convergence_certificate() {
    let mut value = record();
    value.diagnostics.push(diagnostic());
    value.diagnostics[0].measurement.value = 2.0;
    assert_eq!(value.validate_against(&source(), &instance()), Ok(()));
    assert!(value.diagnostics[0].measurement.value > value.tolerance);
}

#[test]
fn nonfinite_or_unknown_player_payoffs_reject() {
    for payoff in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut value = record();
        value.solutions[0]
            .expected_payoffs
            .insert(PlayerId("p".into()), payoff);
        rejects(
            &value,
            "invalid_result_payoff",
            "solutions.0.expected_payoffs.p",
        );
    }
    let mut value = record();
    value.solutions[0]
        .expected_payoffs
        .insert(PlayerId("other".into()), 0.0);
    rejects(
        &value,
        "unknown_result_player",
        "solutions.0.expected_payoffs.other",
    );
}

#[test]
fn sparse_payoffs_and_nonoptimal_legal_choices_remain_structural_only() {
    let mut value = record();
    value.solutions[0].expected_payoffs.clear();
    value.solutions[0]
        .choices
        .insert(NodeId("root".into()), ActionId("b".into()));
    assert_eq!(value.validate_against(&source(), &instance()), Ok(()));
}

#[test]
fn every_decision_requires_one_legal_choice() {
    let mut value = record();
    value.solutions[0].choices.clear();
    rejects(&value, "missing_result_choice", "solutions.0.choices.root");
    value.solutions[0]
        .choices
        .insert(NodeId("root".into()), ActionId("unknown".into()));
    rejects(&value, "invalid_result_action", "solutions.0.choices.root");
}

#[test]
fn terminal_and_unknown_nodes_cannot_be_policy_choices() {
    for node in ["left", "unknown"] {
        let mut value = record();
        value.solutions[0]
            .choices
            .insert(NodeId(node.into()), ActionId("a".into()));
        rejects(
            &value,
            "invalid_result_choice_node",
            &format!("solutions.0.choices.{node}"),
        );
    }
}

#[test]
fn terminal_only_game_needs_no_manufactured_choice_or_player() {
    let mut specification = source();
    specification.game = Game {
        root: NodeId("end".into()),
        nodes: BTreeMap::from([(
            NodeId("end".into()),
            Node::Terminal {
                payoffs: BTreeMap::new(),
            },
        )]),
    };
    let mut value = record();
    value.solutions[0].choices.clear();
    value.solutions[0].expected_payoffs.clear();
    assert_eq!(value.validate_against(&specification, &instance()), Ok(()));
}

#[test]
fn any_public_mutation_requires_readmission() {
    let mut value = record();
    assert_eq!(value.validate_against(&source(), &instance()), Ok(()));
    value.identities.game_instance_id.0.push('x');
    rejects(
        &value,
        "result_source_mismatch",
        "identities.game_instance_id",
    );
}
