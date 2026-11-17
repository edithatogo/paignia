//! Domain-neutral finite extensive-form game types.
//!
//! The crate deliberately contains no application-domain concepts. It is the
//! Paignia extraction seed for a Rust game library independent of Kairos and aligned with
//! UOGTO.

pub mod result;

use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PlayerId(pub String);

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NodeId(pub String);

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ActionId(pub String);

pub type PayoffVector = BTreeMap<PlayerId, f64>;

#[derive(Clone, Debug, PartialEq)]
pub struct ActionEdge {
    pub action: ActionId,
    pub target: NodeId,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ChanceEdge {
    pub probability: f64,
    pub target: NodeId,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    Decision {
        player: PlayerId,
        edges: Vec<ActionEdge>,
    },
    Chance {
        edges: Vec<ChanceEdge>,
    },
    Terminal {
        payoffs: PayoffVector,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Game {
    pub root: NodeId,
    pub nodes: BTreeMap<NodeId, Node>,
}

pub const GAME_SPEC_SCHEMA_VERSION: &str = "1.0.0";

/// Exact input-profile identifier for the existing finite, acyclic,
/// perfect-information graph with intentional sparse-zero terminal payoffs.
///
/// This is a Paignia input-validation profile, not a UOGTO vocabulary IRI,
/// solver-result version, or numerical qualification receipt.
pub const STATIC_GAME_PROFILE_V1: &str =
    "paignia/static-finite-perfect-information-sparse-zero/1.0.0";

/// Component schema for [`ResultIdentityRecord`], not a complete result envelope.
pub const RESULT_IDENTITY_SCHEMA_VERSION: &str = "1.0.0";

/// An opaque specification identity, preserving its complete string.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GameSpecificationId(pub String);

/// An instantiated game's identity, distinct from its specification.
///
/// ```compile_fail
/// use paignia_core::{GameInstanceId, GameSpecificationId};
/// let instance: GameInstanceId = GameSpecificationId("urn:example:same".into());
/// ```
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GameInstanceId(pub String);

/// One solver invocation; this is not a game session or simulation replicate.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SolverRunId(pub String);

/// Optional external game-session identity; no live session is created here.
///
/// ```compile_fail
/// use paignia_core::{GameSessionId, SolverRunId};
/// let session: GameSessionId = SolverRunId("urn:example:same".into());
/// ```
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GameSessionId(pub String);

/// An optional policy artifact identity, not a solver run or live policy state.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PolicyArtifactId(pub String);

/// A solver-diagnostic trace identity, never a realized causal execution trace.
///
/// ```compile_fail
/// use paignia_core::{PolicyArtifactId, SolverDiagnosticTraceId};
/// let trace: SolverDiagnosticTraceId = PolicyArtifactId("urn:example:same".into());
/// ```
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SolverDiagnosticTraceId(pub String);

/// Role-specific identity metadata for a future result envelope.
///
/// Admission checks this component's schema and supplied strings only. It does
/// not resolve references, certify source bindings or validate a solver result.
/// Optional context remains absent for a static caller that does not supply it.
/// The role and full string together identify a record; equal strings in
/// different roles are not implicitly equated or converted.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResultIdentityRecord {
    pub schema_version: String,
    pub specification_id: GameSpecificationId,
    pub game_instance_id: GameInstanceId,
    pub solver_run_id: SolverRunId,
    pub session_id: Option<GameSessionId>,
    pub policy_artifact_id: Option<PolicyArtifactId>,
    pub solver_trace_id: Option<SolverDiagnosticTraceId>,
}

/// Structural admission failure for [`ResultIdentityRecord`].
#[derive(Clone, Debug, Eq, PartialEq, Error)]
pub enum ResultIdentityValidationError {
    #[error("unsupported result identity schema version {value}")]
    UnsupportedSchemaVersion { value: String },
    #[error("empty result identity at {path}")]
    EmptyIdentifier { path: &'static str },
}

impl ResultIdentityValidationError {
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::UnsupportedSchemaVersion { .. } => "unsupported_result_identity_schema",
            Self::EmptyIdentifier { .. } => "empty_result_identity",
        }
    }

    #[must_use]
    pub fn path(&self) -> &'static str {
        match self {
            Self::UnsupportedSchemaVersion { .. } => "schema_version",
            Self::EmptyIdentifier { path } => path,
        }
    }
}

impl ResultIdentityRecord {
    /// Admit the exact component schema and nonblank supplied identities.
    ///
    /// Strings are not trimmed, normalized, shortened, generated or resolved.
    /// Missing optional context is distinct from a supplied empty identifier.
    ///
    /// # Errors
    ///
    /// Returns [`ResultIdentityValidationError`] for an unsupported schema,
    /// checked first, or a blank identifier in declaration order.
    pub fn validate(&self) -> Result<(), ResultIdentityValidationError> {
        if self.schema_version != RESULT_IDENTITY_SCHEMA_VERSION {
            return Err(ResultIdentityValidationError::UnsupportedSchemaVersion {
                value: self.schema_version.clone(),
            });
        }
        let identities = [
            ("specification_id", Some(self.specification_id.0.as_str())),
            ("game_instance_id", Some(self.game_instance_id.0.as_str())),
            ("solver_run_id", Some(self.solver_run_id.0.as_str())),
            (
                "session_id",
                self.session_id.as_ref().map(|id| id.0.as_str()),
            ),
            (
                "policy_artifact_id",
                self.policy_artifact_id.as_ref().map(|id| id.0.as_str()),
            ),
            (
                "solver_trace_id",
                self.solver_trace_id.as_ref().map(|id| id.0.as_str()),
            ),
        ];
        for (path, value) in identities {
            if value.is_some_and(|id| id.trim().is_empty()) {
                return Err(ResultIdentityValidationError::EmptyIdentifier { path });
            }
        }
        Ok(())
    }
}

/// Exact schema for a scalar diagnostic component, not a full result envelope.
pub const RESULT_DIAGNOSTIC_SCHEMA_VERSION: &str = "1.0.0";

/// The caller-declared category of a nonnegative diagnostic magnitude.
///
/// This tag does not define a norm, unit, game class or supported solver method.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ResultDiagnosticKind {
    Residual,
    Exploitability,
}

/// A declared diagnostic magnitude and its explicit, caller-supplied tolerance.
///
/// Structural admission does not certify the calculation, source binding,
/// tolerance policy, convergence or existence of an equilibrium. The enclosing
/// method contract must define the metric and its meaning. Undefined diagnostics
/// must not be manufactured as zero-valued records. Public mutation requires
/// revalidation; no solver automatically creates this component.
#[derive(Clone, Debug, PartialEq)]
pub struct ResultDiagnosticRecord {
    pub schema_version: String,
    pub kind: ResultDiagnosticKind,
    pub value: f64,
    pub tolerance: f64,
}

/// Structural admission failure for a result diagnostic component.
#[derive(Clone, Debug, Eq, PartialEq, Error)]
pub enum ResultDiagnosticValidationError {
    #[error("unsupported result diagnostic schema version {value}")]
    UnsupportedSchemaVersion { value: String },
    #[error("non-finite result diagnostic at {path}")]
    NonFiniteNumber { path: &'static str },
    #[error("negative result diagnostic at {path}")]
    NegativeNumber { path: &'static str },
}

impl ResultDiagnosticValidationError {
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::UnsupportedSchemaVersion { .. } => "unsupported_result_diagnostic_schema",
            Self::NonFiniteNumber { .. } => "non_finite_result_diagnostic",
            Self::NegativeNumber { .. } => "negative_result_diagnostic",
        }
    }

    #[must_use]
    pub fn path(&self) -> &'static str {
        match self {
            Self::UnsupportedSchemaVersion { .. } => "schema_version",
            Self::NonFiniteNumber { path } | Self::NegativeNumber { path } => path,
        }
    }
}

impl ResultDiagnosticRecord {
    /// Admit this exact schema and finite, nonnegative supplied numbers.
    ///
    /// Checks schema first, then value, then tolerance. Zero tolerance is legal;
    /// signed zero and subnormal magnitudes are preserved. No value is clamped,
    /// normalized or compared with tolerance to infer convergence.
    ///
    /// # Errors
    ///
    /// Returns a schema error for an unsupported version, otherwise the first
    /// invalid numeric field. Non-finite checks precede negative checks.
    pub fn validate(&self) -> Result<(), ResultDiagnosticValidationError> {
        if self.schema_version != RESULT_DIAGNOSTIC_SCHEMA_VERSION {
            return Err(ResultDiagnosticValidationError::UnsupportedSchemaVersion {
                value: self.schema_version.clone(),
            });
        }
        for (path, value) in [("value", self.value), ("tolerance", self.tolerance)] {
            if !value.is_finite() {
                return Err(ResultDiagnosticValidationError::NonFiniteNumber { path });
            }
            if value < 0.0 {
                return Err(ResultDiagnosticValidationError::NegativeNumber { path });
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GameSpecification {
    pub schema_version: String,
    pub id: String,
    pub game: Game,
}

#[derive(Clone, Debug, Eq, PartialEq, Error)]
pub enum ValidationError {
    #[error("empty identifier at {path}")]
    EmptyIdentifier { path: String },
    #[error("unsupported schema version {value} at {path}")]
    UnsupportedSchemaVersion { path: String, value: String },
    #[error("missing root node {0:?}")]
    MissingRoot(NodeId),
    #[error("missing target {target:?} referenced by {from:?}")]
    MissingTarget { from: NodeId, target: NodeId },
    #[error("decision node {0:?} has no actions")]
    EmptyDecision(NodeId),
    #[error("duplicate action {action:?} at decision node {node:?}")]
    DuplicateAction { node: NodeId, action: ActionId },
    #[error("chance node {0:?} has no outcomes")]
    EmptyChance(NodeId),
    #[error("invalid probability at node {0:?}")]
    InvalidProbability(NodeId),
    #[error("probability mass does not equal one at node {0:?}")]
    ProbabilityMass(NodeId),
    #[error("invalid payoff for player {player:?} at node {node:?}")]
    InvalidPayoff { node: NodeId, player: PlayerId },
    #[error("cycle detected at node {0:?}")]
    Cycle(NodeId),
    #[error("unreachable node {0:?}")]
    UnreachableNode(NodeId),
}

impl ValidationError {
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::EmptyIdentifier { .. } => "empty_identifier",
            Self::UnsupportedSchemaVersion { .. } => "unsupported_schema_version",
            Self::MissingRoot(_) => "missing_root",
            Self::MissingTarget { .. } => "missing_target",
            Self::EmptyDecision(_) => "empty_decision",
            Self::DuplicateAction { .. } => "duplicate_action",
            Self::EmptyChance(_) => "empty_chance",
            Self::InvalidProbability(_) => "invalid_probability",
            Self::ProbabilityMass(_) => "probability_mass",
            Self::InvalidPayoff { .. } => "invalid_payoff",
            Self::Cycle(_) => "cycle",
            Self::UnreachableNode(_) => "unreachable_node",
        }
    }

    #[must_use]
    pub fn path(&self) -> String {
        match self {
            Self::EmptyIdentifier { path } | Self::UnsupportedSchemaVersion { path, .. } => {
                path.clone()
            }
            Self::MissingTarget { from, target } => {
                format!("game.nodes.{}.target.{}", from.0, target.0)
            }
            Self::MissingRoot(node)
            | Self::EmptyDecision(node)
            | Self::DuplicateAction { node, .. }
            | Self::EmptyChance(node)
            | Self::InvalidProbability(node)
            | Self::ProbabilityMass(node)
            | Self::InvalidPayoff { node, .. }
            | Self::Cycle(node)
            | Self::UnreachableNode(node) => format!("game.nodes.{}", node.0),
        }
    }
}

/// Admission failure for an explicitly requested semantic input profile.
///
/// The separate wrapper preserves the existing [`ValidationError`] variants
/// for callers of [`GameSpecification::validate`].
#[derive(Clone, Debug, Eq, PartialEq, Error)]
pub enum ProfileValidationError {
    #[error("unsupported semantic profile {value}")]
    UnsupportedSemanticProfile { value: String },
    #[error("invalid game specification: {0}")]
    InvalidSpecification(#[from] ValidationError),
}

impl ProfileValidationError {
    /// Return the admission error code, preserving wrapped validation codes.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::UnsupportedSemanticProfile { .. } => "unsupported_semantic_profile",
            Self::InvalidSpecification(error) => error.code(),
        }
    }

    /// Return the profile field or the original specification/graph error path.
    #[must_use]
    pub fn path(&self) -> String {
        match self {
            Self::UnsupportedSemanticProfile { .. } => "semantic_profile".into(),
            Self::InvalidSpecification(error) => error.path(),
        }
    }
}

impl GameSpecification {
    /// Validate this specification under an explicitly requested input profile.
    ///
    /// Only [`STATIC_GAME_PROFILE_V1`] is admitted, using exact string equality
    /// before the existing schema and graph checks. No identifier, probability,
    /// or payoff is rewritten. Profile admission does not solve the game or
    /// certify the interpretation of an external representation.
    ///
    /// # Errors
    ///
    /// Returns [`ProfileValidationError::UnsupportedSemanticProfile`] for any
    /// other profile, including unknown versions. For the supported profile,
    /// wraps the original [`ValidationError`] from [`Self::validate`].
    pub fn validate_for_profile(
        &self,
        semantic_profile: &str,
    ) -> Result<(), ProfileValidationError> {
        if semantic_profile != STATIC_GAME_PROFILE_V1 {
            return Err(ProfileValidationError::UnsupportedSemanticProfile {
                value: semantic_profile.into(),
            });
        }
        self.validate().map_err(ProfileValidationError::from)
    }

    /// Validate specification metadata and the embedded game graph.
    ///
    /// # Errors
    ///
    /// Returns a structured [`ValidationError`] for unsupported schema
    /// versions, empty canonical identifiers, or invalid game semantics.
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.schema_version != GAME_SPEC_SCHEMA_VERSION {
            return Err(ValidationError::UnsupportedSchemaVersion {
                path: "schema_version".into(),
                value: self.schema_version.clone(),
            });
        }
        Game::ensure_identifier("id", &self.id)?;
        self.game.validate()
    }
}

impl Game {
    /// # Errors
    ///
    /// Returns a [`ValidationError`] when the game graph references a missing
    /// node, contains a cycle, or otherwise violates the supported finite-game
    /// contract.
    pub fn validate(&self) -> Result<(), ValidationError> {
        Self::ensure_identifier("root", &self.root.0)?;
        if !self.nodes.contains_key(&self.root) {
            return Err(ValidationError::MissingRoot(self.root.clone()));
        }
        for (node_id, node) in &self.nodes {
            Self::ensure_identifier("nodes", &node_id.0)?;
            match node {
                Node::Decision { player, edges } => {
                    Self::ensure_identifier("decision.player", &player.0)?;
                    if edges.is_empty() {
                        return Err(ValidationError::EmptyDecision(node_id.clone()));
                    }
                    let mut actions = BTreeSet::new();
                    for edge in edges {
                        Self::ensure_identifier("decision.action", &edge.action.0)?;
                        if !actions.insert(edge.action.clone()) {
                            return Err(ValidationError::DuplicateAction {
                                node: node_id.clone(),
                                action: edge.action.clone(),
                            });
                        }
                        self.ensure_target(node_id, &edge.target)?;
                    }
                }
                Node::Chance { edges } => {
                    if edges.is_empty() {
                        return Err(ValidationError::EmptyChance(node_id.clone()));
                    }
                    let mut total = 0.0;
                    for edge in edges {
                        if !edge.probability.is_finite() || edge.probability < 0.0 {
                            return Err(ValidationError::InvalidProbability(node_id.clone()));
                        }
                        total += edge.probability;
                        self.ensure_target(node_id, &edge.target)?;
                    }
                    if (total - 1.0).abs() > 1e-9 {
                        return Err(ValidationError::ProbabilityMass(node_id.clone()));
                    }
                }
                Node::Terminal { payoffs } => {
                    for (player, value) in payoffs {
                        Self::ensure_identifier("terminal.player", &player.0)?;
                        if !value.is_finite() {
                            return Err(ValidationError::InvalidPayoff {
                                node: node_id.clone(),
                                player: player.clone(),
                            });
                        }
                    }
                }
            }
        }
        let mut visited = BTreeSet::new();
        self.validate_acyclic_reachability(&mut visited)?;
        if let Some(unreachable) = self
            .nodes
            .keys()
            .find(|node_id| !visited.contains(*node_id))
        {
            return Err(ValidationError::UnreachableNode(unreachable.clone()));
        }
        Ok(())
    }

    fn ensure_identifier(path: &str, value: &str) -> Result<(), ValidationError> {
        if value.trim().is_empty() {
            Err(ValidationError::EmptyIdentifier {
                path: path.to_owned(),
            })
        } else {
            Ok(())
        }
    }

    fn ensure_target(&self, from: &NodeId, target: &NodeId) -> Result<(), ValidationError> {
        if self.nodes.contains_key(target) {
            Ok(())
        } else {
            Err(ValidationError::MissingTarget {
                from: from.clone(),
                target: target.clone(),
            })
        }
    }

    fn validate_acyclic_reachability(
        &self,
        visited: &mut BTreeSet<NodeId>,
    ) -> Result<(), ValidationError> {
        let mut state = BTreeMap::<NodeId, u8>::new();
        let mut stack = vec![(self.root.clone(), false)];
        while let Some((node_id, exiting)) = stack.pop() {
            if exiting {
                state.insert(node_id.clone(), 2);
                visited.insert(node_id);
                continue;
            }
            match state.get(&node_id).copied() {
                Some(1) => return Err(ValidationError::Cycle(node_id)),
                Some(2) => continue,
                _ => {}
            }
            state.insert(node_id.clone(), 1);
            stack.push((node_id.clone(), true));
            let node = self.nodes.get(&node_id).expect("validated node existence");
            match node {
                Node::Decision { edges, .. } => {
                    for edge in edges.iter().rev() {
                        stack.push((edge.target.clone(), false));
                    }
                }
                Node::Chance { edges } => {
                    for edge in edges.iter().rev() {
                        stack.push((edge.target.clone(), false));
                    }
                }
                Node::Terminal { .. } => {}
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_cycles() {
        let root = NodeId("root".into());
        let mut nodes = BTreeMap::new();
        nodes.insert(
            root.clone(),
            Node::Decision {
                player: PlayerId("p".into()),
                edges: vec![ActionEdge {
                    action: ActionId("again".into()),
                    target: root.clone(),
                }],
            },
        );
        let game = Game { root, nodes };
        assert!(matches!(game.validate(), Err(ValidationError::Cycle(_))));
    }

    #[test]
    fn rejects_duplicate_actions_at_a_decision_node() {
        let root = NodeId("root".into());
        let terminal = NodeId("terminal".into());
        let action = ActionId("same".into());
        let game = Game {
            root: root.clone(),
            nodes: BTreeMap::from([
                (
                    root.clone(),
                    Node::Decision {
                        player: PlayerId("player".into()),
                        edges: vec![
                            ActionEdge {
                                action: action.clone(),
                                target: terminal.clone(),
                            },
                            ActionEdge {
                                action: action.clone(),
                                target: terminal.clone(),
                            },
                        ],
                    },
                ),
                (
                    terminal,
                    Node::Terminal {
                        payoffs: BTreeMap::new(),
                    },
                ),
            ]),
        };
        assert_eq!(
            game.validate(),
            Err(ValidationError::DuplicateAction { node: root, action })
        );
    }

    #[test]
    fn validates_deep_games_without_recursive_stack_growth() {
        let depth = 20_000;
        let mut nodes = BTreeMap::new();
        for index in 0..depth {
            nodes.insert(
                NodeId(format!("n{index}")),
                Node::Decision {
                    player: PlayerId("p".into()),
                    edges: vec![ActionEdge {
                        action: ActionId("next".into()),
                        target: NodeId(format!("n{}", index + 1)),
                    }],
                },
            );
        }
        nodes.insert(
            NodeId(format!("n{depth}")),
            Node::Terminal {
                payoffs: BTreeMap::new(),
            },
        );
        let game = Game {
            root: NodeId("n0".into()),
            nodes,
        };
        assert_eq!(game.validate(), Ok(()));
    }

    #[test]
    fn rejects_unreachable_nodes_and_non_finite_payoffs() {
        let root = NodeId("root".into());
        let unreachable = NodeId("unreachable".into());
        let player = PlayerId("player".into());
        let invalid_payoff = Game {
            root: root.clone(),
            nodes: BTreeMap::from([(
                root.clone(),
                Node::Terminal {
                    payoffs: BTreeMap::from([(player.clone(), f64::NAN)]),
                },
            )]),
        };
        assert_eq!(
            invalid_payoff.validate(),
            Err(ValidationError::InvalidPayoff {
                node: root.clone(),
                player,
            })
        );

        let unreachable_game = Game {
            root: root.clone(),
            nodes: BTreeMap::from([
                (
                    root,
                    Node::Terminal {
                        payoffs: BTreeMap::new(),
                    },
                ),
                (
                    unreachable.clone(),
                    Node::Terminal {
                        payoffs: BTreeMap::new(),
                    },
                ),
            ]),
        };
        assert_eq!(
            unreachable_game.validate(),
            Err(ValidationError::UnreachableNode(unreachable))
        );
    }

    #[test]
    fn specification_metadata_is_versioned_and_errors_have_paths() {
        let root = NodeId("terminal".into());
        let game = Game {
            root: root.clone(),
            nodes: BTreeMap::from([(
                root,
                Node::Terminal {
                    payoffs: BTreeMap::new(),
                },
            )]),
        };
        let valid = GameSpecification {
            schema_version: GAME_SPEC_SCHEMA_VERSION.into(),
            id: "urn:uogto:game:fixture".into(),
            game: game.clone(),
        };
        assert_eq!(valid.validate(), Ok(()));

        let invalid = GameSpecification {
            schema_version: "2.0.0".into(),
            id: valid.id,
            game,
        };
        let error = invalid.validate().unwrap_err();
        assert_eq!(error.code(), "unsupported_schema_version");
        assert_eq!(error.path(), "schema_version");
    }

    #[test]
    fn validation_errors_explain_location_without_debug_only_display() {
        let empty = ValidationError::EmptyIdentifier {
            path: "game.id".into(),
        };
        assert_eq!(empty.code(), "empty_identifier");
        assert_eq!(empty.path(), "game.id");
        assert_eq!(empty.to_string(), "empty identifier at game.id");

        let wrong_schema = ValidationError::UnsupportedSchemaVersion {
            path: "schema_version".into(),
            value: "2.0.0".into(),
        };
        assert_eq!(wrong_schema.code(), "unsupported_schema_version");
        assert!(wrong_schema.to_string().contains("2.0.0"));
        assert!(wrong_schema.to_string().contains("schema_version"));
    }
}
