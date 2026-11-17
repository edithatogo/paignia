//! Independent native solver and identity controls for static conversion.
use paignia_core::*;
use paignia_solve::{backward_induction_with_config, SolverConfig};
use std::collections::BTreeMap;

type Role = StaticIdentityRole;
const CONTEXT: &str = " https://example.test/#same\0 ";

fn n(id: &str) -> NodeId {
    NodeId(id.into())
}
fn p(id: &str) -> PlayerId {
    PlayerId(id.into())
}
fn a(id: &str) -> ActionId {
    ActionId(id.into())
}
fn decision(player: &str, edges: &[(&str, &str)]) -> Node {
    Node::Decision {
        player: p(player),
        edges: edges
            .iter()
            .map(|(action, target)| ActionEdge {
                action: a(action),
                target: n(target),
            })
            .collect(),
    }
}
fn terminal(values: &[(&str, f64)]) -> Node {
    Node::Terminal {
        payoffs: values
            .iter()
            .map(|(player, value)| (p(player), *value))
            .collect(),
    }
}
fn record(game: Game) -> StaticGameRecord {
    StaticGameRecord {
        schema_version: "1.0.0".into(),
        semantic_profile: STATIC_GAME_PROFILE_V1.into(),
        information_structure: ResultInformationStructure::PerfectInformation,
        chance_rule: StaticChanceRule::AllowZeroAndValidateEveryBranch,
        payoff_rule: StaticPayoffRule::IntentionalSparseZero,
        selection_rule: ResultSelectionRule::InputOrderPairwiseToleranceThenLexicographic,
        selection_tolerance: 1.0,
        specification: GameSpecification {
            schema_version: "1.0.0".into(),
            id: CONTEXT.into(),
            game,
        },
        instance_id: GameInstanceId(CONTEXT.into()),
        session_id: Some(GameSessionId(CONTEXT.into())),
        policy_artifact_id: Some(PolicyArtifactId(CONTEXT.into())),
        solver_run_id: Some(SolverRunId(CONTEXT.into())),
        solver_trace_id: Some(SolverDiagnosticTraceId(CONTEXT.into())),
    }
}
fn context_map(extra: &[(Role, &str, &str)]) -> StaticIdentityMap {
    let mut bindings: Vec<_> = [
        Role::Specification,
        Role::Instance,
        Role::Session,
        Role::PolicyArtifact,
        Role::SolverRun,
        Role::SolverDiagnosticTrace,
    ]
    .into_iter()
    .map(|role| StaticIdentityBinding {
        role,
        source: CONTEXT.into(),
        target: "record".into(),
    })
    .collect();
    bindings.extend(
        extra
            .iter()
            .map(|(role, source, target)| StaticIdentityBinding {
                role: *role,
                source: (*source).into(),
                target: (*target).into(),
            }),
    );
    StaticIdentityMap {
        schema_version: "1.0.0".into(),
        bindings,
    }
}
fn mapped(mapping: &StaticIdentityMap, role: Role, source: &str) -> String {
    mapping
        .bindings
        .iter()
        .find(|b| b.role == role && b.source == source)
        .unwrap()
        .target
        .clone()
}
fn assert_solver_invariant(source: &StaticGameRecord, conversion: &StaticGameConversion) {
    let cfg = SolverConfig {
        tolerance: source.selection_tolerance,
        ..SolverConfig::default()
    };
    let original = backward_induction_with_config(&source.specification.game, cfg).unwrap();
    let converted =
        backward_induction_with_config(&conversion.record.specification.game, cfg).unwrap();
    let choices: BTreeMap<_, _> = original
        .choices
        .iter()
        .map(|(node, action)| {
            (
                n(&mapped(&conversion.mapping, Role::Node, &node.0)),
                a(&mapped(&conversion.mapping, Role::Action, &action.0)),
            )
        })
        .collect();
    assert_eq!(converted.choices, choices);
    let payoffs: BTreeMap<_, _> = original
        .expected_payoffs
        .iter()
        .map(|(player, value)| {
            (
                p(&mapped(&conversion.mapping, Role::Player, &player.0)),
                value.to_bits(),
            )
        })
        .collect();
    assert_eq!(
        converted
            .expected_payoffs
            .iter()
            .map(|(player, value)| (player.clone(), value.to_bits()))
            .collect::<BTreeMap<_, _>>(),
        payoffs
    );
}
fn chain(order: &[&str]) -> (StaticGameRecord, StaticIdentityMap) {
    let source = record(Game {
        root: n("root"),
        nodes: BTreeMap::from([
            (
                n("root"),
                decision("p", &order.iter().map(|id| (*id, *id)).collect::<Vec<_>>()),
            ),
            (n("a"), terminal(&[("p", 0.0)])),
            (n("b"), terminal(&[("p", 0.75)])),
            (n("c"), terminal(&[("p", 1.5)])),
        ]),
    });
    let map = context_map(&[
        (Role::Node, "root", "z-root"),
        (Role::Node, "a", "3"),
        (Role::Node, "b", "2"),
        (Role::Node, "c", "1"),
        (Role::Player, "p", "mapped-player"),
        (Role::Action, "a", "0"),
        (Role::Action, "b", "β"),
        (Role::Action, "c", "😀"),
    ]);
    (source, map)
}

#[test]
fn every_near_tie_edge_order_preserves_its_actual_legacy_policy() {
    // Derived manually from the pairwise tolerance rule, not from the converter.
    let cases = [
        (["a", "b", "c"], "c"),
        (["a", "c", "b"], "b"),
        (["b", "a", "c"], "c"),
        (["b", "c", "a"], "a"),
        (["c", "a", "b"], "b"),
        (["c", "b", "a"], "a"),
    ];
    for (order, expected) in cases {
        let (source, map) = chain(&order);
        let result = backward_induction_with_config(
            &source.specification.game,
            SolverConfig {
                tolerance: 1.0,
                ..SolverConfig::default()
            },
        )
        .unwrap();
        assert_eq!(result.choices[&n("root")], a(expected));
        let converted = convert_static_game(&source, &map, STATIC_GAME_PROFILE_V1).unwrap();
        assert_solver_invariant(&source, &converted);
        assert_eq!(restore_static_game(&converted).unwrap(), source);
    }
}

#[test]
fn per_decision_order_preserves_a_shared_dag_without_requiring_global_action_order() {
    let mut source = record(Game {
        root: n("root"),
        nodes: BTreeMap::from([
            (
                n("root"),
                decision("urn:entity:7:1", &[("x", "other"), ("y", "chance")]),
            ),
            (
                n("other"),
                decision("urn:entity:7:2", &[("b", "second"), ("a", "shared")]),
            ),
            (
                n("chance"),
                Node::Chance {
                    edges: vec![
                        ChanceEdge {
                            probability: -0.0,
                            target: n("second"),
                        },
                        ChanceEdge {
                            probability: 1.0,
                            target: n("shared"),
                        },
                    ],
                },
            ),
            (
                n("shared"),
                terminal(&[("urn:entity:7:1", 1.0), ("urn:entity:7:2", 2.0)]),
            ),
            (
                n("second"),
                terminal(&[("urn:entity:7:1", 4.0), ("urn:entity:7:2", 1.0)]),
            ),
        ]),
    });
    source.selection_tolerance = -0.0;
    let map = context_map(&[
        (Role::Node, "root", "e"),
        (Role::Node, "other", "d"),
        (Role::Node, "chance", "c"),
        (Role::Node, "shared", "b"),
        (Role::Node, "second", "a"),
        (Role::Player, "urn:entity:7:1", "z-player"),
        (Role::Player, "urn:entity:7:2", "a-player"),
        (Role::Action, "a", "z-a"),
        (Role::Action, "b", "z-b"),
        (Role::Action, "x", "0-x"),
        (Role::Action, "y", "0-y"),
    ]);
    let converted = convert_static_game(&source, &map, STATIC_GAME_PROFILE_V1).unwrap();
    assert_solver_invariant(&source, &converted);
    assert_eq!(
        converted.record.selection_tolerance.to_bits(),
        (-0.0_f64).to_bits()
    );
    let Node::Chance { edges } = &converted.record.specification.game.nodes[&n("c")] else {
        panic!("chance")
    };
    assert_eq!(edges[0].probability.to_bits(), (-0.0_f64).to_bits());
    assert_eq!(edges[1].target, n("b"));
    assert_eq!(restore_static_game(&converted).unwrap(), source);
}

#[test]
fn all_nine_identity_roles_can_share_a_full_string_without_collapsing() {
    let mut source = record(Game {
        root: n(CONTEXT),
        nodes: BTreeMap::from([
            (n(CONTEXT), decision(CONTEXT, &[("z", "b"), (CONTEXT, "a")])),
            (n("a"), terminal(&[(CONTEXT, -0.0)])),
            (n("b"), terminal(&[(CONTEXT, f64::from_bits(1))])),
        ]),
    });
    source.selection_tolerance = 0.0;
    let map = context_map(&[
        (Role::Node, CONTEXT, "record"),
        (Role::Node, "a", "a"),
        (Role::Node, "b", "b"),
        (Role::Player, CONTEXT, "record"),
        (Role::Action, CONTEXT, "record"),
        (Role::Action, "z", "z"),
    ]);
    let converted = convert_static_game(&source, &map, STATIC_GAME_PROFILE_V1).unwrap();
    assert_eq!(
        converted
            .mapping
            .bindings
            .iter()
            .filter(|b| b.source == CONTEXT && b.target == "record")
            .count(),
        9
    );
    assert_eq!(converted.record.specification.game.root, n("record"));
    assert_solver_invariant(&source, &converted);
    let restored = restore_static_game(&converted).unwrap();
    assert_eq!(restored, source);
    let Node::Terminal { payoffs } = &restored.specification.game.nodes[&n("a")] else {
        panic!("terminal")
    };
    assert_eq!(payoffs[&p(CONTEXT)].to_bits(), (-0.0_f64).to_bits());
    let Node::Terminal { payoffs } = &restored.specification.game.nodes[&n("b")] else {
        panic!("terminal")
    };
    assert_eq!(payoffs[&p(CONTEXT)].to_bits(), 1);
    for role in [
        Role::Specification,
        Role::Instance,
        Role::Session,
        Role::PolicyArtifact,
        Role::SolverRun,
        Role::SolverDiagnosticTrace,
        Role::Player,
        Role::Node,
        Role::Action,
    ] {
        let mut incomplete = map.clone();
        incomplete
            .bindings
            .retain(|b| !(b.role == role && b.source == CONTEXT));
        assert_eq!(
            convert_static_game(&source, &incomplete, STATIC_GAME_PROFILE_V1)
                .unwrap_err()
                .code,
            "missing_identity_mapping"
        );
    }
}

#[test]
fn changing_action_lexical_order_rejects_even_when_current_payoffs_do_not_tie() {
    let (mut source, mut map) = chain(&["a", "b", "c"]);
    source.selection_tolerance = 0.0;
    map.bindings
        .iter_mut()
        .find(|b| b.role == Role::Action && b.source == "a")
        .unwrap()
        .target = "💥".into();
    assert_eq!(
        convert_static_game(&source, &map, STATIC_GAME_PROFILE_V1)
            .unwrap_err()
            .code,
        "selection_order_changed"
    );
}

#[test]
fn reverse_mapping_revalidates_public_structure_and_ledger_mutations() {
    let (source, map) = chain(&["c", "b", "a"]);
    let original = convert_static_game(&source, &map, STATIC_GAME_PROFILE_V1).unwrap();
    let mut changed = original.clone();
    changed.record.instance_id = GameInstanceId("missing-instance".into());
    assert_eq!(
        restore_static_game(&changed).unwrap_err().code,
        "unknown_identity_mapping"
    );
    let mut changed = original.clone();
    changed
        .mapping
        .bindings
        .retain(|b| !(b.role == Role::Action && b.source == "a"));
    assert_eq!(
        restore_static_game(&changed).unwrap_err().code,
        "missing_identity_mapping"
    );
    let mut changed = original.clone();
    changed
        .mapping
        .bindings
        .iter_mut()
        .find(|b| b.role == Role::Action && b.source == "a")
        .unwrap()
        .source = "z".into();
    assert_eq!(
        restore_static_game(&changed).unwrap_err().code,
        "selection_order_changed"
    );
    let mut changed = original.clone();
    changed.record.information_structure = ResultInformationStructure::ImperfectInformation;
    assert_eq!(
        restore_static_game(&changed).unwrap_err().code,
        "unsupported_conversion_information"
    );
    let mut changed = original.clone();
    changed.record.chance_rule = StaticChanceRule::StrictlyPositiveOnly;
    assert_eq!(
        restore_static_game(&changed).unwrap_err().code,
        "unsupported_conversion_chance_rule"
    );
    let mut changed = original.clone();
    changed.record.payoff_rule = StaticPayoffRule::CompleteVectors;
    assert_eq!(
        restore_static_game(&changed).unwrap_err().code,
        "unsupported_conversion_payoff_rule"
    );
    let mut changed = original;
    changed.record.selection_tolerance = f64::NAN;
    assert_eq!(
        restore_static_game(&changed).unwrap_err().code,
        "invalid_conversion_tolerance"
    );
}

#[test]
fn absent_context_is_not_invented_and_extra_context_bindings_are_rejected() {
    let (mut source, mut map) = chain(&["a", "b", "c"]);
    source.session_id = None;
    source.policy_artifact_id = None;
    source.solver_run_id = None;
    source.solver_trace_id = None;
    assert_eq!(
        convert_static_game(&source, &map, STATIC_GAME_PROFILE_V1)
            .unwrap_err()
            .code,
        "unknown_identity_mapping"
    );
    map.bindings.retain(|b| {
        !matches!(
            b.role,
            Role::Session | Role::PolicyArtifact | Role::SolverRun | Role::SolverDiagnosticTrace
        )
    });
    let converted = convert_static_game(&source, &map, STATIC_GAME_PROFILE_V1).unwrap();
    assert!(
        converted.record.session_id.is_none()
            && converted.record.policy_artifact_id.is_none()
            && converted.record.solver_run_id.is_none()
            && converted.record.solver_trace_id.is_none()
    );
    assert_eq!(restore_static_game(&converted).unwrap(), source);
}
