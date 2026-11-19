//! Domain-neutral finite information-set admission over the static graph profile.
//!
//! The base [`crate::Game`] retains its bounded perfect-information profile.
//! This module adds a separate declared partition of decision nodes; it does not
//! add hidden state, Kairos storage, ontology semantics or solver behavior.

use std::collections::{BTreeMap, BTreeSet};

use thiserror::Error;

use crate::{
    ActionEdge, ActionId, GameSpecification, GameSpecificationId, Node, NodeId, PlayerId,
    ProfileValidationError, STATIC_GAME_PROFILE_V1,
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

/// A player's own decision history, interned so equality is constant time.
type HistoryId = usize;
type HistoryStep = (InformationSetId, ActionId);
type HistoryKey = (HistoryId, Option<HistoryStep>);
type HistoryTable = BTreeMap<HistoryKey, HistoryId>;
type AdmittedSets = BTreeMap<InformationSetId, Set>;
type Assignments = BTreeMap<NodeId, InformationSetId>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Recall {
    Unset,
    Unique(HistoryId),
    Ambiguous,
}

/// Admit a complete declared information-set partition of the source graph.
///
/// The source is first validated under [`STATIC_GAME_PROFILE_V1`] and the
/// declaration must name [`crate::INFORMATION_SET_PROFILE_V1`]. Every decision
/// node must belong to exactly one set; every set is owned by one player, has a
/// nonempty member list and requires identical ordered legal actions across its
/// members. Perfect recall is required: all members of a set must share the
/// same preceding sequence of that player's own information sets and actions,
/// and every decision reached through shared paths must have a unique such
/// history. Members are returned in canonical node order; declared action order
/// is preserved.
///
/// # Errors
///
/// Errors are deterministic in this order: declaration schema; semantic
/// profile; source profile/schema/graph; declaration structure and membership
/// in declaration order; unassigned decision coverage in source node order;
/// legal-action consistency in set order; ambiguous shared histories in source
/// node order; imperfect recall in set order. Inputs are never mutated.
pub fn build(source: &GameSpecification, declaration: &Declaration) -> Result<Graph, Error> {
    if declaration.schema_version != SCHEMA_VERSION {
        return Err(Error::UnsupportedSchemaVersion {
            value: declaration.schema_version.clone(),
        });
    }
    if declaration.semantic_profile != crate::INFORMATION_SET_PROFILE_V1 {
        return Err(Error::UnsupportedSemanticProfile {
            value: declaration.semantic_profile.clone(),
        });
    }
    source.validate_for_profile(STATIC_GAME_PROFILE_V1)?;

    let (sets, assignments) = admit_declarations(source, declaration)?;
    ensure_complete_coverage(source, &assignments)?;
    ensure_shared_actions(source, &sets)?;
    ensure_perfect_recall(source, &sets, &assignments)?;

    Ok(Graph {
        schema_version: declaration.schema_version.clone(),
        semantic_profile: declaration.semantic_profile.clone(),
        specification_id: GameSpecificationId(source.id.clone()),
        sets,
        assignments,
    })
}

fn admit_declarations(
    source: &GameSpecification,
    declaration: &Declaration,
) -> Result<(AdmittedSets, Assignments), Error> {
    let mut sets = BTreeMap::new();
    let mut assignments: Assignments = BTreeMap::new();
    let mut identifiers = BTreeSet::new();

    for declared in &declaration.sets {
        if declared.id.0.trim().is_empty() {
            return Err(Error::EmptyIdentifier { path: "sets.id" });
        }
        if !identifiers.insert(declared.id.clone()) {
            return Err(Error::DuplicateInformationSet(declared.id.clone()));
        }
        if declared.player.0.trim().is_empty() {
            return Err(Error::EmptyIdentifier {
                path: "sets.player",
            });
        }
        if declared.members.is_empty() {
            return Err(Error::EmptyInformationSet(declared.id.clone()));
        }

        let mut members = BTreeSet::new();
        for member in &declared.members {
            if member.0.trim().is_empty() {
                return Err(Error::EmptyIdentifier {
                    path: "sets.members",
                });
            }
            let node = source
                .game
                .nodes
                .get(member)
                .ok_or_else(|| Error::UnknownNode {
                    set: declared.id.clone(),
                    node: member.clone(),
                })?;
            let owner = match node {
                Node::Decision { player, .. } => player,
                Node::Chance { .. } => {
                    return Err(Error::NonDecisionNode {
                        set: declared.id.clone(),
                        node: member.clone(),
                        kind: RejectedNodeKind::Chance,
                    })
                }
                Node::Terminal { .. } => {
                    return Err(Error::NonDecisionNode {
                        set: declared.id.clone(),
                        node: member.clone(),
                        kind: RejectedNodeKind::Terminal,
                    })
                }
            };
            if owner != &declared.player {
                return Err(Error::PlayerMismatch {
                    set: declared.id.clone(),
                    node: member.clone(),
                    expected: declared.player.clone(),
                    actual: owner.clone(),
                });
            }
            if !members.insert(member.clone()) {
                return Err(Error::DuplicateMember {
                    set: declared.id.clone(),
                    node: member.clone(),
                });
            }
            if let Some(first) = assignments.get(member) {
                return Err(Error::DuplicateMembership {
                    node: member.clone(),
                    first: first.clone(),
                    second: declared.id.clone(),
                });
            }
            assignments.insert(member.clone(), declared.id.clone());
        }

        let reference = members.first().expect("nonempty membership checked above");
        let legal_actions = legal_actions(source, reference);
        sets.insert(
            declared.id.clone(),
            Set {
                player: declared.player.clone(),
                legal_actions,
                members: members.into_iter().collect(),
            },
        );
    }

    Ok((sets, assignments))
}

fn action_edges<'game>(source: &'game GameSpecification, node: &NodeId) -> &'game [ActionEdge] {
    match &source.game.nodes[node] {
        Node::Decision { edges, .. } => edges,
        Node::Chance { .. } | Node::Terminal { .. } => {
            panic!("membership admitted only decision nodes")
        }
    }
}

fn legal_actions(source: &GameSpecification, node: &NodeId) -> Vec<ActionId> {
    action_edges(source, node)
        .iter()
        .map(|edge| edge.action.clone())
        .collect()
}

fn ensure_complete_coverage(
    source: &GameSpecification,
    assignments: &Assignments,
) -> Result<(), Error> {
    for (node_id, node) in &source.game.nodes {
        if matches!(node, Node::Decision { .. }) && !assignments.contains_key(node_id) {
            return Err(Error::UnassignedDecisionNode(node_id.clone()));
        }
    }
    Ok(())
}

fn ensure_shared_actions(source: &GameSpecification, sets: &AdmittedSets) -> Result<(), Error> {
    for (set_id, set) in sets {
        let reference = &set.members[0];
        let expected = legal_actions(source, reference);
        for member in &set.members[1..] {
            if legal_actions(source, member) != expected {
                return Err(Error::ActionMismatch {
                    set: set_id.clone(),
                    node: member.clone(),
                    reference: reference.clone(),
                });
            }
        }
    }
    Ok(())
}

fn ensure_perfect_recall(
    source: &GameSpecification,
    sets: &AdmittedSets,
    assignments: &Assignments,
) -> Result<(), Error> {
    let players = sets
        .values()
        .map(|set| set.player.clone())
        .collect::<BTreeSet<_>>();
    let incoming = incoming_edges(source);
    let order = topological_order(source);
    let mut histories = BTreeMap::new();
    let mut interned = HistoryTable::new();

    for node_id in &order {
        for player in &players {
            let state = recall_state(
                source,
                node_id,
                player,
                &incoming,
                assignments,
                &histories,
                &mut interned,
            );
            histories.insert((node_id.clone(), player.clone()), state);
        }
    }

    for (node_id, node) in &source.game.nodes {
        if let Node::Decision { player, .. } = node {
            if histories.get(&(node_id.clone(), player.clone())) == Some(&Recall::Ambiguous) {
                return Err(Error::AmbiguousRecall {
                    node: node_id.clone(),
                    player: player.clone(),
                });
            }
        }
    }

    for (set_id, set) in sets {
        let reference = &set.members[0];
        let expected = histories
            .get(&(reference.clone(), set.player.clone()))
            .copied();
        for member in &set.members[1..] {
            let actual = histories
                .get(&(member.clone(), set.player.clone()))
                .copied();
            if actual != expected {
                return Err(Error::ImperfectRecall {
                    set: set_id.clone(),
                    node: member.clone(),
                    reference: reference.clone(),
                });
            }
        }
    }

    Ok(())
}

fn recall_state(
    source: &GameSpecification,
    node_id: &NodeId,
    player: &PlayerId,
    incoming: &BTreeMap<NodeId, Vec<(NodeId, Option<ActionId>)>>,
    assignments: &Assignments,
    histories: &BTreeMap<(NodeId, PlayerId), Recall>,
    interned: &mut HistoryTable,
) -> Recall {
    let Some(parents) = incoming.get(node_id) else {
        return Recall::Unique(0);
    };
    let mut state = Recall::Unset;
    for (parent, action) in parents {
        let parent_state = histories
            .get(&(parent.clone(), player.clone()))
            .copied()
            .expect("topological order processes parents first");
        let contribution = match parent_state {
            Recall::Ambiguous => Recall::Ambiguous,
            Recall::Unique(history) => {
                let parent_node = &source.game.nodes[parent];
                match parent_node {
                    Node::Decision { player: owner, .. } if owner == player => {
                        let information_set =
                            assignments.get(parent).expect("complete coverage admitted");
                        let action = action.as_ref().expect("decision edge carries an action");
                        let key = (history, Some((information_set.clone(), action.clone())));
                        let next = if let Some(existing) = interned.get(&key) {
                            *existing
                        } else {
                            let next = interned.len() + 1;
                            interned.insert(key, next);
                            next
                        };
                        Recall::Unique(next)
                    }
                    Node::Decision { .. } | Node::Chance { .. } | Node::Terminal { .. } => {
                        Recall::Unique(history)
                    }
                }
            }
            Recall::Unset => unreachable!("parents are processed before their children"),
        };
        state = match (state, contribution) {
            (Recall::Unset, next) => next,
            (current, next) if current == next => current,
            _ => Recall::Ambiguous,
        };
    }
    if state == Recall::Unset {
        Recall::Unique(0)
    } else {
        state
    }
}

fn incoming_edges(source: &GameSpecification) -> BTreeMap<NodeId, Vec<(NodeId, Option<ActionId>)>> {
    let mut incoming: BTreeMap<NodeId, Vec<(NodeId, Option<ActionId>)>> = BTreeMap::new();
    for (parent, node) in &source.game.nodes {
        match node {
            Node::Decision { edges, .. } => {
                for edge in edges {
                    incoming
                        .entry(edge.target.clone())
                        .or_default()
                        .push((parent.clone(), Some(edge.action.clone())));
                }
            }
            Node::Chance { edges } => {
                for edge in edges {
                    incoming
                        .entry(edge.target.clone())
                        .or_default()
                        .push((parent.clone(), None));
                }
            }
            Node::Terminal { .. } => {}
        }
    }
    incoming
}

fn topological_order(source: &GameSpecification) -> Vec<NodeId> {
    let mut indegree = source
        .game
        .nodes
        .keys()
        .map(|node| (node.clone(), 0_usize))
        .collect::<BTreeMap<_, _>>();
    for node in source.game.nodes.values() {
        visit_targets(node, &mut |target| {
            *indegree.get_mut(target).expect("validated targets exist") += 1;
        });
    }
    let mut ready = indegree
        .iter()
        .filter(|(_, degree)| **degree == 0)
        .map(|(node, _)| node.clone())
        .collect::<BTreeSet<_>>();
    let mut order = Vec::with_capacity(source.game.nodes.len());
    while let Some(node_id) = ready.pop_first() {
        order.push(node_id.clone());
        let node = &source.game.nodes[&node_id];
        visit_targets(node, &mut |target| {
            let degree = indegree.get_mut(target).expect("validated targets exist");
            *degree -= 1;
            if *degree == 0 {
                ready.insert(target.clone());
            }
        });
    }
    debug_assert_eq!(order.len(), source.game.nodes.len());
    order
}

fn visit_targets(node: &Node, visit: &mut impl FnMut(&NodeId)) {
    match node {
        Node::Decision { edges, .. } => {
            for edge in edges {
                visit(&edge.target);
            }
        }
        Node::Chance { edges } => {
            for edge in edges {
                visit(&edge.target);
            }
        }
        Node::Terminal { .. } => {}
    }
}
