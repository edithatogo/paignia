//! T17.01.profile: native admission controls for the existing static input model.

use std::collections::BTreeMap;
use std::error::Error;

use paignia_core::{
    ActionEdge, ActionId, ChanceEdge, Game, GameSpecification, Node, NodeId, PlayerId,
    ProfileValidationError, ValidationError, STATIC_GAME_PROFILE_V1,
};

// Independent literal pins: changing a production constant must not silently
// change which contract these fixtures ask the implementation to admit.
const PROFILE: &str = "paignia/static-finite-perfect-information-sparse-zero/1.0.0";

fn terminal_specification() -> GameSpecification {
    let root = NodeId("https://w3id.org/uogto/core#terminal-fixture".into());
    GameSpecification {
        schema_version: "1.0.0".into(),
        id: "urn:paignia:test:static-specification".into(),
        game: Game {
            root: root.clone(),
            nodes: BTreeMap::from([(
                root,
                Node::Terminal {
                    payoffs: BTreeMap::from([(PlayerId("player".into()), 2.0)]),
                },
            )]),
        },
    }
}

fn chance_specification(probabilities: &[f64]) -> GameSpecification {
    let root = NodeId("chance".into());
    let mut nodes = BTreeMap::new();
    let mut edges = Vec::new();
    for (index, probability) in probabilities.iter().enumerate() {
        let target = NodeId(format!("outcome-{index}"));
        edges.push(ChanceEdge {
            probability: *probability,
            target: target.clone(),
        });
        nodes.insert(
            target,
            Node::Terminal {
                payoffs: BTreeMap::new(),
            },
        );
    }
    nodes.insert(root.clone(), Node::Chance { edges });
    GameSpecification {
        schema_version: "1.0.0".into(),
        id: "urn:paignia:test:chance-specification".into(),
        game: Game { root, nodes },
    }
}

#[test]
fn profile_identifier_is_pinned_independently() {
    assert_eq!(STATIC_GAME_PROFILE_V1, PROFILE);
}

#[test]
fn explicit_profile_admits_the_seed_without_mutation() {
    let specification = terminal_specification();
    let before = specification.clone();
    assert_eq!(specification.validate_for_profile(PROFILE), Ok(()));
    assert_eq!(specification, before);
    // Existing callers do not have to construct a new field or error type.
    assert_eq!(specification.validate(), Ok(()));
}

#[test]
fn unknown_profile_versions_are_not_accepted_as_the_current_profile() {
    let specification = terminal_specification();
    for profile in [
        "paignia/static-finite-perfect-information-sparse-zero/0.9.0",
        "paignia/static-finite-perfect-information-sparse-zero/1.0.1",
        "paignia/static-finite-perfect-information-sparse-zero/2.0.0",
    ] {
        let error = specification.validate_for_profile(profile).unwrap_err();
        assert_eq!(
            error,
            ProfileValidationError::UnsupportedSemanticProfile {
                value: profile.into(),
            }
        );
        assert_eq!(error.code(), "unsupported_semantic_profile");
        assert_eq!(error.path(), "semantic_profile");
        assert!(error.to_string().contains(profile));
        assert!(error.source().is_none());
    }
}

#[test]
fn other_representations_do_not_acquire_support_from_a_profile_name() {
    let specification = terminal_specification();
    for profile in [
        "paignia/static-normal-form/1.0.0",
        "paignia/static-imperfect-information/1.0.0",
        "paignia/static-finite-perfect-information-complete-payoffs/1.0.0",
        "https://w3id.org/uogto/core#Game",
        "https://kairos.dev/ontology/game-theory#Game",
    ] {
        assert!(matches!(
            specification.validate_for_profile(profile),
            Err(ProfileValidationError::UnsupportedSemanticProfile { value }) if value == profile
        ));
    }
}

#[test]
fn profile_strings_are_exact_and_not_normalized() {
    let specification = terminal_specification();
    for profile in [
        String::new(),
        "1.0.0".into(),
        format!(" {PROFILE}"),
        format!("{PROFILE} "),
        PROFILE.to_uppercase(),
        format!("{PROFILE}\0"),
    ] {
        assert_eq!(
            specification.validate_for_profile(&profile),
            Err(ProfileValidationError::UnsupportedSemanticProfile { value: profile })
        );
    }
}

#[test]
fn known_profile_still_requires_the_exact_specification_schema() {
    for schema in ["", "1", "1.0", "1.0.1", "2.0.0", " 1.0.0"] {
        let mut specification = terminal_specification();
        specification.schema_version = schema.into();
        assert_eq!(
            specification.validate_for_profile(PROFILE),
            Err(ProfileValidationError::InvalidSpecification(
                ValidationError::UnsupportedSchemaVersion {
                    path: "schema_version".into(),
                    value: schema.into(),
                }
            ))
        );
    }
}

#[test]
fn unsupported_profile_is_rejected_before_schema_or_graph_validation() {
    let mut specification = terminal_specification();
    specification.schema_version = "future-schema".into();
    specification.game.nodes.clear();
    assert_eq!(
        specification.validate_for_profile("future-profile"),
        Err(ProfileValidationError::UnsupportedSemanticProfile {
            value: "future-profile".into(),
        })
    );
}

#[test]
fn known_profile_does_not_bypass_graph_validation() {
    let mut specification = terminal_specification();
    specification.game.nodes.clear();
    assert_eq!(
        specification.validate_for_profile(PROFILE),
        Err(ProfileValidationError::InvalidSpecification(
            ValidationError::MissingRoot(specification.game.root.clone())
        ))
    );
    let root = specification.game.root.clone();
    specification.game.nodes.insert(
        root.clone(),
        Node::Chance {
            edges: vec![ChanceEdge {
                probability: 1.0,
                target: root.clone(),
            }],
        },
    );
    assert_eq!(
        specification.validate_for_profile(PROFILE),
        Err(ProfileValidationError::InvalidSpecification(
            ValidationError::Cycle(root)
        ))
    );
}

#[test]
fn zero_probability_edges_are_retained_and_their_targets_are_validated() {
    let mut specification = chance_specification(&[1.0, 0.0]);
    let before = specification.clone();
    assert_eq!(specification.validate_for_profile(PROFILE), Ok(()));
    assert_eq!(specification, before);
    let zero_target = NodeId("outcome-1".into());
    specification.game.nodes.remove(&zero_target);
    assert_eq!(
        specification.validate_for_profile(PROFILE),
        Err(ProfileValidationError::InvalidSpecification(
            ValidationError::MissingTarget {
                from: specification.game.root.clone(),
                target: zero_target,
            }
        ))
    );
}

#[test]
fn existing_mass_tolerance_is_pinned_without_normalization() {
    // Exact binary fractions avoid decimal rounding at the 1e-9 threshold.
    let within = 2.0_f64.powi(-31);
    let outside = 2.0_f64.powi(-29);
    for mass in [1.0 - within, 1.0 + within] {
        let specification = chance_specification(&[mass]);
        let before = specification.clone();
        assert_eq!(specification.validate_for_profile(PROFILE), Ok(()));
        assert_eq!(specification, before);
    }
    for mass in [0.0, 1.0 - outside, 1.0 + outside] {
        let specification = chance_specification(&[mass]);
        assert_eq!(
            specification.validate_for_profile(PROFILE),
            Err(ProfileValidationError::InvalidSpecification(
                ValidationError::ProbabilityMass(specification.game.root.clone())
            ))
        );
    }
}

#[test]
fn invalid_chance_weights_are_rejected_under_the_profile() {
    for probability in [-f64::EPSILON, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let specification = chance_specification(&[probability, 1.0]);
        assert_eq!(
            specification.validate_for_profile(PROFILE),
            Err(ProfileValidationError::InvalidSpecification(
                ValidationError::InvalidProbability(specification.game.root.clone())
            ))
        );
    }
}

#[test]
fn intentional_sparse_payoffs_are_admitted_without_inserting_zero_entries() {
    let mut specification = terminal_specification();
    let terminal = specification.game.root.clone();
    let root = NodeId("decision".into());
    specification.game.nodes.insert(
        root.clone(),
        Node::Decision {
            player: PlayerId("absent-from-terminal-payoffs".into()),
            edges: vec![ActionEdge {
                action: ActionId("choose".into()),
                target: terminal.clone(),
            }],
        },
    );
    specification.game.root = root;
    let before = specification.clone();
    assert_eq!(specification.validate_for_profile(PROFILE), Ok(()));
    assert_eq!(specification, before);
    specification.game.nodes.insert(
        terminal,
        Node::Terminal {
            payoffs: BTreeMap::new(),
        },
    );
    assert_eq!(specification.validate_for_profile(PROFILE), Ok(()));
}

#[test]
fn non_finite_payoffs_are_not_treated_as_sparse_zero() {
    for payoff in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut specification = terminal_specification();
        let player = PlayerId("player".into());
        specification.game.nodes.insert(
            specification.game.root.clone(),
            Node::Terminal {
                payoffs: BTreeMap::from([(player.clone(), payoff)]),
            },
        );
        assert_eq!(
            specification.validate_for_profile(PROFILE),
            Err(ProfileValidationError::InvalidSpecification(
                ValidationError::InvalidPayoff {
                    node: specification.game.root.clone(),
                    player,
                }
            ))
        );
    }
}

#[test]
fn full_namespace_identities_survive_without_aliasing_or_rewriting() {
    let mut specification = terminal_specification();
    let terminal = specification.game.root.clone();
    let root = NodeId("https://w3id.org/uogto/extensions#decision-fixture".into());
    let actions = [
        "https://w3id.org/uogto/core#same-local-name",
        "https://w3id.org/uogto/extensions#same-local-name",
        "https://kairos.dev/ontology/game-theory#same-local-name",
    ];
    specification.game.nodes.insert(
        root.clone(),
        Node::Decision {
            player: PlayerId("urn:paignia:test:player".into()),
            edges: actions
                .map(|action| ActionEdge {
                    action: ActionId(action.into()),
                    target: terminal.clone(),
                })
                .into(),
        },
    );
    specification.game.root = root;
    let before = specification.clone();
    assert_eq!(specification.validate_for_profile(PROFILE), Ok(()));
    assert_eq!(specification, before);
}

#[test]
fn wrapped_errors_retain_the_original_code_path_and_error_source() {
    let mut specification = terminal_specification();
    specification.id = "  ".into();
    let original = specification.validate().unwrap_err();
    let wrapped = specification.validate_for_profile(PROFILE).unwrap_err();
    assert_eq!(wrapped.code(), "empty_identifier");
    assert_eq!(wrapped.path(), "id");
    assert_eq!(
        wrapped.source().unwrap().downcast_ref::<ValidationError>(),
        Some(&original)
    );
    assert_eq!(
        wrapped,
        ProfileValidationError::InvalidSpecification(original)
    );
}

// Retained from accepted independent review 5909550432 at PR14 head
// 0c88607a142508d1fe4f1153b963fd1411b49395. The external harness remains
// the independent evidence; these are native regression controls, not a new review.
mod independent_review_controls {
    use paignia_core::{
        ActionEdge, ActionId, ChanceEdge, Game, GameSpecification, Node, NodeId, PlayerId,
        ProfileValidationError, ValidationError,
    };
    use std::collections::BTreeMap;
    use std::error::Error;

    const PROFILE: &str = "paignia/static-finite-perfect-information-sparse-zero/1.0.0";
    fn nid(s: &str) -> NodeId {
        NodeId(s.into())
    }
    fn pid(s: &str) -> PlayerId {
        PlayerId(s.into())
    }
    fn terminal() -> Node {
        Node::Terminal {
            payoffs: BTreeMap::new(),
        }
    }
    fn specification(root: &str, nodes: Vec<(&str, Node)>) -> GameSpecification {
        GameSpecification {
            schema_version: "1.0.0".into(),
            id: "review:synthetic".into(),
            game: Game {
                root: nid(root),
                nodes: nodes.into_iter().map(|(k, v)| (nid(k), v)).collect(),
            },
        }
    }
    fn decision(edges: &[(&str, &str)]) -> Node {
        Node::Decision {
            player: pid("p"),
            edges: edges
                .iter()
                .map(|(a, t)| ActionEdge {
                    action: ActionId((*a).into()),
                    target: nid(t),
                })
                .collect(),
        }
    }
    fn chance(weights: &[f64]) -> GameSpecification {
        specification(
            "c",
            vec![
                (
                    "c",
                    Node::Chance {
                        edges: weights
                            .iter()
                            .map(|p| ChanceEdge {
                                probability: *p,
                                target: nid("t"),
                            })
                            .collect(),
                    },
                ),
                ("t", terminal()),
            ],
        )
    }
    // Compile-time compatibility control: an old exhaustive match needs no new arm.
    fn legacy_variant_index(e: &ValidationError) -> usize {
        match e {
            ValidationError::EmptyIdentifier { .. } => 0,
            ValidationError::UnsupportedSchemaVersion { .. } => 1,
            ValidationError::MissingRoot(_) => 2,
            ValidationError::MissingTarget { .. } => 3,
            ValidationError::EmptyDecision(_) => 4,
            ValidationError::DuplicateAction { .. } => 5,
            ValidationError::EmptyChance(_) => 6,
            ValidationError::InvalidProbability(_) => 7,
            ValidationError::ProbabilityMass(_) => 8,
            ValidationError::InvalidPayoff { .. } => 9,
            ValidationError::Cycle(_) => 10,
            ValidationError::UnreachableNode(_) => 11,
        }
    }
    fn expect_error(s: &GameSpecification, expected: ValidationError, code: &str, path: &str) {
        assert_eq!(s.validate(), Err(expected.clone()));
        let wrapped = s.validate_for_profile(PROFILE).unwrap_err();
        assert_eq!(wrapped.code(), code);
        assert_eq!(wrapped.path(), path);
        let source = wrapped.source().expect("original typed source");
        assert_eq!(source.downcast_ref::<ValidationError>(), Some(&expected));
        assert!(source.source().is_none());
        assert_eq!(
            wrapped.to_string(),
            format!("invalid game specification: {expected}")
        );
        assert!(legacy_variant_index(&expected) < 12);
        assert_eq!(
            wrapped,
            ProfileValidationError::InvalidSpecification(expected)
        );
    }
    fn stored_weights(s: &GameSpecification) -> Vec<u64> {
        match &s.game.nodes[&nid("c")] {
            Node::Chance { edges } => edges.iter().map(|e| e.probability.to_bits()).collect(),
            _ => panic!("fixture requires chance root"),
        }
    }

    #[test]
    fn metadata_errors_preserve_payload_code_path_and_source() {
        let mut blank = specification("r", vec![("r", terminal())]);
        blank.id = " ".into();
        expect_error(
            &blank,
            ValidationError::EmptyIdentifier { path: "id".into() },
            "empty_identifier",
            "id",
        );
        let mut future = specification("r", vec![("r", terminal())]);
        future.schema_version = "2.0.0".into();
        expect_error(
            &future,
            ValidationError::UnsupportedSchemaVersion {
                path: "schema_version".into(),
                value: "2.0.0".into(),
            },
            "unsupported_schema_version",
            "schema_version",
        );
        expect_error(
            &specification("r", vec![]),
            ValidationError::MissingRoot(nid("r")),
            "missing_root",
            "game.nodes.r",
        );
    }

    #[test]
    fn graph_errors_preserve_payload_code_path_and_source() {
        expect_error(
            &specification("r", vec![("r", decision(&[("a", "missing.with.dot")]))]),
            ValidationError::MissingTarget {
                from: nid("r"),
                target: nid("missing.with.dot"),
            },
            "missing_target",
            "game.nodes.r.target.missing.with.dot",
        );
        expect_error(
            &specification("r", vec![("r", decision(&[]))]),
            ValidationError::EmptyDecision(nid("r")),
            "empty_decision",
            "game.nodes.r",
        );
        expect_error(
            &specification(
                "r",
                vec![
                    ("r", decision(&[("a", "t"), ("a", "t")])),
                    ("t", terminal()),
                ],
            ),
            ValidationError::DuplicateAction {
                node: nid("r"),
                action: ActionId("a".into()),
            },
            "duplicate_action",
            "game.nodes.r",
        );
        expect_error(
            &chance(&[]),
            ValidationError::EmptyChance(nid("c")),
            "empty_chance",
            "game.nodes.c",
        );
        expect_error(
            &chance(&[f64::NAN]),
            ValidationError::InvalidProbability(nid("c")),
            "invalid_probability",
            "game.nodes.c",
        );
        expect_error(
            &chance(&[0.5]),
            ValidationError::ProbabilityMass(nid("c")),
            "probability_mass",
            "game.nodes.c",
        );
        expect_error(
            &specification(
                "t",
                vec![(
                    "t",
                    Node::Terminal {
                        payoffs: BTreeMap::from([(pid("p"), f64::INFINITY)]),
                    },
                )],
            ),
            ValidationError::InvalidPayoff {
                node: nid("t"),
                player: pid("p"),
            },
            "invalid_payoff",
            "game.nodes.t",
        );
        expect_error(
            &specification("r", vec![("r", decision(&[("a", "r")]))]),
            ValidationError::Cycle(nid("r")),
            "cycle",
            "game.nodes.r",
        );
        expect_error(
            &specification("r", vec![("r", terminal()), ("orphan", terminal())]),
            ValidationError::UnreachableNode(nid("orphan")),
            "unreachable_node",
            "game.nodes.orphan",
        );
    }

    #[test]
    fn blank_identifier_roles_keep_their_original_paths() {
        let blank = "\u{2003}\t\n";
        let mut cases = Vec::new();
        let mut s = specification("r", vec![("r", terminal())]);
        s.id = blank.into();
        cases.push((s, "id"));
        cases.push((specification(blank, vec![(blank, terminal())]), "root"));
        cases.push((
            specification("r", vec![("r", terminal()), (blank, terminal())]),
            "nodes",
        ));
        cases.push((
            specification(
                "r",
                vec![
                    (
                        "r",
                        Node::Decision {
                            player: pid(blank),
                            edges: vec![ActionEdge {
                                action: ActionId("a".into()),
                                target: nid("t"),
                            }],
                        },
                    ),
                    ("t", terminal()),
                ],
            ),
            "decision.player",
        ));
        cases.push((
            specification(
                "r",
                vec![("r", decision(&[(blank, "t")])), ("t", terminal())],
            ),
            "decision.action",
        ));
        cases.push((
            specification(
                "r",
                vec![(
                    "r",
                    Node::Terminal {
                        payoffs: BTreeMap::from([(pid(blank), 1.0)]),
                    },
                )],
            ),
            "terminal.player",
        ));
        for (s, path) in cases {
            expect_error(
                &s,
                ValidationError::EmptyIdentifier { path: path.into() },
                "empty_identifier",
                path,
            );
        }
    }

    #[test]
    fn adversarial_profile_spellings_reject_before_invalid_schema_or_graph() {
        let mut s = specification("missing", vec![]);
        s.schema_version = "future".into();
        s.id.clear();
        let variants = [
            format!("{PROFILE}+build.1"),
            format!("{PROFILE}-rc.1"),
            format!("{PROFILE}/"),
            PROFILE.replace("1.0.0", "01.0.0"),
            PROFILE.replace("1.0.0", "^1.0.0"),
            PROFILE.replace("paignia", "p\u{0430}ignia"),
            PROFILE.replace("/1.0.0", "/%31.0.0"),
            format!("{PROFILE}\n"),
            format!("\0{PROFILE}"),
            PROFILE.replace('-', "_"),
        ];
        for profile in variants {
            let e = s.validate_for_profile(&profile).unwrap_err();
            assert_eq!(
                e,
                ProfileValidationError::UnsupportedSemanticProfile { value: profile }
            );
            assert_eq!(e.code(), "unsupported_semantic_profile");
            assert_eq!(e.path(), "semantic_profile");
            assert!(e.source().is_none());
        }
        expect_error(
            &s,
            ValidationError::UnsupportedSchemaVersion {
                path: "schema_version".into(),
                value: "future".into(),
            },
            "unsupported_schema_version",
            "schema_version",
        );
    }

    #[test]
    fn adjacent_binary64_values_straddle_each_mass_boundary() {
        // Frozen IEEE-754 neighbours, checked independently with exact Decimal.from_float.
        for bits in [0x3ff0_0000_0044_b82f_u64, 0x3fef_ffff_ff76_8fa1] {
            let s = chance(&[f64::from_bits(bits)]);
            assert_eq!(s.validate_for_profile(PROFILE), Ok(()));
            assert_eq!(stored_weights(&s), vec![bits]);
        }
        for bits in [0x3ff0_0000_0044_b830_u64, 0x3fef_ffff_ff76_8fa0] {
            let s = chance(&[f64::from_bits(bits)]);
            expect_error(
                &s,
                ValidationError::ProbabilityMass(nid("c")),
                "probability_mass",
                "game.nodes.c",
            );
            assert_eq!(stored_weights(&s), vec![bits]);
        }
    }

    #[test]
    fn finite_weights_with_overflowed_total_reject() {
        expect_error(
            &chance(&[f64::MAX, f64::MAX]),
            ValidationError::ProbabilityMass(nid("c")),
            "probability_mass",
            "game.nodes.c",
        );
        expect_error(
            &chance(&[0.0, -0.0]),
            ValidationError::ProbabilityMass(nid("c")),
            "probability_mass",
            "game.nodes.c",
        );
    }

    #[test]
    fn shared_descendants_and_parallel_chance_edges_are_admitted() {
        let mut s = specification(
            "r",
            vec![
                ("r", decision(&[("left", "l"), ("right", "h")])),
                (
                    "l",
                    Node::Chance {
                        edges: vec![
                            ChanceEdge {
                                probability: 0.25,
                                target: nid("t"),
                            },
                            ChanceEdge {
                                probability: 0.75,
                                target: nid("t"),
                            },
                        ],
                    },
                ),
                ("h", decision(&[("stay", "t")])),
                ("t", terminal()),
            ],
        );
        let before = s.clone();
        assert_eq!(s.validate_for_profile(PROFILE), Ok(()));
        assert_eq!(s, before);
        s.game.nodes.insert(nid("h"), decision(&[("back", "r")]));
        expect_error(
            &s,
            ValidationError::Cycle(nid("r")),
            "cycle",
            "game.nodes.r",
        );
    }

    #[test]
    fn zero_weight_branches_do_not_hide_cycles_or_nonfinite_payoffs() {
        let mut s = specification(
            "c",
            vec![
                (
                    "c",
                    Node::Chance {
                        edges: vec![
                            ChanceEdge {
                                probability: 1.0,
                                target: nid("t"),
                            },
                            ChanceEdge {
                                probability: 0.0,
                                target: nid("bad"),
                            },
                        ],
                    },
                ),
                ("t", terminal()),
                ("bad", decision(&[("again", "bad")])),
            ],
        );
        expect_error(
            &s,
            ValidationError::Cycle(nid("bad")),
            "cycle",
            "game.nodes.bad",
        );
        s.game.nodes.insert(
            nid("bad"),
            Node::Terminal {
                payoffs: BTreeMap::from([(pid("p"), f64::NAN)]),
            },
        );
        expect_error(
            &s,
            ValidationError::InvalidPayoff {
                node: nid("bad"),
                player: pid("p"),
            },
            "invalid_payoff",
            "game.nodes.bad",
        );
    }

    #[test]
    fn string_identity_preserves_namespace_whitespace_unicode_and_nul() {
        let ids = [
            "https://w3id.org/uogto/core#same",
            "https://w3id.org/uogto/extensions#same",
            "https://kairos.dev/ontology/game-theory#same",
            " a",
            "a",
            "\u{e9}",
            "e\u{301}",
            "x\0y",
        ];
        let mut nodes = vec![(
            " root ",
            Node::Decision {
                player: pid(" p "),
                edges: ids
                    .iter()
                    .map(|id| ActionEdge {
                        action: ActionId((*id).into()),
                        target: nid(id),
                    })
                    .collect(),
            },
        )];
        for id in ids {
            nodes.push((
                id,
                Node::Terminal {
                    payoffs: BTreeMap::from([(pid(id), 1.0)]),
                },
            ));
        }
        let mut s = specification(" root ", nodes);
        s.id = " spec ".into();
        let before = s.clone();
        assert_eq!(s.validate_for_profile(PROFILE), Ok(()));
        assert_eq!(s, before);
        assert_eq!(s.game.nodes.len(), 9);
    }

    #[test]
    fn signed_zero_and_minimum_subnormal_weights_are_retained_bitwise() {
        let s = chance(&[1.0, -0.0, f64::from_bits(1)]);
        assert_eq!(s.validate_for_profile(PROFILE), Ok(()));
        assert_eq!(
            stored_weights(&s),
            vec![1.0_f64.to_bits(), 0x8000_0000_0000_0000, 1]
        );
    }
}
