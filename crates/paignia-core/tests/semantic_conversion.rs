//! Independent fixed admission controls for the neutral static conversion.

use paignia_core::{
    convert_static_game, restore_static_game, ActionEdge, ActionId, ChanceEdge, Game,
    GameInstanceId, GameSessionId, GameSpecification, Node, NodeId, PlayerId, PolicyArtifactId,
    ResultInformationStructure, ResultSelectionRule, SolverDiagnosticTraceId, SolverRunId,
    StaticChanceRule, StaticGameRecord, StaticIdentityBinding, StaticIdentityMap,
    StaticIdentityRole as Role, StaticPayoffRule,
};
use std::collections::BTreeMap;

const PROFILE: &str = "paignia/static-finite-perfect-information-sparse-zero/1.0.0";
const PLAYER: &str = "https://w3id.org/uogto/core#player";
const OTHER: &str = "https://kairos.dev/ontology/game-theory#player";
const FIRST: &str = "https://kairos.dev/ontology/game-theory#act";
const SECOND: &str = "https://w3id.org/uogto/core#act";

fn source() -> StaticGameRecord {
    StaticGameRecord {
        schema_version: "1.0.0".into(),
        semantic_profile: PROFILE.into(),
        information_structure: ResultInformationStructure::PerfectInformation,
        chance_rule: StaticChanceRule::AllowZeroAndValidateEveryBranch,
        payoff_rule: StaticPayoffRule::IntentionalSparseZero,
        selection_rule: ResultSelectionRule::InputOrderPairwiseToleranceThenLexicographic,
        selection_tolerance: -0.0,
        specification: GameSpecification {
            schema_version: "1.0.0".into(),
            id: "https://w3id.org/uogto/extensions#specification".into(),
            game: Game {
                root: NodeId("root".into()),
                nodes: BTreeMap::from([
                    (
                        NodeId("root".into()),
                        Node::Decision {
                            player: PlayerId(PLAYER.into()),
                            edges: vec![
                                ActionEdge {
                                    action: ActionId(SECOND.into()),
                                    target: NodeId("chance".into()),
                                },
                                ActionEdge {
                                    action: ActionId(FIRST.into()),
                                    target: NodeId("urn:kairos:entity:7:2".into()),
                                },
                            ],
                        },
                    ),
                    (
                        NodeId("chance".into()),
                        Node::Chance {
                            edges: vec![
                                ChanceEdge {
                                    probability: -0.0,
                                    target: NodeId("urn:kairos:entity:7:1".into()),
                                },
                                ChanceEdge {
                                    probability: 1.0,
                                    target: NodeId("urn:kairos:entity:7:2".into()),
                                },
                            ],
                        },
                    ),
                    (
                        NodeId("urn:kairos:entity:7:1".into()),
                        Node::Terminal {
                            payoffs: BTreeMap::from([
                                (PlayerId(PLAYER.into()), -0.0),
                                (PlayerId(OTHER.into()), f64::from_bits(1)),
                            ]),
                        },
                    ),
                    (
                        NodeId("urn:kairos:entity:7:2".into()),
                        Node::Terminal {
                            payoffs: BTreeMap::from([(PlayerId(PLAYER.into()), -3.5)]),
                        },
                    ),
                ]),
            },
        },
        instance_id: GameInstanceId("same-context".into()),
        session_id: Some(GameSessionId("same-context".into())),
        policy_artifact_id: Some(PolicyArtifactId("urn:policy".into())),
        solver_run_id: Some(SolverRunId("urn:run".into())),
        solver_trace_id: Some(SolverDiagnosticTraceId("urn:diagnostic".into())),
    }
}

fn mapping() -> StaticIdentityMap {
    let mut bindings = Vec::new();
    for (role, ids) in [
        (
            Role::Specification,
            vec!["https://w3id.org/uogto/extensions#specification"],
        ),
        (Role::Instance, vec!["same-context"]),
        (Role::Session, vec!["same-context"]),
        (Role::PolicyArtifact, vec!["urn:policy"]),
        (Role::SolverRun, vec!["urn:run"]),
        (Role::SolverDiagnosticTrace, vec!["urn:diagnostic"]),
        (Role::Player, vec![PLAYER, OTHER]),
        (
            Role::Node,
            vec![
                "root",
                "chance",
                "urn:kairos:entity:7:1",
                "urn:kairos:entity:7:2",
            ],
        ),
        (Role::Action, vec![FIRST, SECOND]),
    ] {
        for id in ids {
            bindings.push(StaticIdentityBinding {
                role,
                source: id.into(),
                target: format!("mapped:{role:?}:{id}"),
            });
        }
    }
    StaticIdentityMap {
        schema_version: "1.0.0".into(),
        bindings,
    }
}

fn mapped(map: &StaticIdentityMap, role: Role, id: &str) -> String {
    map.bindings
        .iter()
        .find(|b| b.role == role && b.source == id)
        .unwrap()
        .target
        .clone()
}

fn reject(record: &StaticGameRecord, map: &StaticIdentityMap, code: &str) {
    let actual = convert_static_game(record, map, PROFILE)
        .expect_err("the explicit admission boundary must reject");
    assert_eq!(actual.code, code, "{actual:?}");
}

#[test]
fn round_trip_preserves_every_role_full_namespace_and_complete_ledger() {
    let record = source();
    let map = mapping();
    let converted = convert_static_game(&record, &map, PROFILE).unwrap();
    assert_eq!(converted.mapping, map);
    assert_eq!(restore_static_game(&converted).unwrap(), record);
    assert_ne!(
        converted.record.instance_id.0,
        converted.record.session_id.as_ref().unwrap().0
    );
    assert_eq!(
        converted.record.specification.id,
        mapped(&map, Role::Specification, &record.specification.id)
    );
    assert_eq!(converted.record.schema_version, "1.0.0");
}

#[test]
fn stored_branch_order_and_numeric_bits_survive_conversion() {
    let record = source();
    let map = mapping();
    let actual = convert_static_game(&record, &map, PROFILE).unwrap().record;
    let Node::Decision { player, edges } =
        &actual.specification.game.nodes[&NodeId(mapped(&map, Role::Node, "root"))]
    else {
        panic!("decision expected")
    };
    assert_eq!(player.0, mapped(&map, Role::Player, PLAYER));
    assert_eq!(edges[0].action.0, mapped(&map, Role::Action, SECOND));
    assert_eq!(edges[1].action.0, mapped(&map, Role::Action, FIRST));
    let Node::Chance { edges } =
        &actual.specification.game.nodes[&NodeId(mapped(&map, Role::Node, "chance"))]
    else {
        panic!("chance expected")
    };
    assert_eq!(edges[0].probability.to_bits(), (-0.0_f64).to_bits());
    assert_eq!(edges[1].probability.to_bits(), 1.0_f64.to_bits());
    let Node::Terminal { payoffs } = &actual.specification.game.nodes
        [&NodeId(mapped(&map, Role::Node, "urn:kairos:entity:7:1"))]
    else {
        panic!("terminal expected")
    };
    assert_eq!(
        payoffs[&PlayerId(mapped(&map, Role::Player, PLAYER))].to_bits(),
        (-0.0_f64).to_bits()
    );
    assert_eq!(
        payoffs[&PlayerId(mapped(&map, Role::Player, OTHER))].to_bits(),
        1
    );
    assert_eq!(actual.selection_tolerance.to_bits(), (-0.0_f64).to_bits());
}

#[test]
fn absent_payoffs_are_not_filled_or_dropped() {
    let map = mapping();
    let actual = convert_static_game(&source(), &map, PROFILE).unwrap();
    let Node::Terminal { payoffs } = &actual.record.specification.game.nodes
        [&NodeId(mapped(&map, Role::Node, "urn:kairos:entity:7:2"))]
    else {
        panic!("terminal expected")
    };
    assert_eq!(payoffs.len(), 1);
    assert!(!payoffs.contains_key(&PlayerId(mapped(&map, Role::Player, OTHER))));
}

#[test]
fn generation_like_opaque_identities_do_not_collapse_to_an_index() {
    let map = mapping();
    let actual = convert_static_game(&source(), &map, PROFILE).unwrap();
    assert!(actual
        .record
        .specification
        .game
        .nodes
        .contains_key(&NodeId(mapped(&map, Role::Node, "urn:kairos:entity:7:1"))));
    assert!(actual
        .record
        .specification
        .game
        .nodes
        .contains_key(&NodeId(mapped(&map, Role::Node, "urn:kairos:entity:7:2"))));
    let mut collapsed = map;
    for binding in &mut collapsed.bindings {
        if binding.role == Role::Node && binding.source.starts_with("urn:kairos:entity:") {
            binding.target = "entity:7".into();
        }
    }
    reject(&source(), &collapsed, "identity_target_collision");
}

#[test]
fn every_mapping_is_explicit_even_when_its_target_string_is_unchanged() {
    let mut map = mapping();
    for b in &mut map.bindings {
        b.target.clone_from(&b.source);
    }
    let actual = convert_static_game(&source(), &map, PROFILE).unwrap();
    assert_eq!(actual.record, source());
    map.bindings.pop();
    reject(&source(), &map, "missing_identity_mapping");
}

#[test]
fn duplicate_source_bindings_reject_even_when_identical() {
    let mut map = mapping();
    map.bindings.push(map.bindings[0].clone());
    reject(&source(), &map, "duplicate_identity_source");
}

#[test]
fn distinct_full_iris_with_the_same_local_name_cannot_share_a_target() {
    let mut map = mapping();
    for b in &mut map.bindings {
        if b.role == Role::Player {
            b.target = "player".into();
        }
    }
    reject(&source(), &map, "identity_target_collision");
}

#[test]
fn same_string_in_different_roles_is_not_a_collision() {
    let mut map = mapping();
    for b in &mut map.bindings {
        if matches!(b.role, Role::Instance | Role::Session) {
            b.target = "same-target".into();
        }
    }
    let actual = convert_static_game(&source(), &map, PROFILE).unwrap();
    assert_eq!(restore_static_game(&actual).unwrap(), source());
}

#[test]
fn unknown_or_shortened_source_names_are_not_matched_implicitly() {
    let mut map = mapping();
    map.bindings[0].source = "specification".into();
    reject(&source(), &map, "unknown_identity_mapping");
}

#[test]
fn unused_mapping_cannot_hide_an_unknown_identity() {
    let mut map = mapping();
    map.bindings.push(StaticIdentityBinding {
        role: Role::Node,
        source: "not-in-graph".into(),
        target: "unused".into(),
    });
    reject(&source(), &map, "unknown_identity_mapping");
}

#[test]
fn a_binding_in_the_wrong_role_is_not_a_mapping_for_that_identity() {
    let mut map = mapping();
    map.bindings[0].role = Role::Node;
    reject(&source(), &map, "unknown_identity_mapping");
}

#[test]
fn blank_source_and_target_strings_reject_without_trimming_other_strings() {
    for blank in ["", " ", "\u{2003}"] {
        let mut map = mapping();
        map.bindings[0].source = blank.into();
        reject(&source(), &map, "empty_identity_mapping");
        let mut map = mapping();
        map.bindings[0].target = blank.into();
        reject(&source(), &map, "empty_identity_mapping");
    }
    let mut map = mapping();
    map.bindings[0].target = "  urn:preserve\0identity  ".into();
    let actual = convert_static_game(&source(), &map, PROFILE).unwrap();
    assert_eq!(actual.record.specification.id, "  urn:preserve\0identity  ");
}

#[test]
fn action_renaming_that_reverses_lexical_ties_rejects() {
    let mut map = mapping();
    for b in &mut map.bindings {
        if b.role == Role::Action {
            b.target = if b.source == FIRST { "z" } else { "a" }.into();
        }
    }
    reject(&source(), &map, "selection_order_changed");
}

#[test]
fn order_preserving_action_aliases_keep_a_reversible_mapping() {
    let mut map = mapping();
    for b in &mut map.bindings {
        if b.role == Role::Action {
            b.target = if b.source == FIRST { "a" } else { "z" }.into();
        }
    }
    let actual = convert_static_game(&source(), &map, PROFILE).unwrap();
    assert_eq!(restore_static_game(&actual).unwrap(), source());
}

#[test]
fn unknown_record_schema_rejects_before_other_invalid_fields() {
    let mut record = source();
    record.schema_version = "1.0.1".into();
    record.specification.game.nodes.clear();
    reject(&record, &mapping(), "unsupported_static_record_schema");
}

#[test]
fn unknown_mapping_schema_rejects() {
    let mut map = mapping();
    map.schema_version = "1.0".into();
    reject(&source(), &map, "unsupported_identity_map_schema");
}

#[test]
fn source_and_target_profiles_require_exact_supported_identity() {
    for bad in [
        "1.0.0",
        "",
        "kairos/strict-positive-complete-first-maximum/1.0.0",
        "paignia/static-finite-perfect-information-sparse-zero/1.0.1",
    ] {
        let mut record = source();
        record.semantic_profile = bad.into();
        reject(&record, &mapping(), "unsupported_semantic_profile");
        assert_eq!(
            convert_static_game(&source(), &mapping(), bad)
                .unwrap_err()
                .code,
            "unsupported_target_profile"
        );
    }
}

#[test]
fn information_sets_and_simultaneous_structure_are_never_discarded() {
    for information in [
        ResultInformationStructure::ImperfectInformation,
        ResultInformationStructure::Simultaneous,
    ] {
        let mut record = source();
        record.information_structure = information;
        reject(&record, &mapping(), "unsupported_conversion_information");
    }
}

#[test]
fn strict_positive_chance_policy_is_not_silently_converted() {
    let mut record = source();
    record.chance_rule = StaticChanceRule::StrictlyPositiveOnly;
    reject(&record, &mapping(), "unsupported_conversion_chance_rule");
}

#[test]
fn complete_payoff_claim_is_not_silently_changed_to_sparse_zero() {
    let mut record = source();
    record.payoff_rule = StaticPayoffRule::CompleteVectors;
    reject(&record, &mapping(), "unsupported_conversion_payoff_rule");
}

#[test]
fn unspecified_selection_is_not_assigned_the_existing_tie_policy() {
    let mut record = source();
    record.selection_rule = ResultSelectionRule::Unspecified;
    reject(&record, &mapping(), "unsupported_conversion_selection");
}

#[test]
fn nonfinite_or_negative_selection_tolerance_rejects() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -1e-30] {
        let mut record = source();
        record.selection_tolerance = value;
        reject(&record, &mapping(), "invalid_conversion_tolerance");
    }
}

#[test]
fn invalid_source_graph_rejects_even_for_a_zero_probability_branch() {
    let mut record = source();
    record
        .specification
        .game
        .nodes
        .remove(&NodeId("urn:kairos:entity:7:1".into()));
    reject(&record, &mapping(), "missing_target");
}

#[test]
fn nonfinite_source_payoff_remains_an_error() {
    let mut record = source();
    let Node::Terminal { payoffs } = record
        .specification
        .game
        .nodes
        .get_mut(&NodeId("urn:kairos:entity:7:2".into()))
        .unwrap()
    else {
        panic!("terminal expected")
    };
    payoffs.insert(PlayerId(PLAYER.into()), f64::NAN);
    reject(&record, &mapping(), "invalid_payoff");
}

#[test]
fn absent_context_stays_absent_in_both_directions() {
    let mut record = source();
    record.session_id = None;
    record.policy_artifact_id = None;
    record.solver_run_id = None;
    record.solver_trace_id = None;
    let mut map = mapping();
    map.bindings.retain(|b| {
        !matches!(
            b.role,
            Role::Session | Role::PolicyArtifact | Role::SolverRun | Role::SolverDiagnosticTrace
        )
    });
    let actual = convert_static_game(&record, &map, PROFILE).unwrap();
    assert_eq!(restore_static_game(&actual).unwrap(), record);
    assert!(actual.record.solver_trace_id.is_none());
}

#[test]
fn supplied_blank_context_is_not_treated_as_absent() {
    for role in [
        Role::Instance,
        Role::Session,
        Role::PolicyArtifact,
        Role::SolverRun,
        Role::SolverDiagnosticTrace,
    ] {
        let mut record = source();
        match role {
            Role::Instance => record.instance_id.0 = " ".into(),
            Role::Session => record.session_id = Some(GameSessionId(" ".into())),
            Role::PolicyArtifact => record.policy_artifact_id = Some(PolicyArtifactId(" ".into())),
            Role::SolverRun => record.solver_run_id = Some(SolverRunId(" ".into())),
            Role::SolverDiagnosticTrace => {
                record.solver_trace_id = Some(SolverDiagnosticTraceId(" ".into()));
            }
            _ => unreachable!(),
        }
        reject(&record, &mapping(), "empty_conversion_identity");
    }
}

#[test]
fn invalid_public_mutation_is_revalidated_on_restore() {
    let mut actual = convert_static_game(&source(), &mapping(), PROFILE).unwrap();
    actual.record.instance_id.0 = "unmapped-change".into();
    assert_eq!(
        restore_static_game(&actual).unwrap_err().code,
        "unknown_identity_mapping"
    );
}

#[test]
fn conversion_never_mutates_source_or_mapping_even_on_error() {
    let record = source();
    let original = record.clone();
    let mut map = mapping();
    map.bindings.pop();
    let original_map = map.clone();
    reject(&record, &map, "missing_identity_mapping");
    assert_eq!(record, original);
    assert_eq!(map, original_map);
}

#[test]
fn terminal_only_empty_payoffs_need_no_invented_players_or_actions() {
    let mut record = source();
    record.specification.game = Game {
        root: NodeId("terminal".into()),
        nodes: BTreeMap::from([(
            NodeId("terminal".into()),
            Node::Terminal {
                payoffs: BTreeMap::new(),
            },
        )]),
    };
    let mut map = mapping();
    map.bindings
        .retain(|b| !matches!(b.role, Role::Player | Role::Node | Role::Action));
    map.bindings.push(StaticIdentityBinding {
        role: Role::Node,
        source: "terminal".into(),
        target: "mapped-terminal".into(),
    });
    let actual = convert_static_game(&record, &map, PROFILE).unwrap();
    assert_eq!(restore_static_game(&actual).unwrap(), record);
}
