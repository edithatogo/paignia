//! Native projection and binding controls, without native lifecycle claims.

use std::collections::BTreeMap;
use std::error::Error as StdError;

use paignia_core::{
    build_policy_observation, ActionEdge, ActionId, ChanceEdge, Game, GameInstanceId,
    GameSessionId, GameSpecification, GameSpecificationId, Node, NodeId, ObservationRevision,
    PlayerId, PolicyObservation, PolicyObservationBinding as Binding,
    PolicyObservationError as Error, PolicyObservationNodeKind as NodeKind,
    PolicyObservationRequest as Request, ProfileValidationError, ValidationError,
    POLICY_OBSERVATION_SCHEMA_VERSION, STATIC_GAME_PROFILE_V1,
};

fn node(value: &str) -> NodeId {
    NodeId(format!("https://example.test/node/{value}"))
}
fn action(value: &str) -> ActionId {
    ActionId(format!("https://example.test/action/{value}"))
}
fn player(value: &str) -> PlayerId {
    PlayerId(format!("https://example.test/player/{value}"))
}

fn source() -> GameSpecification {
    GameSpecification {
        schema_version: "1.0.0".into(),
        id: "https://example.test/specification/observation".into(),
        game: Game {
            root: node("public-decision"),
            nodes: BTreeMap::from([
                (
                    node("public-decision"),
                    Node::Decision {
                        player: player("acting"),
                        edges: vec![
                            ActionEdge {
                                action: action("zeta"),
                                target: node("unobserved-chance"),
                            },
                            ActionEdge {
                                action: action("alpha"),
                                target: node("unobserved-opponent"),
                            },
                        ],
                    },
                ),
                (
                    node("unobserved-chance"),
                    Node::Chance {
                        edges: vec![
                            ChanceEdge {
                                probability: 0.25,
                                target: node("unobserved-left"),
                            },
                            ChanceEdge {
                                probability: 0.75,
                                target: node("unobserved-right"),
                            },
                        ],
                    },
                ),
                (
                    node("unobserved-opponent"),
                    Node::Decision {
                        player: player("opponent"),
                        edges: vec![ActionEdge {
                            action: action("unobserved-continuation"),
                            target: node("unobserved-other"),
                        }],
                    },
                ),
                (
                    node("unobserved-left"),
                    Node::Terminal {
                        payoffs: BTreeMap::from([(player("acting"), 123_456.0)]),
                    },
                ),
                (
                    node("unobserved-right"),
                    Node::Terminal {
                        payoffs: BTreeMap::from([(player("opponent"), -654_321.0)]),
                    },
                ),
                (
                    node("unobserved-other"),
                    Node::Terminal {
                        payoffs: BTreeMap::new(),
                    },
                ),
            ]),
        },
    }
}

fn binding() -> Binding {
    Binding {
        specification_id: GameSpecificationId(source().id),
        game_instance_id: GameInstanceId("urn:example:instance:opaque".into()),
        session_id: GameSessionId("urn:example:session:opaque".into()),
        player_id: player("acting"),
        decision_node_id: node("public-decision"),
        revision: ObservationRevision(17),
    }
}

fn request(expected: &Binding) -> Request {
    Request {
        schema_version: "1.0.0".into(),
        semantic_profile: STATIC_GAME_PROFILE_V1.into(),
        binding: expected.clone(),
    }
}

fn observe(source: &GameSpecification, expected: &Binding) -> PolicyObservation {
    build_policy_observation(source, expected, &request(expected)).unwrap()
}

fn rejects(
    source: &GameSpecification,
    expected: &Binding,
    request: &Request,
    code: &str,
    path: &str,
) -> Error {
    let error = build_policy_observation(source, expected, request).unwrap_err();
    assert_eq!(error.code(), code);
    assert_eq!(error.path(), path);
    error
}

#[test]
fn returns_only_the_declared_owned_policy_dto_fields() {
    assert_eq!(POLICY_OBSERVATION_SCHEMA_VERSION, "1.0.0");
    let expected = binding();
    // Exhaustive destructuring intentionally pins the entire public DTO shape.
    let PolicyObservation {
        schema_version,
        semantic_profile,
        binding,
        legal_actions,
    } = observe(&source(), &expected);
    assert_eq!(schema_version, "1.0.0");
    assert_eq!(semantic_profile, STATIC_GAME_PROFILE_V1);
    assert_eq!(binding, expected);
    assert_eq!(legal_actions, vec![action("zeta"), action("alpha")]);
}

#[test]
fn observation_outlives_all_builder_inputs() {
    fn owned_static<T: 'static>(value: T) -> T {
        value
    }
    let observation = {
        let source = source();
        let expected = binding();
        let request = request(&expected);
        owned_static(build_policy_observation(&source, &expected, &request).unwrap())
    };
    assert_eq!(observation.binding, binding());
    assert_eq!(
        observation.legal_actions,
        vec![action("zeta"), action("alpha")]
    );
}

#[test]
fn successful_projection_never_contains_unobserved_nodes_players_or_payoffs() {
    let observation = observe(&source(), &binding());
    let display = format!("{observation:?}");
    for excluded in [
        "unobserved",
        "opponent",
        "123456",
        "654321",
        "probability",
        "target",
        "payoffs",
        "trace",
    ] {
        assert!(
            !display.contains(excluded),
            "projection contains {excluded}"
        );
    }
}

#[test]
fn changing_unobserved_terminal_payoffs_preserves_the_complete_dto() {
    let initial = source();
    let mut changed = initial.clone();
    for value in changed.game.nodes.values_mut() {
        if let Node::Terminal { payoffs } = value {
            *payoffs = BTreeMap::from([(player("different-unobserved-owner"), -99_000.5)]);
        }
    }
    assert_eq!(observe(&initial, &binding()), observe(&changed, &binding()));
}

#[test]
fn changing_unobserved_chance_and_opponent_decisions_preserves_the_dto() {
    let initial = source();
    let mut changed = initial.clone();
    if let Node::Chance { edges } = changed
        .game
        .nodes
        .get_mut(&node("unobserved-chance"))
        .unwrap()
    {
        edges[0].probability = 0.5;
        edges[1].probability = 0.5;
        edges.reverse();
    }
    if let Node::Decision {
        player: owner,
        edges,
    } = changed
        .game
        .nodes
        .get_mut(&node("unobserved-opponent"))
        .unwrap()
    {
        *owner = player("different-unobserved-owner");
        edges[0].action = action("different-unobserved-action");
    }
    assert_eq!(observe(&initial, &binding()), observe(&changed, &binding()));
}

#[test]
fn edge_targets_are_not_part_of_the_policy_projection() {
    let initial = source();
    let mut changed = initial.clone();
    if let Node::Decision { edges, .. } = changed
        .game
        .nodes
        .get_mut(&node("public-decision"))
        .unwrap()
    {
        let other = edges[1].target.clone();
        edges[1].target = edges[0].target.clone();
        edges[0].target = other;
    }
    assert_eq!(observe(&initial, &binding()), observe(&changed, &binding()));
}

#[test]
fn legal_actions_preserve_declared_order_instead_of_sorting() {
    let initial = source();
    let original = observe(&initial, &binding());
    let mut changed = initial;
    if let Node::Decision { edges, .. } = changed
        .game
        .nodes
        .get_mut(&node("public-decision"))
        .unwrap()
    {
        edges.reverse();
    }
    let reordered = observe(&changed, &binding());
    assert_eq!(
        original.legal_actions,
        vec![action("zeta"), action("alpha")]
    );
    assert_eq!(
        reordered.legal_actions,
        vec![action("alpha"), action("zeta")]
    );
    assert_eq!(reordered.binding, original.binding);
    assert_ne!(reordered, original);
}

#[test]
fn construction_preserves_source_expected_context_and_request() {
    let source = source();
    let expected = binding();
    let request = request(&expected);
    let before = (source.clone(), expected.clone(), request.clone());
    build_policy_observation(&source, &expected, &request).unwrap();
    assert_eq!((source, expected, request), before);
}

#[test]
fn opaque_full_identifiers_survive_without_trimming_or_shortening() {
    let mut source = source();
    source.id = " https://example.test/full/specification/長い?rev=1#identity ".into();
    let mut expected = binding();
    expected.specification_id = GameSpecificationId(source.id.clone());
    expected.game_instance_id = GameInstanceId("urn:example:instance:full/路径\0tail".into());
    expected.session_id = GameSessionId(" urn:example:session:unchanged ".into());
    let old_root = source.game.root.clone();
    let mut root = source.game.nodes.remove(&old_root).unwrap();
    expected.decision_node_id =
        NodeId("https://example.test/full/node/decision?case=α#body".into());
    expected.player_id = PlayerId("https://example.test/full/player/acting#opaque".into());
    let public_action = ActionId(" urn:example:action:full/路径?x=1#fragment ".into());
    if let Node::Decision { player, edges } = &mut root {
        *player = expected.player_id.clone();
        edges[0].action = public_action.clone();
    }
    source.game.root = expected.decision_node_id.clone();
    source
        .game
        .nodes
        .insert(expected.decision_node_id.clone(), root);
    let output = observe(&source, &expected);
    assert_eq!(output.binding, expected);
    assert_eq!(output.legal_actions[0], public_action);
}

#[test]
fn equal_strings_in_distinct_identity_roles_remain_distinct_typed_fields() {
    let mut source = source();
    source.id = "urn:example:shared".into();
    let mut expected = binding();
    expected.specification_id = GameSpecificationId(source.id.clone());
    expected.game_instance_id = GameInstanceId(source.id.clone());
    expected.session_id = GameSessionId(source.id.clone());
    assert_eq!(observe(&source, &expected).binding, expected);
}

#[test]
fn revisions_are_opaque_labels_including_zero_and_maximum() {
    for value in [0, 17, u64::MAX] {
        let mut expected = binding();
        expected.revision = ObservationRevision(value);
        assert_eq!(
            observe(&source(), &expected).binding.revision,
            ObservationRevision(value)
        );
    }
}

#[test]
fn a_nonroot_owned_decision_is_an_explicit_valid_context() {
    let mut expected = binding();
    expected.decision_node_id = node("unobserved-opponent");
    expected.player_id = player("opponent");
    let observation = observe(&source(), &expected);
    assert_eq!(observation.binding, expected);
    assert_eq!(
        observation.legal_actions,
        vec![action("unobserved-continuation")]
    );
}

#[test]
fn unsupported_request_schemas_precede_profile_source_and_binding_errors() {
    for schema in ["", "2.0.0", "1.0.1", " 1.0.0", "1.0.0\n"] {
        let mut source = source();
        source.schema_version = "unsupported".into();
        let mut expected = binding();
        expected.session_id.0.clear();
        let mut request = request(&expected);
        request.schema_version = schema.into();
        request.semantic_profile.clear();
        let error = rejects(
            &source,
            &expected,
            &request,
            "unsupported_policy_observation_schema",
            "schema_version",
        );
        assert_eq!(
            error,
            Error::UnsupportedSchemaVersion {
                value: schema.into()
            }
        );
    }
}

#[test]
fn unsupported_information_structures_and_profile_spellings_fail_before_graph_validation() {
    for profile in [
        "",
        "paignia/static-finite-perfect-information-sparse-zero/2.0.0",
        "paignia/extensive-information-sets/1.0.0",
        "paignia/simultaneous/1.0.0",
        "paignia/hidden-state/1.0.0",
        " paignia/static-finite-perfect-information-sparse-zero/1.0.0",
    ] {
        let mut source = source();
        source.game.nodes.clear();
        let expected = binding();
        let mut request = request(&expected);
        request.semantic_profile = profile.into();
        let error = rejects(
            &source,
            &expected,
            &request,
            "unsupported_semantic_profile",
            "semantic_profile",
        );
        assert_eq!(
            error,
            Error::InvalidSource(ProfileValidationError::UnsupportedSemanticProfile {
                value: profile.into()
            })
        );
    }
}

#[test]
fn invalid_specification_schema_retains_the_original_typed_source() {
    let mut source = source();
    source.schema_version = "2.0.0".into();
    let expected = binding();
    let error = rejects(
        &source,
        &expected,
        &request(&expected),
        "unsupported_schema_version",
        "source.schema_version",
    );
    let wrapped = error
        .source()
        .unwrap()
        .downcast_ref::<ProfileValidationError>()
        .unwrap();
    assert_eq!(
        wrapped,
        &ProfileValidationError::InvalidSpecification(ValidationError::UnsupportedSchemaVersion {
            path: "schema_version".into(),
            value: "2.0.0".into()
        })
    );
}

#[test]
fn invalid_graphs_never_receive_an_observation() {
    let mut source = source();
    source.game.root = node("missing-root");
    let expected = binding();
    let error = rejects(
        &source,
        &expected,
        &request(&expected),
        "missing_root",
        "source.game.nodes.https://example.test/node/missing-root",
    );
    assert_eq!(
        error,
        Error::InvalidSource(ProfileValidationError::InvalidSpecification(
            ValidationError::MissingRoot(node("missing-root"))
        ))
    );
}

#[test]
fn unobserved_invalid_payoffs_still_fail_full_source_validation() {
    let mut source = source();
    source.game.nodes.insert(
        node("unobserved-other"),
        Node::Terminal {
            payoffs: BTreeMap::from([(player("opponent"), f64::NAN)]),
        },
    );
    let expected = binding();
    let error = rejects(
        &source,
        &expected,
        &request(&expected),
        "invalid_payoff",
        "source.game.nodes.https://example.test/node/unobserved-other",
    );
    assert!(matches!(
        error,
        Error::InvalidSource(ProfileValidationError::InvalidSpecification(
            ValidationError::InvalidPayoff { .. }
        ))
    ));
}

type BlankMutation = fn(&mut Binding);

fn blank_fields() -> Vec<(&'static str, BlankMutation)> {
    vec![
        ("specification_id", |b| b.specification_id.0 = " \t".into()),
        ("game_instance_id", |b| b.game_instance_id.0.clear()),
        ("session_id", |b| b.session_id.0 = "\n".into()),
        ("player_id", |b| b.player_id.0.clear()),
        ("decision_node_id", |b| b.decision_node_id.0 = "\t".into()),
    ]
}

#[test]
fn each_expected_identity_rejects_blank_strings_without_generating_defaults() {
    for (field, make_blank) in blank_fields() {
        let mut expected = binding();
        make_blank(&mut expected);
        rejects(
            &source(),
            &expected,
            &request(&binding()),
            "empty_observation_identifier",
            &format!("expected.{field}"),
        );
    }
}

#[test]
fn each_request_identity_rejects_blank_strings_before_comparing_bindings() {
    for (field, make_blank) in blank_fields() {
        let expected = binding();
        let mut request = request(&expected);
        make_blank(&mut request.binding);
        rejects(
            &source(),
            &expected,
            &request,
            "empty_observation_identifier",
            &format!("binding.{field}"),
        );
    }
}

#[test]
fn expected_specification_must_match_the_supplied_source() {
    let mut expected = binding();
    expected.specification_id = GameSpecificationId("urn:other:specification".into());
    let error = rejects(
        &source(),
        &expected,
        &request(&expected),
        "observation_source_mismatch",
        "expected.specification_id",
    );
    assert_eq!(
        error,
        Error::SourceIdentityMismatch {
            expected: GameSpecificationId(source().id),
            actual: expected.specification_id
        }
    );
}

#[test]
fn requested_specification_mismatch_is_distinct_from_source_context_mismatch() {
    let expected = binding();
    let mut request = request(&expected);
    request.binding.specification_id = GameSpecificationId("urn:other:specification".into());
    rejects(
        &source(),
        &expected,
        &request,
        "observation_binding_mismatch",
        "binding.specification_id",
    );
}

#[test]
fn requested_instance_mismatch_is_explicit() {
    let expected = binding();
    let mut request = request(&expected);
    request.binding.game_instance_id = GameInstanceId("urn:other:instance".into());
    rejects(
        &source(),
        &expected,
        &request,
        "observation_binding_mismatch",
        "binding.game_instance_id",
    );
}

#[test]
fn requested_session_mismatch_is_explicit() {
    let expected = binding();
    let mut request = request(&expected);
    request.binding.session_id = GameSessionId("urn:other:session".into());
    rejects(
        &source(),
        &expected,
        &request,
        "observation_binding_mismatch",
        "binding.session_id",
    );
}

#[test]
fn requested_player_mismatch_is_explicit() {
    let expected = binding();
    let mut request = request(&expected);
    request.binding.player_id = player("opponent");
    rejects(
        &source(),
        &expected,
        &request,
        "observation_binding_mismatch",
        "binding.player_id",
    );
}

#[test]
fn requested_decision_mismatch_is_explicit_even_for_a_known_node() {
    let expected = binding();
    let mut request = request(&expected);
    request.binding.decision_node_id = node("unobserved-opponent");
    rejects(
        &source(),
        &expected,
        &request,
        "observation_binding_mismatch",
        "binding.decision_node_id",
    );
}

#[test]
fn requested_revision_mismatch_is_explicit_in_both_directions() {
    for revision in [0, 16, 18, u64::MAX] {
        let expected = binding();
        let mut request = request(&expected);
        request.binding.revision = ObservationRevision(revision);
        rejects(
            &source(),
            &expected,
            &request,
            "observation_binding_mismatch",
            "binding.revision",
        );
    }
}

#[test]
fn an_unknown_declared_decision_is_not_replaced_with_the_root() {
    let mut expected = binding();
    expected.decision_node_id = node("unknown");
    let error = rejects(
        &source(),
        &expected,
        &request(&expected),
        "unknown_observation_decision",
        "binding.decision_node_id",
    );
    assert_eq!(
        error,
        Error::UnknownDecisionNode {
            node: node("unknown")
        }
    );
}

#[test]
fn chance_nodes_do_not_acquire_policy_actions() {
    let mut expected = binding();
    expected.decision_node_id = node("unobserved-chance");
    let error = rejects(
        &source(),
        &expected,
        &request(&expected),
        "nondecision_observation_node",
        "binding.decision_node_id",
    );
    assert_eq!(
        error,
        Error::NonDecisionNode {
            node: node("unobserved-chance"),
            kind: NodeKind::Chance
        }
    );
}

#[test]
fn terminal_nodes_do_not_receive_a_synthetic_continue_action() {
    let mut expected = binding();
    expected.decision_node_id = node("unobserved-other");
    let error = rejects(
        &source(),
        &expected,
        &request(&expected),
        "nondecision_observation_node",
        "binding.decision_node_id",
    );
    assert_eq!(
        error,
        Error::NonDecisionNode {
            node: node("unobserved-other"),
            kind: NodeKind::Terminal
        }
    );
}

#[test]
fn consistently_requesting_the_wrong_player_still_fails_source_ownership() {
    let mut expected = binding();
    expected.player_id = player("opponent");
    let error = rejects(
        &source(),
        &expected,
        &request(&expected),
        "wrong_observation_player",
        "binding.player_id",
    );
    assert_eq!(
        error,
        Error::WrongPlayer {
            node: node("public-decision"),
            expected: player("acting"),
            actual: player("opponent")
        }
    );
}

#[test]
fn error_precedence_and_payloads_are_repeatable_without_mutation() {
    let source = source();
    let mut expected = binding();
    expected.specification_id.0.clear();
    expected.session_id.0.clear();
    let mut request = request(&binding());
    request.binding.player_id.0.clear();
    let before = (source.clone(), expected.clone(), request.clone());
    let first = rejects(
        &source,
        &expected,
        &request,
        "empty_observation_identifier",
        "expected.specification_id",
    );
    let second = build_policy_observation(&source, &expected, &request).unwrap_err();
    assert_eq!(first, second);
    assert_eq!(
        first.to_string(),
        "empty policy observation identifier at expected.specification_id"
    );
    assert_eq!((source, expected, request), before);
}

#[test]
fn malformed_legal_action_lists_fail_before_projection() {
    for blank in [false, true] {
        let mut source = source();
        if let Node::Decision { edges, .. } =
            source.game.nodes.get_mut(&node("public-decision")).unwrap()
        {
            edges[0].action = if blank {
                ActionId(" ".into())
            } else {
                edges[1].action.clone()
            };
        }
        let expected = binding();
        let error = build_policy_observation(&source, &expected, &request(&expected)).unwrap_err();
        assert_eq!(
            error.code(),
            if blank {
                "empty_identifier"
            } else {
                "duplicate_action"
            }
        );
    }
}
