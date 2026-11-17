//! Owned policy observations for the explicitly declared static profile.

use thiserror::Error;

use crate::{
    ActionId, GameInstanceId, GameSessionId, GameSpecification, GameSpecificationId, NodeId,
    PlayerId, ProfileValidationError,
};

pub const SCHEMA_VERSION: &str = "1.0.0";

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Revision(pub u64);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Binding {
    pub specification_id: GameSpecificationId,
    pub game_instance_id: GameInstanceId,
    pub session_id: GameSessionId,
    pub player_id: PlayerId,
    pub decision_node_id: NodeId,
    pub revision: Revision,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Request {
    pub schema_version: String,
    pub semantic_profile: String,
    pub binding: Binding,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Observation {
    pub schema_version: String,
    pub semantic_profile: String,
    pub binding: Binding,
    pub legal_actions: Vec<ActionId>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RejectedNodeKind {
    Chance,
    Terminal,
}

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
    #[error("test-first policy observation implementation is absent")]
    NotImplemented,
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
            Self::NotImplemented => "not_implemented",
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
            Self::NotImplemented => "implementation".into(),
        }
    }
}

/// # Errors
/// Returns an explicit test-first failure until the implementation exists.
pub fn build(
    _source: &GameSpecification,
    _expected: &Binding,
    _request: &Request,
) -> Result<Observation, Error> {
    Err(Error::NotImplemented)
}
