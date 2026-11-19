//! Domain-neutral finite information-set admission over the static graph profile.
//!
//! The base [`crate::Game`] retains its bounded perfect-information profile.
//! This module adds a separate declared partition of decision nodes; it does not
//! add hidden state, Kairos storage, ontology semantics or solver behavior.

use std::collections::BTreeMap;

use thiserror::Error;

use crate::{
    ActionId, GameSpecification, GameSpecificationId, NodeId, PlayerId, ProfileValidationError,
};

/// Exact native declaration/graph schema. This is not a wire-format version.
pub const SCHEMA_VERSION: &str = "1.0.0";

/// An opaque information-set identity, preserving its complete string.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct InformationSetId(pub String);

/// One caller-declared information set and its member decision nodes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SetDeclaration {
    pub id: InformationSetId,
    pub player: PlayerId,
    pub members: Vec<NodeId>,
}

/// A complete declared partition of the source graph's decision nodes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Declaration {
    pub schema_version: String,
    pub semantic_profile: String,
    pub sets: Vec<SetDeclaration>,
}

/// One admitted information set in canonical member order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Set {
    pub player: PlayerId,
    pub legal_actions: Vec<ActionId>,
    pub members: Vec<NodeId>,
}

/// Admitted information-set graph with an explicit node-to-set assignment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Graph {
    pub schema_version: String,
    pub semantic_profile: String,
    pub specification_id: GameSpecificationId,
    pub sets: BTreeMap<InformationSetId, Set>,
    pub assignments: BTreeMap<NodeId, InformationSetId>,
}

/// Node kinds that cannot join an information set in this profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RejectedNodeKind {
    Chance,
    Terminal,
}

/// Structural admission failure for a declared information-set graph.
#[derive(Clone, Debug, Eq, PartialEq, Error)]
pub enum Error {
    #[error("unsupported information-set schema version {value}")]
    UnsupportedSchemaVersion { value: String },
    #[error("unsupported information-set semantic profile {value}")]
    UnsupportedSemanticProfile { value: String },
    #[error("invalid information-set source: {0}")]
    InvalidSource(#[from] ProfileValidationError),
    #[error("empty information-set identifier at {path}")]
    EmptyIdentifier { path: &'static str },
    #[error("duplicate information set {0:?}")]
    DuplicateInformationSet(InformationSetId),
    #[error("information set {0:?} has no member nodes")]
    EmptyInformationSet(InformationSetId),
    #[error("duplicate member {node:?} in information set {set:?}")]
    DuplicateMember { set: InformationSetId, node: NodeId },
    #[error("unknown information-set member node {node:?} in set {set:?}")]
    UnknownNode { set: InformationSetId, node: NodeId },
    #[error("information-set member {node:?} in {set:?} is a {kind:?} node")]
    NonDecisionNode {
        set: InformationSetId,
        node: NodeId,
        kind: RejectedNodeKind,
    },
    #[error(
        "information set {set:?} declares player {expected:?}, but member {node:?} is controlled by {actual:?}"
    )]
    PlayerMismatch {
        set: InformationSetId,
        node: NodeId,
        expected: PlayerId,
        actual: PlayerId,
    },
    #[error("node {node:?} belongs to both information sets {first:?} and {second:?}")]
    DuplicateMembership {
        node: NodeId,
        first: InformationSetId,
        second: InformationSetId,
    },
    #[error("decision node {0:?} is not assigned to an information set")]
    UnassignedDecisionNode(NodeId),
    #[error(
        "information-set member {node:?} in {set:?} has legal actions differing from {reference:?}"
    )]
    ActionMismatch {
        set: InformationSetId,
        node: NodeId,
        reference: NodeId,
    },
    #[error(
        "information-set member {node:?} in {set:?} has a prior own history differing from {reference:?}"
    )]
    ImperfectRecall {
        set: InformationSetId,
        node: NodeId,
        reference: NodeId,
    },
    #[error("player {player:?} cannot be recalled consistently at decision node {node:?}")]
    AmbiguousRecall { node: NodeId, player: PlayerId },
}

impl Error {
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::UnsupportedSchemaVersion { .. } => "unsupported_information_set_schema",
            Self::UnsupportedSemanticProfile { .. } => "unsupported_information_set_profile",
            Self::InvalidSource(error) => error.code(),
            Self::EmptyIdentifier { .. } => "empty_information_set_identifier",
            Self::DuplicateInformationSet(_) => "duplicate_information_set",
            Self::EmptyInformationSet(_) => "empty_information_set",
            Self::DuplicateMember { .. } => "duplicate_information_set_member",
            Self::UnknownNode { .. } => "unknown_information_set_member",
            Self::NonDecisionNode { .. } => "nondecision_information_set_member",
            Self::PlayerMismatch { .. } => "information_set_player_mismatch",
            Self::DuplicateMembership { .. } => "duplicate_information_set_membership",
            Self::UnassignedDecisionNode(_) => "unassigned_information_set_decision",
            Self::ActionMismatch { .. } => "information_set_action_mismatch",
            Self::ImperfectRecall { .. } => "imperfect_recall",
            Self::AmbiguousRecall { .. } => "ambiguous_recall_history",
        }
    }

    #[must_use]
    pub fn path(&self) -> String {
        match self {
            Self::UnsupportedSchemaVersion { .. } => "schema_version".into(),
            Self::UnsupportedSemanticProfile { .. } => "semantic_profile".into(),
            Self::InvalidSource(ProfileValidationError::UnsupportedSemanticProfile { .. }) => {
                "semantic_profile".into()
            }
            Self::InvalidSource(error) => format!("source.{}", error.path()),
            Self::EmptyIdentifier { path } => (*path).into(),
            Self::DuplicateInformationSet(set) => format!("sets.{}.id", set.0),
            Self::EmptyInformationSet(set) => format!("sets.{}.members", set.0),
            Self::DuplicateMember { set, node }
            | Self::UnknownNode { set, node }
            | Self::NonDecisionNode { set, node, .. }
            | Self::PlayerMismatch { set, node, .. }
            | Self::ActionMismatch { set, node, .. }
            | Self::ImperfectRecall { set, node, .. } => {
                format!("sets.{}.members.{}", set.0, node.0)
            }
            Self::DuplicateMembership { node, .. }
            | Self::UnassignedDecisionNode(node)
            | Self::AmbiguousRecall { node, .. } => format!("game.nodes.{}", node.0),
        }
    }
}

/// Admit a complete declared information-set partition of the source graph.
///
/// # Errors
///
/// Returns a deterministic [`Error`] for an unsupported schema or profile, an
/// invalid source graph, malformed memberships, inconsistent legal actions,
/// imperfect recall or an ambiguous own history across shared paths.
pub fn build(_source: &GameSpecification, _declaration: &Declaration) -> Result<Graph, Error> {
    unimplemented!("information-set admission is not implemented")
}
