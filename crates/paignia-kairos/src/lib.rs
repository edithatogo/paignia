//! Execute one finite perfect-information game through a native Kairos scheduler.
//!
//! The host supplies a policy action after seeing a [`PolicyObservation`]. Each
//! action or chance outcome advances the scheduler by one tick and yields the
//! event actually dispatched. The host owns model, policy and entity bindings.

use std::{collections::BTreeMap, num::NonZeroU64, sync::Arc};

use kairo_ecs_core::Scheduler;
use kairo_ecs_rng::{
    rand_distr::weighted::{Error as WeightError, WeightedIndex},
    DeterministicStream,
};
use kairo_ecs_state::World;
use kairo_ecs_types::{
    DispatchedEvent, EntityId, EventId, EventKind, ScheduleRequest, SimDuration, SimTime,
    StepOutcome,
};
use paignia_core::{
    build_policy_observation, ActionId, ChanceEdge, GameInstanceId, GameSessionId,
    GameSpecification, GameSpecificationId, Node, NodeId, ObservationRevision, PayoffVector,
    PolicyArtifactId, PolicyObservation, PolicyObservationBinding, PolicyObservationError,
    PolicyObservationRequest, ProfileValidationError, POLICY_OBSERVATION_SCHEMA_VERSION,
    STATIC_GAME_PROFILE_V1,
};
use thiserror::Error;

pub const EXECUTION_SCHEMA_VERSION: &str = "1.0.0";
pub const DECISION_EVENT: EventKind = EventKind::custom(u32::from_be_bytes(*b"PGAC"));
pub const CHANCE_EVENT: EventKind = EventKind::custom(u32::from_be_bytes(*b"PGCH"));
const CHANCE_PURPOSE: &str = "paignia/game-execution/chance/v1";
// Binary64 encoding of 2^120, leaving room for the admitted total in u128.
const CHANCE_WEIGHT_SCALE: f64 = f64::from_bits((1023 + 120) << 52);

/// One host-supplied run binding. The entity is interpreted in the supplied world.
#[derive(Clone, Debug)]
pub struct ExecutionConfig<'a> {
    pub schema_version: &'a str,
    pub semantic_profile: &'a str,
    pub game_instance_id: GameInstanceId,
    pub session_id: GameSessionId,
    pub policy_artifact_id: PolicyArtifactId,
    pub entity: EntityId,
    pub seed: u64,
    pub transition_limit: NonZeroU64,
}

/// Opaque in-process identity of one causal execution, local to this binding.
#[derive(Clone, Debug)]
pub struct CausalTraceId(Arc<()>);

impl PartialEq for CausalTraceId {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for CausalTraceId {}

/// The live trace identity and host-supplied labels of a committed transition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionIdentity {
    pub trace_id: CausalTraceId,
    pub specification_id: GameSpecificationId,
    pub game_instance_id: GameInstanceId,
    pub session_id: GameSessionId,
    pub policy_artifact_id: PolicyArtifactId,
}

/// A decision issued by a particular live run for one observation revision.
#[derive(Clone, Debug)]
pub struct DecisionTurn {
    observation: PolicyObservation,
    trace_id: CausalTraceId,
}

impl DecisionTurn {
    #[must_use]
    pub const fn observation(&self) -> &PolicyObservation {
        &self.observation
    }
}

/// One action submission. The issued turn identifies a single commit slot.
#[derive(Clone, Copy, Debug)]
pub enum Command<'a> {
    Act {
        turn: &'a DecisionTurn,
        action: &'a ActionId,
        at: SimTime,
    },
    /// The first execution profile rejects cancellation without changing state.
    Cancel,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Phase {
    Decision,
    Chance,
    Terminal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransitionOutcome {
    Action(ActionId),
    /// Index in the original ordered chance edge list.
    Chance {
        edge_index: usize,
    },
}

/// The realized transition, with its event identity assigned by Kairos.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalTransition {
    pub identity: ExecutionIdentity,
    pub event: DispatchedEvent,
    pub from: NodeId,
    pub to: NodeId,
    pub before: ObservationRevision,
    pub after: ObservationRevision,
    pub outcome: TransitionOutcome,
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum ExecutionError {
    #[error("unsupported native execution schema version {0:?}")]
    UnsupportedSchemaVersion(String),
    #[error(transparent)]
    InvalidSpecification(#[from] ProfileValidationError),
    #[error("{0} must be a non-blank identifier")]
    EmptyIdentity(&'static str),
    #[error("entity {0:?} is inactive in the supplied world")]
    InactiveEntity(EntityId),
    #[error("native execution requires a scheduler without live pending events")]
    SchedulerBusy,
    #[error("the game is outside a decision phase")]
    NotAtDecision,
    #[error("the game is outside a chance phase")]
    NotAtChance,
    #[error(transparent)]
    Observation(#[from] PolicyObservationError),
    #[error("the issued turn belongs to another live session or revision")]
    StaleDecision,
    #[error("action {0:?} is outside the current decision's legal actions")]
    UnknownAction(ActionId),
    #[error("the execution profile rejects cancellation")]
    UnsupportedCancellation,
    #[error("the execution profile requires {required:?}, received {at:?}")]
    UnsupportedTime { at: SimTime, required: SimTime },
    #[error("the next native tick is outside the representable time range")]
    TimeOverflow,
    #[error("the native transition limit has been reached")]
    TransitionLimit,
    #[error("the native scheduler event counter is exhausted")]
    EventCounterExhausted,
    #[error("invalid native chance weights: {0}")]
    ChanceWeights(#[from] WeightError),
    #[error("chance edge {edge} at {node:?} is outside the exact native weight grid")]
    UnrepresentableChanceWeight { node: NodeId, edge: usize },
    #[error(
        "Kairos dispatch for {scheduled:?} did not match the submitted transition: {outcome:?}"
    )]
    UnexpectedDispatch {
        scheduled: EventId,
        outcome: StepOutcome,
    },
    #[error("the run stopped after a native dispatch invariant failed")]
    DispatchInterrupted,
}

/// Drives one game with exclusive access to a native scheduler. The world is
/// borrowed for the run so the admitted entity cannot retire between operations.
pub struct NativeGame<'a> {
    specification: &'a GameSpecification,
    _world: &'a World,
    scheduler: &'a mut Scheduler,
    identity: ExecutionIdentity,
    entity: EntityId,
    chance: DeterministicStream,
    chance_weights: BTreeMap<&'a NodeId, WeightedIndex<u128>>,
    current: &'a NodeId,
    revision: ObservationRevision,
    transition_limit: NonZeroU64,
    interrupted: bool,
}

impl<'a> NativeGame<'a> {
    /// Bind an immutable game and live entity to an idle scheduler.
    ///
    /// # Errors
    /// Rejects unsupported schemas or game profiles, invalid graphs, blank run
    /// identifiers, inactive entity handles, schedulers with pending work and
    /// chance weights outside the exact native grid.
    pub fn start(
        specification: &'a GameSpecification,
        world: &'a World,
        scheduler: &'a mut Scheduler,
        config: ExecutionConfig<'_>,
    ) -> Result<Self, ExecutionError> {
        if config.schema_version != EXECUTION_SCHEMA_VERSION {
            return Err(ExecutionError::UnsupportedSchemaVersion(
                config.schema_version.into(),
            ));
        }
        specification.validate_for_profile(config.semantic_profile)?;
        for (field, id) in [
            ("game_instance_id", &config.game_instance_id.0),
            ("session_id", &config.session_id.0),
            ("policy_artifact_id", &config.policy_artifact_id.0),
        ] {
            if id.trim().is_empty() {
                return Err(ExecutionError::EmptyIdentity(field));
            }
        }
        if !world.is_alive(config.entity) {
            return Err(ExecutionError::InactiveEntity(config.entity));
        }
        if scheduler.pending_events() != 0 {
            return Err(ExecutionError::SchedulerBusy);
        }
        let mut chance_weights = BTreeMap::new();
        for (node, state) in &specification.game.nodes {
            if let Node::Chance { edges } = state {
                chance_weights.insert(node, native_chance_weights(node, edges)?);
            }
        }
        Ok(Self {
            specification,
            _world: world,
            scheduler,
            identity: ExecutionIdentity {
                trace_id: CausalTraceId(Arc::new(())),
                specification_id: GameSpecificationId(specification.id.clone()),
                game_instance_id: config.game_instance_id,
                session_id: config.session_id,
                policy_artifact_id: config.policy_artifact_id,
            },
            entity: config.entity,
            chance: DeterministicStream::from_entity_purpose(
                config.seed,
                config.entity,
                CHANCE_PURPOSE,
            ),
            chance_weights,
            current: &specification.game.root,
            revision: ObservationRevision(0),
            transition_limit: config.transition_limit,
            interrupted: false,
        })
    }

    #[must_use]
    pub fn phase(&self) -> Phase {
        match self.current_node() {
            Node::Decision { .. } => Phase::Decision,
            Node::Chance { .. } => Phase::Chance,
            Node::Terminal { .. } => Phase::Terminal,
        }
    }

    /// The only effective time accepted for the next transition.
    ///
    /// # Errors
    /// Returns an error if there is no representable next native tick or the
    /// run has already stopped on a dispatch invariant failure.
    pub fn next_transition_time(&self) -> Result<SimTime, ExecutionError> {
        if self.interrupted {
            return Err(ExecutionError::DispatchInterrupted);
        }
        self.scheduler
            .now()
            .checked_add(SimDuration::from_ticks(1))
            .ok_or(ExecutionError::TimeOverflow)
    }

    /// Issue the owned observation for the current decision.
    ///
    /// # Errors
    /// Returns an error outside decision phases or when observation admission
    /// or a native dispatch invariant has failed.
    pub fn observe(&self) -> Result<DecisionTurn, ExecutionError> {
        if self.interrupted {
            return Err(ExecutionError::DispatchInterrupted);
        }
        let Node::Decision { player, .. } = self.current_node() else {
            return Err(ExecutionError::NotAtDecision);
        };
        let request = PolicyObservationRequest {
            schema_version: POLICY_OBSERVATION_SCHEMA_VERSION.into(),
            semantic_profile: STATIC_GAME_PROFILE_V1.into(),
            binding: PolicyObservationBinding {
                specification_id: self.identity.specification_id.clone(),
                game_instance_id: self.identity.game_instance_id.clone(),
                session_id: self.identity.session_id.clone(),
                player_id: player.clone(),
                decision_node_id: self.current.clone(),
                revision: self.revision,
            },
        };
        Ok(DecisionTurn {
            observation: build_policy_observation(self.specification, &request.binding, &request)?,
            trace_id: self.identity.trace_id.clone(),
        })
    }

    /// Commit the action selected by the host for this turn.
    ///
    /// # Errors
    /// Rejects cancellation, stale or foreign turns, illegal actions, timing
    /// mismatches and exhausted counters before scheduling an event. A native
    /// dispatch mismatch stops the run.
    pub fn submit(&mut self, command: Command<'_>) -> Result<CausalTransition, ExecutionError> {
        let Command::Act { turn, action, at } = command else {
            return Err(ExecutionError::UnsupportedCancellation);
        };
        if self.identity.trace_id != turn.trace_id
            || turn.observation.binding.revision != self.revision
        {
            return Err(ExecutionError::StaleDecision);
        }
        let Node::Decision { edges, .. } = self.current_node() else {
            return Err(ExecutionError::NotAtDecision);
        };
        let edge = edges
            .iter()
            .find(|edge| &edge.action == action)
            .ok_or_else(|| ExecutionError::UnknownAction(action.clone()))?;
        self.validate_transition(at)?;
        self.dispatch(
            at,
            &edge.target,
            TransitionOutcome::Action(action.clone()),
            DECISION_EVENT,
        )
    }

    /// Draw and commit one outcome from the ordered weights of a chance node.
    ///
    /// # Errors
    /// Rejects a non-chance phase, unsupported timing and exhausted counters
    /// before drawing. A dispatch mismatch stops the run.
    pub fn realize_chance(&mut self, at: SimTime) -> Result<CausalTransition, ExecutionError> {
        let Node::Chance { edges } = self.current_node() else {
            return Err(ExecutionError::NotAtChance);
        };
        self.validate_transition(at)?;
        let edge_index = self.chance.sample(&self.chance_weights[self.current]);
        self.dispatch(
            at,
            &edges[edge_index].target,
            TransitionOutcome::Chance { edge_index },
            CHANCE_EVENT,
        )
    }

    /// The sparse payoff vector at the realized terminal.
    #[must_use]
    pub fn terminal_payoffs(&self) -> Option<&PayoffVector> {
        match self.current_node() {
            Node::Terminal { payoffs } => Some(payoffs),
            _ => None,
        }
    }

    fn current_node(&self) -> &'a Node {
        &self.specification.game.nodes[self.current]
    }

    fn validate_transition(&self, at: SimTime) -> Result<(), ExecutionError> {
        let required = self.next_transition_time()?;
        if at != required {
            return Err(ExecutionError::UnsupportedTime { at, required });
        }
        if self.revision.0 == self.transition_limit.get() {
            return Err(ExecutionError::TransitionLimit);
        }
        if self.scheduler.stats().scheduled_events == u64::MAX {
            return Err(ExecutionError::EventCounterExhausted);
        }
        Ok(())
    }

    fn dispatch(
        &mut self,
        at: SimTime,
        target: &'a NodeId,
        outcome: TransitionOutcome,
        kind: EventKind,
    ) -> Result<CausalTransition, ExecutionError> {
        let entity = Some(self.entity);
        let scheduled = self.scheduler.schedule(ScheduleRequest {
            at,
            priority: 0,
            entity,
            kind,
        });
        let native = self.scheduler.step();
        let event = match native {
            StepOutcome::Dispatched(event)
                if event.id == scheduled
                    && event.at == at
                    && event.entity == entity
                    && event.kind == kind =>
            {
                event
            }
            _ => {
                self.interrupted = true;
                return Err(ExecutionError::UnexpectedDispatch {
                    scheduled,
                    outcome: native,
                });
            }
        };
        let transition = CausalTransition {
            identity: self.identity.clone(),
            event,
            from: self.current.clone(),
            to: target.clone(),
            before: self.revision,
            after: ObservationRevision(self.revision.0 + 1),
            outcome,
        };
        self.current = target;
        self.revision = transition.after;
        Ok(transition)
    }
}

fn native_chance_weights(
    node: &NodeId,
    edges: &[ChanceEdge],
) -> Result<WeightedIndex<u128>, ExecutionError> {
    let mut weights = Vec::with_capacity(edges.len());
    for (index, edge) in edges.iter().enumerate() {
        let scaled = edge.probability * CHANCE_WEIGHT_SCALE;
        if scaled.fract() != 0.0 {
            return Err(ExecutionError::UnrepresentableChanceWeight {
                node: node.clone(),
                edge: index,
            });
        }
        // Profile admission bounds every weight to [0, 1 + mass tolerance].
        // Power-of-two scaling and the integral check make this cast exact.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let units = scaled as u128;
        weights.push(units);
    }
    Ok(WeightedIndex::new(weights)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rare_edge_has_exact_positive_weight_in_the_native_sampler() {
        let target = NodeId("terminal".into());
        let rare = f64::EPSILON / 2.0;
        let edges = [1.0 - rare, rare].map(|probability| ChanceEdge {
            probability,
            target: target.clone(),
        });
        let distribution = native_chance_weights(&target, &edges).unwrap();
        assert_eq!(distribution.total_weight(), 1 << 120);
        assert_eq!(distribution.weight(1), Some(1 << 67));
    }
}
