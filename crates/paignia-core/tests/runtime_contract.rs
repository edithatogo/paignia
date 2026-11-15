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
