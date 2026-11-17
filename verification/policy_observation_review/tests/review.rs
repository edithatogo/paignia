//! Independent controls for the bounded caller-declared observation projection.

use std::collections::BTreeMap;

use paignia_core::{
    build_policy_observation as build, ActionEdge, ActionId, ChanceEdge, Game, GameInstanceId,
    GameSessionId, GameSpecification, GameSpecificationId, Node, NodeId, ObservationRevision,
    PlayerId, PolicyObservation, PolicyObservationBinding as Binding,
    PolicyObservationRequest as Request, STATIC_GAME_PROFILE_V1,
};

const SHARED: &str = " https://example.test/完整/é?generation=1#role\0tail ";

fn node(id: &str) -> NodeId {
    NodeId(id.into())
}

fn fixture() -> (GameSpecification, Binding, Request) {
    let source = GameSpecification {
        schema_version: "1.0.0".into(),
        id: SHARED.into(),
        game: Game {
            root: node(SHARED),
            nodes: BTreeMap::from([
                (
                    node(SHARED),
                    Node::Decision {
                        player: PlayerId(SHARED.into()),
                        edges: vec![
                            ActionEdge {
                                action: ActionId(SHARED.into()),
                                target: node("left"),
                            },
                            ActionEdge {
                                action: ActionId("e\u{301}".into()),
                                target: node("right"),
                            },
                            ActionEdge {
                                action: ActionId("é".into()),
                                target: node("right"),
                            },
                        ],
                    },
                ),
                (
                    node("left"),
                    Node::Terminal {
                        payoffs: BTreeMap::new(),
                    },
                ),
                (
                    node("right"),
                    Node::Terminal {
                        payoffs: BTreeMap::from([(PlayerId("private-owner".into()), -0.)]),
                    },
                ),
            ]),
        },
    };
    let expected = Binding {
        specification_id: GameSpecificationId(SHARED.into()),
        game_instance_id: GameInstanceId(SHARED.into()),
        session_id: GameSessionId(SHARED.into()),
        player_id: PlayerId(SHARED.into()),
        decision_node_id: node(SHARED),
        revision: ObservationRevision(u64::MAX),
    };
    let request = Request {
        schema_version: "1.0.0".into(),
        semantic_profile: STATIC_GAME_PROFILE_V1.into(),
        binding: expected.clone(),
    };
    (source, expected, request)
}

#[test]
fn every_identity_role_and_both_unicode_spellings_remain_distinct_and_opaque() {
    let (source, expected, request) = fixture();
    let observation = build(&source, &expected, &request).unwrap();
    let PolicyObservation {
        schema_version,
        semantic_profile,
        binding,
        legal_actions,
    } = observation;
    assert_eq!(schema_version, "1.0.0");
    assert_eq!(semantic_profile, STATIC_GAME_PROFILE_V1);
    assert_eq!(binding, expected);
    assert_eq!(
        legal_actions,
        vec![
            ActionId(SHARED.into()),
            ActionId("e\u{301}".into()),
            ActionId("é".into())
        ]
    );
    // Canonically equivalent text is still an exact opaque identifier mismatch.
    let mut changed = request;
    changed.binding.session_id = GameSessionId(SHARED.replace('é', "e\u{301}"));
    let error = build(&source, &expected, &changed).unwrap_err();
    assert_eq!(error.path(), "binding.session_id");
}

#[test]
fn replacing_the_unexposed_graph_and_root_preserves_the_entire_projection() {
    let (source, expected, request) = fixture();
    let original = build(&source, &expected, &request).unwrap();
    let mut changed = source;
    changed.game.root = node("new-private-entry");
    changed.game.nodes.insert(
        node("new-private-entry"),
        Node::Chance {
            edges: vec![
                ChanceEdge {
                    probability: -0.,
                    target: node(SHARED),
                },
                ChanceEdge {
                    probability: 1.,
                    target: node("new-opponent-decision"),
                },
            ],
        },
    );
    changed.game.nodes.insert(
        node("new-opponent-decision"),
        Node::Decision {
            player: PlayerId("different-private-owner".into()),
            edges: vec![ActionEdge {
                action: ActionId(SHARED.into()),
                target: node("new-shared-terminal"),
            }],
        },
    );
    changed.game.nodes.insert(
        node("new-shared-terminal"),
        Node::Terminal {
            payoffs: BTreeMap::from([
                (PlayerId("different-private-owner".into()), f64::MAX),
                (PlayerId(SHARED.into()), f64::from_bits(1)),
            ]),
        },
    );
    changed.game.nodes.remove(&node("left"));
    changed.game.nodes.remove(&node("right"));
    if let Node::Decision { edges, .. } = changed.game.nodes.get_mut(&node(SHARED)).unwrap() {
        for edge in edges {
            edge.target = node("new-shared-terminal");
        }
    }
    let projection = build(&changed, &expected, &request).unwrap();
    assert_eq!(projection, original);
    assert!(!format!("{projection:?}").contains("new-shared-terminal"));
    assert!(!format!("{projection:?}").contains("different-private-owner"));
}

#[test]
fn simultaneous_context_mismatches_report_the_declared_order_without_repair() {
    let (source, expected, mut request) = fixture();
    request.binding.specification_id.0 = "different-specification".into();
    request.binding.game_instance_id.0 = "different-instance".into();
    request.binding.session_id.0 = "different-session".into();
    request.binding.player_id.0 = "different-player".into();
    request.binding.decision_node_id.0 = "different-node".into();
    request.binding.revision = ObservationRevision(0);
    for field in 0..6 {
        let before = request.clone();
        let error = build(&source, &expected, &request).unwrap_err();
        assert_eq!(error.code(), "observation_binding_mismatch");
        assert_eq!(
            error.path(),
            [
                "binding.specification_id",
                "binding.game_instance_id",
                "binding.session_id",
                "binding.player_id",
                "binding.decision_node_id",
                "binding.revision"
            ][field]
        );
        assert_eq!(request, before);
        match field {
            0 => request.binding.specification_id = expected.specification_id.clone(),
            1 => request.binding.game_instance_id = expected.game_instance_id.clone(),
            2 => request.binding.session_id = expected.session_id.clone(),
            3 => request.binding.player_id = expected.player_id.clone(),
            4 => request.binding.decision_node_id = expected.decision_node_id.clone(),
            5 => request.binding.revision = expected.revision,
            _ => unreachable!(),
        }
    }
    assert_eq!(
        build(&source, &expected, &request).unwrap().binding,
        expected
    );
}

#[test]
fn invalid_zero_probability_branches_are_checked_before_context_matching() {
    let (mut source, expected, mut request) = fixture();
    source.game.root = node("entry");
    source.game.nodes.insert(
        node("entry"),
        Node::Chance {
            edges: vec![
                ChanceEdge {
                    probability: 1.,
                    target: node(SHARED),
                },
                ChanceEdge {
                    probability: 0.,
                    target: node("unobserved-cycle"),
                },
            ],
        },
    );
    source.game.nodes.insert(
        node("unobserved-cycle"),
        Node::Decision {
            player: PlayerId("private-owner".into()),
            edges: vec![ActionEdge {
                action: ActionId("private-action".into()),
                target: node("unobserved-cycle"),
            }],
        },
    );
    request.binding.session_id.0 = "crossed-session".into();
    let before = (source.clone(), expected.clone(), request.clone());
    let error = build(&source, &expected, &request).unwrap_err();
    assert_eq!(error.code(), "cycle");
    assert!(error.path().starts_with("source."));
    assert_eq!(error, build(&source, &expected, &request).unwrap_err());
    assert_eq!((source.clone(), expected.clone(), request.clone()), before);
    request.semantic_profile = "paignia/extensive-information-sets/1.0.0".into();
    let error = build(&source, &expected, &request).unwrap_err();
    assert_eq!(error.code(), "unsupported_semantic_profile");
    assert_eq!(error.path(), "semantic_profile");
}

#[test]
fn revision_labels_are_consistency_checks_and_do_not_invent_a_native_clock() {
    let (source, mut expected, mut request) = fixture();
    // A request copied from a different declared revision is rejected.
    request.binding.revision = ObservationRevision(0);
    assert_eq!(
        build(&source, &expected, &request).unwrap_err().path(),
        "binding.revision"
    );
    // Two equal declarations are admissible; no live host exists to contradict them.
    expected.revision = ObservationRevision(0);
    assert_eq!(
        build(&source, &expected, &request)
            .unwrap()
            .binding
            .revision,
        ObservationRevision(0)
    );
    expected.game_instance_id.0 = "another-declared-instance".into();
    expected.session_id.0 = "another-declared-session".into();
    request.binding = expected.clone();
    assert_eq!(
        build(&source, &expected, &request).unwrap().binding,
        expected
    );
}

#[test]
fn owned_output_cannot_change_the_source_or_a_later_checked_projection() {
    fn require_owned<T: Send + Sync + 'static>(value: T) -> T {
        value
    }
    let (source, expected, request) = fixture();
    let original = build(&source, &expected, &request).unwrap();
    let mut edited = original.clone();
    edited.legal_actions.clear();
    edited.binding.revision = ObservationRevision(0);
    edited.binding.session_id.0.clear();
    assert_eq!(build(&source, &expected, &request).unwrap(), original);
    let mut changed_source = source;
    if let Node::Decision { edges, .. } = changed_source.game.nodes.get_mut(&node(SHARED)).unwrap()
    {
        edges.swap(1, 2);
    }
    let changed = build(&changed_source, &expected, &request).unwrap();
    assert_ne!(changed, original);
    assert_eq!(changed.binding, original.binding);
    assert_eq!(
        changed.legal_actions,
        vec![
            ActionId(SHARED.into()),
            ActionId("é".into()),
            ActionId("e\u{301}".into())
        ]
    );
    drop(changed_source);
    drop(expected);
    drop(request);
    let retained = require_owned(original);
    assert_eq!(retained.legal_actions.len(), 3);
    assert_ne!(edited, retained);
}
