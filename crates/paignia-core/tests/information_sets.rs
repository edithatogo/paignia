//! Independent information-set admission controls, without native storage claims.

use std::collections::BTreeMap;

use paignia_core::{
    build_information_set_graph, ActionEdge, ActionId, ChanceEdge, Game, GameSpecification,
    InformationSetDeclaration as Declaration, InformationSetDefinition as SetDeclaration,
    InformationSetError as Error, InformationSetGraph as Graph, InformationSetId,
    InformationSetNodeKind as NodeKind, Node, NodeId, PlayerId, ProfileValidationError,
    ValidationError, INFORMATION_SET_PROFILE_V1, INFORMATION_SET_SCHEMA_VERSION,
};

fn node(value: &str) -> NodeId {
    NodeId(format!("urn:paignia:test:node:{value}"))
}

fn action(value: &str) -> ActionId {
    ActionId(format!("urn:paignia:test:action:{value}"))
}

fn player(value: &str) -> PlayerId {
    PlayerId(format!("urn:paignia:test:player:{value}"))
}

fn decision(owner: &str, edges: &[(&str, &str)]) -> Node {
    Node::Decision {
        player: player(owner),
        edges: edges
            .iter()
            .map(|(label, target)| ActionEdge {
                action: action(label),
                target: node(target),
            })
            .collect(),
    }
}

fn chance(edges: &[(f64, &str)]) -> Node {
    Node::Chance {
        edges: edges
            .iter()
            .map(|(probability, target)| ChanceEdge {
                probability: *probability,
                target: node(target),
            })
            .collect(),
    }
}

fn terminal(owner: &str, payoff: f64) -> Node {
    Node::Terminal {
        payoffs: BTreeMap::from([(player(owner), payoff)]),
    }
}

fn specification(root: &str, nodes: Vec<(&str, Node)>) -> GameSpecification {
    GameSpecification {
        schema_version: "1.0.0".into(),
        id: "urn:paignia:test:specification:information-sets".into(),
        game: Game {
            root: node(root),
            nodes: nodes
                .into_iter()
                .map(|(id, value)| (node(id), value))
                .collect(),
        },
    }
}

fn set(id: &str, owner: &str, members: &[&str]) -> SetDeclaration {
    SetDeclaration {
        id: InformationSetId(format!("urn:paignia:test:set:{id}")),
        player: player(owner),
        members: members.iter().map(|member| node(member)).collect(),
    }
}

fn declared(sets: Vec<SetDeclaration>) -> Declaration {
    Declaration {
        schema_version: INFORMATION_SET_SCHEMA_VERSION.into(),
        semantic_profile: INFORMATION_SET_PROFILE_V1.into(),
        sets,
    }
}

fn accept(source: &GameSpecification, declaration: &Declaration) -> Graph {
    build_information_set_graph(source, declaration).expect("declaration should be accepted")
}

fn reject(source: &GameSpecification, declaration: &Declaration) -> Error {
    build_information_set_graph(source, declaration).expect_err("declaration should be rejected")
}

fn perfect_information() -> (GameSpecification, Declaration) {
    let source = specification(
        "root",
        vec![
            (
                "root",
                decision("alice", &[("left", "after-left"), ("right", "after-right")]),
            ),
            (
                "after-left",
                decision("alice", &[("end-left", "left-terminal")]),
            ),
            (
                "after-right",
                decision("bob", &[("end-right", "right-terminal")]),
            ),
            ("left-terminal", terminal("alice", 4.0)),
            ("right-terminal", terminal("bob", 7.0)),
        ],
    );
    let declaration = declared(vec![
        set("root", "alice", &["root"]),
        set("after-left", "alice", &["after-left"]),
        set("after-right", "bob", &["after-right"]),
    ]);
    (source, declaration)
}

fn simultaneous() -> (GameSpecification, Declaration) {
    let source = specification(
        "root",
        vec![
            (
                "root",
                decision("alice", &[("a", "after-a"), ("b", "after-b")]),
            ),
            (
                "after-a",
                decision("bob", &[("left", "a-left"), ("right", "a-right")]),
            ),
            (
                "after-b",
                decision("bob", &[("left", "b-left"), ("right", "b-right")]),
            ),
            ("a-left", terminal("alice", 1.0)),
            ("a-right", terminal("alice", 2.0)),
            ("b-left", terminal("bob", 3.0)),
            ("b-right", terminal("bob", 4.0)),
        ],
    );
    let declaration = declared(vec![
        set("alice-first", "alice", &["root"]),
        set("bob-unobserved", "bob", &["after-b", "after-a"]),
    ]);
    (source, declaration)
}

fn imperfect_recall() -> (GameSpecification, Declaration) {
    let source = specification(
        "root",
        vec![
            (
                "root",
                decision("alice", &[("a", "after-a"), ("b", "after-b")]),
            ),
            ("after-a", decision("alice", &[("x", "a-x"), ("y", "a-y")])),
            ("after-b", decision("alice", &[("x", "b-x"), ("y", "b-y")])),
            ("a-x", terminal("alice", 1.0)),
            ("a-y", terminal("alice", 2.0)),
            ("b-x", terminal("alice", 3.0)),
            ("b-y", terminal("alice", 4.0)),
        ],
    );
    let declaration = declared(vec![
        set("first", "alice", &["root"]),
        set("forgotten", "alice", &["after-a", "after-b"]),
    ]);
    (source, declaration)
}

fn chance_merge() -> (GameSpecification, Declaration) {
    let source = specification(
        "root",
        vec![
            (
                "root",
                decision("alice", &[("a", "chance"), ("b", "chance")]),
            ),
            ("chance", chance(&[(0.5, "after-a"), (0.5, "after-b")])),
            (
                "after-a",
                decision("bob", &[("left", "a-left"), ("right", "a-right")]),
            ),
            (
                "after-b",
                decision("bob", &[("left", "b-left"), ("right", "b-right")]),
            ),
            ("a-left", terminal("alice", 1.0)),
            ("a-right", terminal("alice", 2.0)),
            ("b-left", terminal("bob", 3.0)),
            ("b-right", terminal("bob", 4.0)),
        ],
    );
    let declaration = declared(vec![
        set("alice-first", "alice", &["root"]),
        set("bob-unobserved", "bob", &["after-a", "after-b"]),
    ]);
    (source, declaration)
}

fn consistent_dag_merge() -> (GameSpecification, Declaration) {
    let source = specification(
        "root",
        vec![
            ("root", chance(&[(0.5, "first-a"), (0.5, "first-b")])),
            ("first-a", decision("alice", &[("same", "merged")])),
            ("first-b", decision("alice", &[("same", "merged")])),
            ("merged", decision("alice", &[("finish", "terminal")])),
            ("terminal", terminal("alice", 9.0)),
        ],
    );
    let declaration = declared(vec![
        set("first", "alice", &["first-a", "first-b"]),
        set("second", "alice", &["merged"]),
    ]);
    (source, declaration)
}

#[test]
fn accepts_perfect_information_singletons() {
    let (source, declaration) = perfect_information();
    let graph = accept(&source, &declaration);

    assert_eq!(graph.schema_version, INFORMATION_SET_SCHEMA_VERSION);
    assert_eq!(graph.semantic_profile, INFORMATION_SET_PROFILE_V1);
    assert_eq!(graph.specification_id.0, source.id);
    assert_eq!(graph.sets.len(), 3);
    assert_eq!(graph.assignments.len(), 3);

    let root = &graph.sets[&InformationSetId("urn:paignia:test:set:root".into())];
    assert_eq!(root.player, player("alice"));
    assert_eq!(root.members, vec![node("root")]);
    assert_eq!(root.legal_actions, vec![action("left"), action("right")]);
    assert_eq!(
        graph.assignments[&node("after-right")],
        InformationSetId("urn:paignia:test:set:after-right".into())
    );
}

#[test]
fn accepts_shared_information_set_with_identical_actions() {
    let (source, declaration) = simultaneous();
    let graph = accept(&source, &declaration);

    let unobserved = &graph.sets[&InformationSetId("urn:paignia:test:set:bob-unobserved".into())];
    assert_eq!(unobserved.player, player("bob"));
    assert_eq!(unobserved.members, vec![node("after-a"), node("after-b")]);
    assert_eq!(
        unobserved.legal_actions,
        vec![action("left"), action("right")]
    );
    assert_eq!(
        graph.assignments[&node("after-a")],
        InformationSetId("urn:paignia:test:set:bob-unobserved".into())
    );
}

#[test]
fn accepts_distinct_information_sets_for_remembered_actions() {
    let source = specification(
        "root",
        vec![
            (
                "root",
                decision("alice", &[("a", "after-a"), ("b", "after-b")]),
            ),
            ("after-a", decision("alice", &[("same", "a-terminal")])),
            ("after-b", decision("alice", &[("same", "b-terminal")])),
            ("a-terminal", terminal("alice", 1.0)),
            ("b-terminal", terminal("alice", 2.0)),
        ],
    );
    let declaration = declared(vec![
        set("first", "alice", &["root"]),
        set("remembered-a", "alice", &["after-a"]),
        set("remembered-b", "alice", &["after-b"]),
    ]);

    let graph = accept(&source, &declaration);
    assert_eq!(graph.sets.len(), 3);
}

#[test]
fn accepts_chance_merge_without_over_rejecting_unused_history() {
    let (source, declaration) = chance_merge();
    let graph = accept(&source, &declaration);

    assert_eq!(
        graph.sets[&InformationSetId("urn:paignia:test:set:bob-unobserved".into())].members,
        vec![node("after-a"), node("after-b")]
    );
}

#[test]
fn accepts_consistent_dag_merge() {
    let (source, declaration) = consistent_dag_merge();
    let graph = accept(&source, &declaration);

    assert_eq!(
        graph.sets[&InformationSetId("urn:paignia:test:set:first".into())].members,
        vec![node("first-a"), node("first-b")]
    );
    assert_eq!(
        graph.assignments[&node("merged")],
        InformationSetId("urn:paignia:test:set:second".into())
    );
}

#[test]
fn accepts_merge_after_different_opponent_actions() {
    let source = specification(
        "root",
        vec![
            (
                "root",
                decision("bob", &[("left", "after-left"), ("right", "after-right")]),
            ),
            ("after-left", decision("alice", &[("same", "merged")])),
            ("after-right", decision("alice", &[("same", "merged")])),
            ("merged", decision("alice", &[("finish", "terminal")])),
            ("terminal", terminal("alice", 1.0)),
        ],
    );
    let declaration = declared(vec![
        set("bob-first", "bob", &["root"]),
        set("alice-unobserved", "alice", &["after-left", "after-right"]),
        set("alice-later", "alice", &["merged"]),
    ]);

    let graph = accept(&source, &declaration);
    assert_eq!(
        graph.assignments[&node("merged")],
        InformationSetId("urn:paignia:test:set:alice-later".into())
    );
}

#[test]
fn accepts_a_game_without_decision_nodes() {
    let source = specification("root", vec![("root", terminal("alice", 0.0))]);
    let graph = accept(&source, &declared(Vec::new()));

    assert!(graph.sets.is_empty());
    assert!(graph.assignments.is_empty());
}

#[test]
fn member_and_set_order_do_not_change_the_result() {
    let (source, first) = simultaneous();
    let mut second = declared(vec![
        set("bob-unobserved", "bob", &["after-a", "after-b"]),
        set("alice-first", "alice", &["root"]),
    ]);
    second.sets.reverse();
    second.sets[0].members.reverse();

    assert_eq!(accept(&source, &first), accept(&source, &second));
}

#[test]
fn preserves_opaque_identifiers_and_declared_action_order() {
    let opaque_player = PlayerId("https://example.test/player/üñïçø∂é".into());
    let opaque_node = NodeId("https://example.test/node/a b?x=1#fragment".into());
    let opaque_set = InformationSetId("https://example.test/set/ ünïcode ".into());
    let zeta = NodeId("urn:terminal:zeta".into());
    let alpha = NodeId("urn:terminal:alpha".into());
    let source = GameSpecification {
        schema_version: "1.0.0".into(),
        id: "https://example.test/specification/opaque".into(),
        game: Game {
            root: opaque_node.clone(),
            nodes: BTreeMap::from([
                (
                    opaque_node.clone(),
                    Node::Decision {
                        player: opaque_player.clone(),
                        edges: vec![
                            ActionEdge {
                                action: ActionId("urn:action:zeta".into()),
                                target: zeta.clone(),
                            },
                            ActionEdge {
                                action: ActionId("urn:action:alpha".into()),
                                target: alpha.clone(),
                            },
                        ],
                    },
                ),
                (
                    zeta,
                    Node::Terminal {
                        payoffs: BTreeMap::from([(opaque_player.clone(), 1.0)]),
                    },
                ),
                (
                    alpha,
                    Node::Terminal {
                        payoffs: BTreeMap::from([(opaque_player.clone(), 2.0)]),
                    },
                ),
            ]),
        },
    };
    let declaration = Declaration {
        schema_version: INFORMATION_SET_SCHEMA_VERSION.into(),
        semantic_profile: INFORMATION_SET_PROFILE_V1.into(),
        sets: vec![SetDeclaration {
            id: opaque_set.clone(),
            player: opaque_player,
            members: vec![source.game.root.clone()],
        }],
    };

    let graph = accept(&source, &declaration);
    assert!(graph.sets.contains_key(&opaque_set));
    assert_eq!(
        graph.sets[&opaque_set].legal_actions,
        vec![
            ActionId("urn:action:zeta".into()),
            ActionId("urn:action:alpha".into())
        ]
    );
    assert_eq!(graph.assignments[&source.game.root], opaque_set);
}

#[test]
fn rejects_unsupported_schema_version() {
    let (source, mut declaration) = perfect_information();
    declaration.schema_version = "2.0.0".into();
    let error = reject(&source, &declaration);

    assert_eq!(
        error,
        Error::UnsupportedSchemaVersion {
            value: "2.0.0".into()
        }
    );
    assert_eq!(error.code(), "unsupported_information_set_schema");
    assert_eq!(error.path(), "schema_version");
}

#[test]
fn rejects_unsupported_semantic_profile() {
    let (source, mut declaration) = perfect_information();
    declaration.semantic_profile = "paignia/static-finite-information-set/2.0.0".into();
    let error = reject(&source, &declaration);

    assert_eq!(
        error,
        Error::UnsupportedSemanticProfile {
            value: "paignia/static-finite-information-set/2.0.0".into()
        }
    );
    assert_eq!(error.code(), "unsupported_information_set_profile");
    assert_eq!(error.path(), "semantic_profile");
}

#[test]
fn rejects_invalid_source_before_declaration_checks() {
    let source = specification(
        "root",
        vec![("root", decision("alice", &[("again", "root")]))],
    );
    let declaration = declared(vec![set(" ", "alice", &["root"])]);
    let error = reject(&source, &declaration);

    assert!(matches!(
        error,
        Error::InvalidSource(ProfileValidationError::InvalidSpecification(
            ValidationError::Cycle(_)
        ))
    ));
    assert_eq!(error.code(), "cycle");
    assert!(error.path().starts_with("source.game.nodes."));
}

#[test]
fn rejects_blank_information_set_identifier() {
    let (source, declaration) = perfect_information();
    let mut declaration = declaration;
    declaration.sets[1].id = InformationSetId("   ".into());
    let error = reject(&source, &declaration);

    assert_eq!(error, Error::EmptyIdentifier { path: "sets.id" });
    assert_eq!(error.code(), "empty_information_set_identifier");
    assert_eq!(error.path(), "sets.id");
}

#[test]
fn rejects_blank_declared_player() {
    let (source, declaration) = perfect_information();
    let mut declaration = declaration;
    declaration.sets[0].player = PlayerId(" ".into());
    let error = reject(&source, &declaration);

    assert_eq!(
        error,
        Error::EmptyIdentifier {
            path: "sets.player"
        }
    );
}

#[test]
fn rejects_blank_member_identifier() {
    let (source, declaration) = perfect_information();
    let mut declaration = declaration;
    declaration.sets[0].members = vec![NodeId("\t".into())];
    let error = reject(&source, &declaration);

    assert_eq!(
        error,
        Error::EmptyIdentifier {
            path: "sets.members"
        }
    );
}

#[test]
fn rejects_duplicate_information_set_identifier() {
    let (source, declaration) = perfect_information();
    let mut declaration = declaration;
    declaration.sets.push(set("root", "alice", &["after-left"]));
    let error = reject(&source, &declaration);

    assert_eq!(
        error,
        Error::DuplicateInformationSet(InformationSetId("urn:paignia:test:set:root".into()))
    );
    assert_eq!(error.code(), "duplicate_information_set");
    assert_eq!(error.path(), "sets.urn:paignia:test:set:root.id");
}

#[test]
fn rejects_empty_information_set() {
    let (source, declaration) = perfect_information();
    let mut declaration = declaration;
    declaration.sets[0].members.clear();
    let error = reject(&source, &declaration);

    assert_eq!(
        error,
        Error::EmptyInformationSet(InformationSetId("urn:paignia:test:set:root".into()))
    );
    assert_eq!(error.code(), "empty_information_set");
    assert_eq!(error.path(), "sets.urn:paignia:test:set:root.members");
}

#[test]
fn rejects_duplicate_member_within_one_set() {
    let (source, declaration) = perfect_information();
    let mut declaration = declaration;
    declaration.sets[0].members = vec![node("root"), node("root")];
    let error = reject(&source, &declaration);

    assert_eq!(
        error,
        Error::DuplicateMember {
            set: InformationSetId("urn:paignia:test:set:root".into()),
            node: node("root"),
        }
    );
    assert_eq!(error.code(), "duplicate_information_set_member");
}

#[test]
fn rejects_unknown_member_node() {
    let (source, declaration) = perfect_information();
    let mut declaration = declaration;
    declaration.sets[0].members = vec![node("absent")];
    let error = reject(&source, &declaration);

    assert_eq!(
        error,
        Error::UnknownNode {
            set: InformationSetId("urn:paignia:test:set:root".into()),
            node: node("absent"),
        }
    );
    assert_eq!(error.code(), "unknown_information_set_member");
}

#[test]
fn rejects_chance_member_node() {
    let (source, declaration) = chance_merge();
    let mut declaration = declaration;
    declaration.sets.push(set("chance", "bob", &["chance"]));
    let error = reject(&source, &declaration);

    assert_eq!(
        error,
        Error::NonDecisionNode {
            set: InformationSetId("urn:paignia:test:set:chance".into()),
            node: node("chance"),
            kind: NodeKind::Chance,
        }
    );
    assert_eq!(error.code(), "nondecision_information_set_member");
}

#[test]
fn rejects_terminal_member_node() {
    let (source, declaration) = perfect_information();
    let mut declaration = declaration;
    declaration
        .sets
        .push(set("terminal", "alice", &["left-terminal"]));
    let error = reject(&source, &declaration);

    assert_eq!(
        error,
        Error::NonDecisionNode {
            set: InformationSetId("urn:paignia:test:set:terminal".into()),
            node: node("left-terminal"),
            kind: NodeKind::Terminal,
        }
    );
}

#[test]
fn rejects_declared_player_mismatch() {
    let (source, declaration) = perfect_information();
    let mut declaration = declaration;
    declaration.sets[2].player = player("alice");
    let error = reject(&source, &declaration);

    assert_eq!(
        error,
        Error::PlayerMismatch {
            set: InformationSetId("urn:paignia:test:set:after-right".into()),
            node: node("after-right"),
            expected: player("alice"),
            actual: player("bob"),
        }
    );
    assert_eq!(error.code(), "information_set_player_mismatch");
}

#[test]
fn rejects_unassigned_decision_node() {
    let (source, declaration) = perfect_information();
    let mut declaration = declaration;
    declaration.sets.remove(2);
    let error = reject(&source, &declaration);

    assert_eq!(error, Error::UnassignedDecisionNode(node("after-right")));
    assert_eq!(error.code(), "unassigned_information_set_decision");
    assert_eq!(error.path(), "game.nodes.urn:paignia:test:node:after-right");
}

#[test]
fn rejects_duplicate_membership_across_sets() {
    let (source, declaration) = perfect_information();
    let mut declaration = declaration;
    declaration
        .sets
        .push(set("overlap", "alice", &["after-left"]));
    let error = reject(&source, &declaration);

    assert_eq!(
        error,
        Error::DuplicateMembership {
            node: node("after-left"),
            first: InformationSetId("urn:paignia:test:set:after-left".into()),
            second: InformationSetId("urn:paignia:test:set:overlap".into()),
        }
    );
    assert_eq!(error.code(), "duplicate_information_set_membership");
    assert_eq!(error.path(), "game.nodes.urn:paignia:test:node:after-left");
}

#[test]
fn rejects_different_action_identifiers_within_a_set() {
    let (source, declaration) = simultaneous();
    let mut source = source;
    source.game.nodes.insert(
        node("after-b"),
        decision("bob", &[("left", "b-left"), ("up", "b-right")]),
    );
    let error = reject(&source, &declaration);

    assert_eq!(
        error,
        Error::ActionMismatch {
            set: InformationSetId("urn:paignia:test:set:bob-unobserved".into()),
            node: node("after-b"),
            reference: node("after-a"),
        }
    );
    assert_eq!(error.code(), "information_set_action_mismatch");
}

#[test]
fn rejects_different_action_order_within_a_set() {
    let (source, declaration) = simultaneous();
    let mut source = source;
    source.game.nodes.insert(
        node("after-b"),
        decision("bob", &[("right", "b-right"), ("left", "b-left")]),
    );
    let error = reject(&source, &declaration);

    assert!(matches!(error, Error::ActionMismatch { .. }));
}

#[test]
fn rejects_imperfect_recall_after_own_action() {
    let (source, declaration) = imperfect_recall();
    let error = reject(&source, &declaration);

    assert_eq!(
        error,
        Error::ImperfectRecall {
            set: InformationSetId("urn:paignia:test:set:forgotten".into()),
            node: node("after-b"),
            reference: node("after-a"),
        }
    );
    assert_eq!(error.code(), "imperfect_recall");
    assert_eq!(
        error.path(),
        "sets.urn:paignia:test:set:forgotten.members.urn:paignia:test:node:after-b"
    );
}

#[test]
fn rejects_ambiguous_history_from_distinct_prior_information_sets() {
    let source = specification(
        "root",
        vec![
            ("root", chance(&[(0.5, "first-a"), (0.5, "first-b")])),
            ("first-a", decision("alice", &[("same", "merged")])),
            ("first-b", decision("alice", &[("same", "merged")])),
            ("merged", decision("alice", &[("finish", "terminal")])),
            ("terminal", terminal("alice", 9.0)),
        ],
    );
    let declaration = declared(vec![
        set("first-a", "alice", &["first-a"]),
        set("first-b", "alice", &["first-b"]),
        set("merged", "alice", &["merged"]),
    ]);
    let error = reject(&source, &declaration);

    assert_eq!(
        error,
        Error::AmbiguousRecall {
            node: node("merged"),
            player: player("alice"),
        }
    );
    assert_eq!(error.code(), "ambiguous_recall_history");
}

#[test]
fn rejects_ambiguous_history_at_a_shared_decision_node() {
    let source = specification(
        "root",
        vec![
            (
                "root",
                decision("alice", &[("a", "merged"), ("b", "merged")]),
            ),
            ("merged", decision("alice", &[("finish", "terminal")])),
            ("terminal", terminal("alice", 9.0)),
        ],
    );
    let declaration = declared(vec![
        set("first", "alice", &["root"]),
        set("second", "alice", &["merged"]),
    ]);
    let error = reject(&source, &declaration);

    assert_eq!(
        error,
        Error::AmbiguousRecall {
            node: node("merged"),
            player: player("alice"),
        }
    );
}

#[test]
fn failure_precedence_is_schema_then_profile_then_source_then_declaration() {
    let invalid_source = specification(
        "root",
        vec![("root", decision("alice", &[("again", "root")]))],
    );
    let mut declaration = declared(vec![set(" ", "alice", &["root"])]);
    declaration.schema_version = "9.0.0".into();
    declaration.semantic_profile = "urn:unknown:profile".into();
    assert!(matches!(
        reject(&invalid_source, &declaration),
        Error::UnsupportedSchemaVersion { .. }
    ));

    declaration.schema_version = INFORMATION_SET_SCHEMA_VERSION.into();
    assert!(matches!(
        reject(&invalid_source, &declaration),
        Error::UnsupportedSemanticProfile { .. }
    ));

    declaration.semantic_profile = INFORMATION_SET_PROFILE_V1.into();
    assert!(matches!(
        reject(&invalid_source, &declaration),
        Error::InvalidSource(_)
    ));
}

#[test]
fn failures_are_repeatable_and_do_not_mutate_inputs() {
    let (source, declaration) = imperfect_recall();
    let source_before = source.clone();
    let declaration_before = declaration.clone();

    let first = reject(&source, &declaration);
    let second = reject(&source, &declaration);

    assert_eq!(first, second);
    assert_eq!(source, source_before);
    assert_eq!(declaration, declaration_before);
}

#[test]
fn deep_games_are_validated_iteratively() {
    let depth = 5_000;
    let mut nodes = BTreeMap::new();
    let mut sets = Vec::with_capacity(depth);
    for index in 0..depth {
        let this = node(&format!("n{index}"));
        let next = node(&format!("n{}", index + 1));
        nodes.insert(
            this.clone(),
            Node::Decision {
                player: player("alice"),
                edges: vec![ActionEdge {
                    action: action("next"),
                    target: next,
                }],
            },
        );
        sets.push(SetDeclaration {
            id: InformationSetId(format!("urn:paignia:test:set:n{index}")),
            player: player("alice"),
            members: vec![this],
        });
    }
    nodes.insert(node(&format!("n{depth}")), terminal("alice", 1.0));
    let source = GameSpecification {
        schema_version: "1.0.0".into(),
        id: "urn:paignia:test:specification:information-sets".into(),
        game: Game {
            root: node("n0"),
            nodes,
        },
    };
    let declaration = declared(sets);

    let graph = accept(&source, &declaration);
    assert_eq!(graph.sets.len(), depth);
    assert_eq!(graph.assignments.len(), depth);
}

#[test]
fn output_records_the_exact_source_specification_identity() {
    let (mut source, declaration) = perfect_information();
    source.id = "https://example.test/specification/other".into();
    let graph = accept(&source, &declaration);

    assert_eq!(
        graph.specification_id.0,
        "https://example.test/specification/other"
    );
    assert_eq!(graph.semantic_profile, declaration.semantic_profile);
    assert_eq!(graph.schema_version, declaration.schema_version);
}

#[test]
fn graph_is_not_mutated_or_shared_with_the_declaration() {
    let (source, declaration) = perfect_information();
    let mut graph = accept(&source, &declaration);
    graph.sets.clear();
    graph.assignments.clear();

    assert_eq!(accept(&source, &declaration).sets.len(), 3);
    assert_eq!(declaration.sets[0].members, vec![node("root")]);
}
