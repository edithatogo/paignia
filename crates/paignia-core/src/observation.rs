//! Owned decision/action projections for the explicitly declared static profile.
//!
//! The caller supplies the expected context. Equality checks bind a request to
//! that declaration; they do not establish live state, authority or freshness.

use thiserror::Error;

use crate::{
    ActionId, GameInstanceId, GameSessionId, GameSpecification, GameSpecificationId, Node, NodeId,
    PlayerId, ProfileValidationError,
};

/// Exact native request/projection schema. This is not a wire-format version.
pub const SCHEMA_VERSION: &str = "1.0.0";

/// Caller-declared observation revision. Equality binds a request to a context;
/// zero and `u64::MAX` are valid labels. No native clock or counter is advanced.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Revision(pub u64);

/// Owned role-specific identities for one declared decision observation.
///
/// The caller establishes which instance, session, decision and revision are
/// actually current. This record contains no live native handle or authority
/// token, and matching it does not authenticate the caller's declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Binding {
    pub specification_id: GameSpecificationId,
    pub game_instance_id: GameInstanceId,
    pub session_id: GameSessionId,
    pub player_id: PlayerId,
    pub decision_node_id: NodeId,
    pub revision: Revision,
}

/// Request for a decision projection under one exact supported input profile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Request {
    pub schema_version: String,
    pub semantic_profile: String,
    pub binding: Binding,
}

/// Complete owned policy DTO: public decision metadata and ordered legal IDs.
///
/// The projection has no game, edge targets, payoffs, trace, borrowed world
/// reference or opaque payload. Public mutation or manual construction is not
/// a new source/binding validation; use the builder for a checked projection.
/// Only this successful DTO is intended as policy input. Builder errors are
/// diagnostics for the trusted caller and can contain source-validation paths.
///
/// ```compile_fail
/// use paignia_core::PolicyObservation;
/// fn cannot_read_world(observation: PolicyObservation) {
///     let _ = observation.game;
/// }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Observation {
    pub schema_version: String,
    pub semantic_profile: String,
    pub binding: Binding,
    /// Exact source edge order. IDs are neither sorted nor rewritten, and no
    /// destination node or transition is attached to an action.
    pub legal_actions: Vec<ActionId>,
}

/// Node kinds that cannot produce a player-decision observation in this profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RejectedNodeKind {
    Chance,
    Terminal,
}

/// Construction diagnostic for the caller, separate from the policy DTO.
#[derive(Clone, Debug, Eq, PartialEq, Error)]
pub enum Error {
    #[error("unsupported policy observation schema version {value}")]
    UnsupportedSchemaVersion { value: String },
    #[error("invalid policy observation source: {0}")]
    InvalidSource(#[from] ProfileValidationError),
    #[error("empty policy observation identifier at {path}")]
    EmptyIdentifier { path: &'static str },
    #[error("observation expected specification {actual:?} differs from source {expected:?}")]
    SourceIdentityMismatch {
        expected: GameSpecificationId,
        actual: GameSpecificationId,
    },
    #[error("policy observation binding mismatch at {path}")]
    BindingMismatch { path: &'static str },
    #[error("unknown policy decision node {node:?}")]
    UnknownDecisionNode { node: NodeId },
    #[error("policy observation node {node:?} is a {kind:?} node")]
    NonDecisionNode {
        node: NodeId,
        kind: RejectedNodeKind,
    },
    #[error("policy observation for node {node:?} requests {actual:?}, but its controlling player is {expected:?}")]
    WrongPlayer {
        node: NodeId,
        expected: PlayerId,
        actual: PlayerId,
    },
}

impl Error {
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::UnsupportedSchemaVersion { .. } => "unsupported_policy_observation_schema",
            Self::InvalidSource(error) => error.code(),
            Self::EmptyIdentifier { .. } => "empty_observation_identifier",
            Self::SourceIdentityMismatch { .. } => "observation_source_mismatch",
            Self::BindingMismatch { .. } => "observation_binding_mismatch",
            Self::UnknownDecisionNode { .. } => "unknown_observation_decision",
            Self::NonDecisionNode { .. } => "nondecision_observation_node",
            Self::WrongPlayer { .. } => "wrong_observation_player",
        }
    }

    #[must_use]
    pub fn path(&self) -> String {
        match self {
            Self::UnsupportedSchemaVersion { .. } => "schema_version".into(),
            Self::InvalidSource(ProfileValidationError::UnsupportedSemanticProfile { .. }) => {
                "semantic_profile".into()
            }
            Self::InvalidSource(error) => format!("source.{}", error.path()),
            Self::EmptyIdentifier { path } | Self::BindingMismatch { path } => (*path).into(),
            Self::SourceIdentityMismatch { .. } => "expected.specification_id".into(),
            Self::UnknownDecisionNode { .. } | Self::NonDecisionNode { .. } => {
                "binding.decision_node_id".into()
            }
            Self::WrongPlayer { .. } => "binding.player_id".into(),
        }
    }
}

/// Construct the declared static perfect-information decision observation.
///
/// Validate the entire source under the request's exact input profile, bind the
/// request to the caller's expected specification/instance/session/player/node/
/// revision, and verify that the source node is owned by that player. Return
/// only owned binding metadata and action IDs in the source's declared order.
/// All input values remain unchanged.
///
/// This function does not infer a lawful information set from a reduced game,
/// implement hidden or simultaneous information structures, or prove that the
/// declared node/revision is current. The adapter or host establishes native
/// lifecycle, session authority and freshness before supplying `expected`.
///
/// # Errors
///
/// Errors are deterministic in this order: request schema; semantic profile and
/// full source validation; nonblank expected then request identities; source
/// specification identity; request binding fields in declaration order; node
/// existence/kind; controlling player. Unsupported profiles are explicit errors.
/// A source-validation error retains its original typed error and code; its
/// path is prefixed with `source.` except for the request's semantic profile.
pub fn build(
    source: &GameSpecification,
    expected: &Binding,
    request: &Request,
) -> Result<Observation, Error> {
    if request.schema_version != SCHEMA_VERSION {
        return Err(Error::UnsupportedSchemaVersion {
            value: request.schema_version.clone(),
        });
    }
    source.validate_for_profile(&request.semantic_profile)?;
    validate_binding(
        expected,
        [
            "expected.specification_id",
            "expected.game_instance_id",
            "expected.session_id",
            "expected.player_id",
            "expected.decision_node_id",
        ],
    )?;
    validate_binding(
        &request.binding,
        [
            "binding.specification_id",
            "binding.game_instance_id",
            "binding.session_id",
            "binding.player_id",
            "binding.decision_node_id",
        ],
    )?;
    if expected.specification_id.0 != source.id {
        return Err(Error::SourceIdentityMismatch {
            expected: GameSpecificationId(source.id.clone()),
            actual: expected.specification_id.clone(),
        });
    }
    compare_binding(&request.binding, expected)?;
    let node = source
        .game
        .nodes
        .get(&expected.decision_node_id)
        .ok_or_else(|| Error::UnknownDecisionNode {
            node: expected.decision_node_id.clone(),
        })?;
    let (player, edges) = match node {
        Node::Decision { player, edges } => (player, edges),
        Node::Chance { .. } => {
            return Err(Error::NonDecisionNode {
                node: expected.decision_node_id.clone(),
                kind: RejectedNodeKind::Chance,
            })
        }
        Node::Terminal { .. } => {
            return Err(Error::NonDecisionNode {
                node: expected.decision_node_id.clone(),
                kind: RejectedNodeKind::Terminal,
            })
        }
    };
    if player != &expected.player_id {
        return Err(Error::WrongPlayer {
            node: expected.decision_node_id.clone(),
            expected: player.clone(),
            actual: expected.player_id.clone(),
        });
    }
    Ok(Observation {
        schema_version: request.schema_version.clone(),
        semantic_profile: request.semantic_profile.clone(),
        binding: expected.clone(),
        legal_actions: edges.iter().map(|edge| edge.action.clone()).collect(),
    })
}

fn validate_binding(binding: &Binding, paths: [&'static str; 5]) -> Result<(), Error> {
    let identifiers = [
        &binding.specification_id.0,
        &binding.game_instance_id.0,
        &binding.session_id.0,
        &binding.player_id.0,
        &binding.decision_node_id.0,
    ];
    for (path, value) in paths.into_iter().zip(identifiers) {
        if value.trim().is_empty() {
            return Err(Error::EmptyIdentifier { path });
        }
    }
    Ok(())
}

fn compare_binding(request: &Binding, expected: &Binding) -> Result<(), Error> {
    let matches = [
        (
            "binding.specification_id",
            request.specification_id == expected.specification_id,
        ),
        (
            "binding.game_instance_id",
            request.game_instance_id == expected.game_instance_id,
        ),
        (
            "binding.session_id",
            request.session_id == expected.session_id,
        ),
        ("binding.player_id", request.player_id == expected.player_id),
        (
            "binding.decision_node_id",
            request.decision_node_id == expected.decision_node_id,
        ),
        ("binding.revision", request.revision == expected.revision),
    ];
    for (path, matches) in matches {
        if !matches {
            return Err(Error::BindingMismatch { path });
        }
    }
    Ok(())
}
