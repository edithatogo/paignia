#![deny(warnings)]
use std::collections::BTreeMap;
use std::error::Error;
use paignia_core::*;
use paignia_solve::*;

fn n(s: &str) -> NodeId { NodeId(s.into()) }
fn p(s: &str) -> PlayerId { PlayerId(s.into()) }
fn a(s: &str) -> ActionId { ActionId(s.into()) }
fn terminal(values: &[(&str, f64)]) -> Node {
    Node::Terminal { payoffs: values.iter().map(|(s,v)| (p(s),*v)).collect() }
}
fn decision(player: &str, edges: &[(&str, &str)]) -> Node {
    Node::Decision { player: p(player), edges: edges.iter().map(|(x,y)| ActionEdge { action:a(x),target:n(y) }).collect() }
}
fn source() -> GameSpecification {
    GameSpecification { schema_version:"1.0.0".into(),id:"urn:review:spec".into(),game:Game {
        root:n("root"),nodes:BTreeMap::from([
            (n("root"),decision("p",&[("good","good"),("other","q")])),
            (n("good"),terminal(&[("p",3.0)])),
            (n("q"),decision("q",&[("p_favored","p_leaf"),("q_favored","q_leaf")])),
            (n("p_leaf"),terminal(&[("p",10.0),("q",0.0)])),
            (n("q_leaf"),terminal(&[("p",1.0),("q",5.0)])),
        ])
    }}
}
fn ids() -> ResultIdentityRecord {
    ResultIdentityRecord { schema_version:"1.0.0".into(),specification_id:GameSpecificationId("urn:review:spec".into()),
        game_instance_id:GameInstanceId("urn:review:instance".into()),solver_run_id:SolverRunId("urn:review:run".into()),
        session_id:None,policy_artifact_id:None,solver_trace_id:None }
}
fn computed() -> BackwardInductionResult {
    backward_induction_result(&source(),STATIC_GAME_PROFILE_V1,ids(),SolverConfig::default()).unwrap()
}
fn check_error(value: &ResultEnvelope, expected: (&str,&str)) {
    let e=value.validate_against(&source(),&ids().game_instance_id).unwrap_err();
    assert_eq!((e.code,e.path.as_str()),expected);
}

#[test]
fn independently_derived_contingent_policy_includes_unchosen_decision() {
    let value=computed();
    assert_eq!(value.envelope.solutions.len(),1);
    assert_eq!(value.envelope.solutions[0].choices,BTreeMap::from([(n("root"),a("good")),(n("q"),a("q_favored"))]));
    assert_eq!(value.envelope.solutions[0].expected_payoffs,BTreeMap::from([(p("p"),3.0)]));
    assert_eq!(value.diagnostic_trace.len(),5);
    assert!(value.envelope.diagnostics.is_empty());
    assert_eq!(value.envelope.status,ResultStatus::Completed);
    assert_eq!(value.envelope.catalogue,CatalogueClaim::OneFound);
    assert!(value.envelope.identities.session_id.is_none());
}

#[test]
fn explicit_legacy_tie_version_permits_the_documented_order_effect() {
    let mut s=source();
    s.game.nodes=BTreeMap::from([
        (n("root"),decision("p",&[("a","a"),("b","b"),("c","c")])),
        (n("a"),terminal(&[("p",0.0)])),(n("b"),terminal(&[("p",0.75)])),(n("c"),terminal(&[("p",1.5)])),
    ]);
    let config=SolverConfig{tolerance:1.0,..SolverConfig::default()};
    let first=backward_induction_result(&s,STATIC_GAME_PROFILE_V1,ids(),config).unwrap();
    assert_eq!(first.envelope.solutions[0].choices[&n("root")],a("c"));
    s.game.nodes.insert(n("root"),decision("p",&[("c","c"),("b","b"),("a","a")]));
    let second=backward_induction_result(&s,STATIC_GAME_PROFILE_V1,ids(),config).unwrap();
    assert_eq!(second.envelope.solutions[0].choices[&n("root")],a("a"));
    for value in [first,second] {
        assert_eq!(value.envelope.selection_rule,ResultSelectionRule::InputOrderPairwiseToleranceThenLexicographic);
        assert_eq!(value.envelope.algorithm.version,"1.0.0");
        assert_eq!(value.envelope.solution_concept,ResultSolutionConcept::ToleranceAwareBackwardInductionPolicy);
    }
}

#[test]
fn outer_source_and_claim_failure_precedence_is_deterministic() {
    let mut value=computed().envelope;
    value.schema_version="2.0.0".into();value.semantic_profile="unknown".into();value.identities.schema_version="future".into();
    value.identities.specification_id.0="other".into();value.game_class=ResultGameClass::FiniteNormalForm;value.tolerance=f64::NAN;
    check_error(&value,("unsupported_result_schema","schema_version"));
    value.schema_version="1.0.0".into();check_error(&value,("unsupported_semantic_profile","semantic_profile"));
    value.semantic_profile=STATIC_GAME_PROFILE_V1.into();check_error(&value,("unsupported_result_identity_schema","identities.schema_version"));
    value.identities.schema_version="1.0.0".into();check_error(&value,("result_source_mismatch","identities.specification_id"));
    value.identities.specification_id=ids().specification_id;check_error(&value,("unsupported_result_claim","game_class"));
    value.game_class=ResultGameClass::FiniteAcyclicExtensive;check_error(&value,("invalid_result_tolerance","tolerance"));
    let mut invalid=source();invalid.game.root=n("absent");
    let error=value.validate_against(&invalid,&GameInstanceId("other".into())).unwrap_err();
    assert_eq!((error.code,error.path.as_str()),("missing_root","specification.game.nodes.absent"));
}

#[test]
fn status_and_catalogue_matrix_cannot_promote_one_policy() {
    for status in [ResultStatus::Unsupported,ResultStatus::NotConverged] {
        let mut value=computed().envelope;value.status=status;value.status_detail=Some("explicit failure".into());
        value.catalogue=CatalogueClaim::NotApplicable;
        check_error(&value,("inconsistent_result_status","solutions"));
        value.solutions.clear();assert!(value.validate_against(&source(),&ids().game_instance_id).is_ok());
        value.status=ResultStatus::NoEquilibrium;check_error(&value,("unsupported_result_claim","status"));
    }
    for catalogue in [CatalogueClaim::Approximate,CatalogueClaim::NotApplicable,
        CatalogueClaim::EnumeratedWithinDeclaredScope{scope_id:"whole-game".into(),completeness_evidence_id:None},
        CatalogueClaim::EnumeratedWithinDeclaredScope{scope_id:"whole-game".into(),completeness_evidence_id:Some("asserted-only".into())}] {
        let mut value=computed().envelope;value.catalogue=catalogue;
        check_error(&value,("unsupported_result_catalogue","catalogue"));
    }
}

#[test]
fn integer_bit_classification_agrees_with_tolerance_admission() {
    let bits=[0,1,0x8000_0000_0000_0000,0x8000_0000_0000_0001,0x0010_0000_0000_0000,
        0x7fef_ffff_ffff_ffff,0xffef_ffff_ffff_ffff,0x7ff0_0000_0000_0000,0xfff0_0000_0000_0000,
        0x7ff0_0000_0000_0001,0x7ff8_0000_0000_0000,0xfff8_0000_0000_0001];
    for raw in bits {
        let finite=(raw & 0x7ff0_0000_0000_0000)!=0x7ff0_0000_0000_0000;
        let nonnegative=(raw>>63)==0 || (raw & 0x7fff_ffff_ffff_ffff)==0;
        let mut value=computed().envelope;value.tolerance=f64::from_bits(raw);
        assert_eq!(value.validate_against(&source(),&ids().game_instance_id).is_ok(),finite&&nonnegative,"{raw:016x}");
        assert_eq!(value.tolerance.to_bits(),raw);
    }
}

#[test]
fn diagnostics_require_numeric_tolerance_equality_but_certify_no_convergence() {
    let mut value=computed().envelope;value.tolerance=-0.0;
    value.diagnostics=vec![DeclaredDiagnostic {definition_id:"urn:review:metric".into(),measurement:ResultDiagnosticRecord{
        schema_version:"1.0.0".into(),kind:ResultDiagnosticKind::Exploitability,value:100.0,tolerance:0.0}}];
    assert!(value.validate_against(&source(),&ids().game_instance_id).is_ok());
    assert_eq!(value.tolerance.to_bits(),0x8000_0000_0000_0000);
    assert_eq!(value.diagnostics[0].measurement.tolerance.to_bits(),0);
    value.diagnostics[0].measurement.tolerance=f64::from_bits(1);
    check_error(&value,("result_tolerance_mismatch","diagnostics.0.measurement.tolerance"));
}

#[test]
fn input_identity_and_solver_errors_preserve_typed_source_chains() {
    let input=backward_induction_result(&source(),"bad-profile",ids(),SolverConfig::default()).unwrap_err();
    assert!(input.source().unwrap().downcast_ref::<ProfileValidationError>().is_some());
    let mut invalid=ids();invalid.solver_run_id.0.clear();
    let identity=backward_induction_result(&source(),STATIC_GAME_PROFILE_V1,invalid,SolverConfig::default()).unwrap_err();
    assert!(identity.source().unwrap().downcast_ref::<ResultIdentityValidationError>().is_some());
    let bad=f64::from_bits(0xfff8_0000_0000_0123);
    let solver=backward_induction_result(&source(),STATIC_GAME_PROFILE_V1,ids(),SolverConfig{tolerance:bad,..SolverConfig::default()}).unwrap_err();
    assert!(matches!(solver.source().unwrap().downcast_ref::<SolveError>(),Some(SolveError::InvalidTolerance(v)) if v.to_bits()==bad.to_bits()));
}

#[test]
fn nonfinite_unchosen_player_branch_never_emits_completed_output() {
    let mut s=source();
    s.game.nodes.insert(n("p_leaf"),Node::Chance{edges:vec![ChanceEdge{probability:1.0+5e-10,target:n("huge")} ]});
    s.game.nodes.insert(n("huge"),terminal(&[("unused",f64::MAX)]));
    let result=backward_induction_result(&s,STATIC_GAME_PROFILE_V1,ids(),SolverConfig::default()).unwrap_err();
    let output=result.source().unwrap().downcast_ref::<ResultEnvelopeError>().unwrap();
    assert_eq!(output.code,"invalid_result_payoff");
    assert!(output.path.starts_with("diagnostic_trace."));
    assert!(output.path.ends_with(".expected_payoffs.unused"));
}

#[test]
fn roles_namespaces_and_terminal_numeric_bits_are_retained() {
    let mut s=source();s.id=" https://example.test/spec#id\0 ".into();
    s.game.root=n("terminal");
    for payoff in [f64::from_bits(1),-f64::from_bits(1),-0.0,-f64::MAX,f64::MAX] {
        s.game.nodes=BTreeMap::from([(n("terminal"),terminal(&[("https://example.test/player#p",payoff)]))]);
        let mut id=ids();id.specification_id=GameSpecificationId(s.id.clone());
        id.session_id=Some(GameSessionId("same".into()));id.policy_artifact_id=Some(PolicyArtifactId("same".into()));
        id.solver_trace_id=Some(SolverDiagnosticTraceId("same".into()));
        let result=backward_induction_result(&s,STATIC_GAME_PROFILE_V1,id.clone(),SolverConfig::default()).unwrap();
        assert_eq!(result.envelope.identities,id);
        assert_eq!(result.envelope.solutions[0].expected_payoffs[&p("https://example.test/player#p")].to_bits(),payoff.to_bits());
        assert!(result.envelope.solutions[0].choices.is_empty());
    }
}
