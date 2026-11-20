use std::{collections::BTreeMap, num::NonZeroU64, sync::Arc};

use kairo_ecs_core::{Scheduler, SchedulerCheckpoint, SchedulerStats};
use kairo_ecs_rng::{rand_distr::weighted::WeightedIndex, RngError};
use kairo_ecs_state::World;
use kairo_ecs_types::{EntityId, TypesError, Versioned};
use paignia_core::{GameSpecification, NodeId, ObservationRevision, PolicyArtifactId};
use thiserror::Error;

use super::{
    randomness::{Randomness, RandomnessCheckpoint},
    CausalTraceId, ExecutionIdentity, NativeGame, Phase,
};

pub const REPLAY_CHECKPOINT_VERSION_V1: u16 = 1;

/// Owned policy state encoded by the exact policy artifact's versioned codec.
pub struct PolicySnapshot {
    pub format: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum ReplayError {
    #[error("replay checkpoint version {0} is unsupported")]
    UnsupportedVersion(u16),
    #[error("checkpoint capture requires a committed transition into a chance node")]
    UnsupportedBoundary,
    #[error("the policy snapshot requires a versioned format identifier")]
    EmptyPolicyFormat,
    #[error("the policy artifact or state format does not match the snapshot")]
    PolicyBindingMismatch,
    #[error("the policy rejected the captured state: {0}")]
    PolicyRestoreRejected(String),
    #[error("resume requires a fresh native scheduler")]
    SchedulerNotFresh,
    #[error(transparent)]
    Scheduler(#[from] TypesError),
    #[error(transparent)]
    Randomness(#[from] RngError),
}

/// An in-process checkpoint bound to the original immutable model and world.
/// Each successful resume creates independent native and policy state.
pub struct ReplayCheckpoint<'a> {
    pub version: u16,
    specification: &'a GameSpecification,
    world: &'a World,
    scheduler: Versioned<SchedulerCheckpoint>,
    identity: ExecutionIdentity,
    entity: EntityId,
    randomness: RandomnessCheckpoint,
    chance_weights: BTreeMap<&'a NodeId, WeightedIndex<u128>>,
    current: &'a NodeId,
    revision: ObservationRevision,
    transition_limit: NonZeroU64,
    policy: PolicySnapshot,
}

impl<'a> NativeGame<'a> {
    /// Capture the game and the host's policy state after a committed
    /// transition into a chance node.
    ///
    /// # Errors
    /// Rejects uncommitted or interrupted boundaries and unnamed policy formats.
    pub fn checkpoint(&self, policy: PolicySnapshot) -> Result<ReplayCheckpoint<'a>, ReplayError> {
        if self.interrupted || self.revision.0 == 0 || self.phase() != Phase::Chance {
            return Err(ReplayError::UnsupportedBoundary);
        }
        if policy.format.trim().is_empty() {
            return Err(ReplayError::EmptyPolicyFormat);
        }
        Ok(ReplayCheckpoint {
            version: REPLAY_CHECKPOINT_VERSION_V1,
            specification: self.specification,
            world: self.world,
            scheduler: self.scheduler.checkpoint(),
            identity: self.identity.clone(),
            entity: self.entity,
            randomness: self.randomness.checkpoint(),
            chance_weights: self.chance_weights.clone(),
            current: self.current,
            revision: self.revision,
            transition_limit: self.transition_limit,
            policy,
        })
    }
}

impl ReplayCheckpoint<'_> {
    /// Restore native execution and decode an independent policy state on a
    /// fresh scheduler. The captured world remains borrowed for the branch.
    ///
    /// # Errors
    /// Rejects incompatible versions, policy or codec mismatches, decoder
    /// rejection, and a scheduler that has already been used. All validation
    /// and decoding precede mutation of the supplied scheduler.
    pub fn resume<'a, P>(
        &'a self,
        scheduler: &'a mut Scheduler,
        policy_artifact: &PolicyArtifactId,
        policy_format: &str,
        restore_policy: impl FnOnce(&[u8]) -> Result<P, String>,
    ) -> Result<(NativeGame<'a>, P), ReplayError> {
        if self.version != REPLAY_CHECKPOINT_VERSION_V1 {
            return Err(ReplayError::UnsupportedVersion(self.version));
        }
        if policy_artifact != &self.identity.policy_artifact_id
            || policy_format != self.policy.format
        {
            return Err(ReplayError::PolicyBindingMismatch);
        }
        if scheduler.stats() != SchedulerStats::default() {
            return Err(ReplayError::SchedulerNotFresh);
        }
        let restored = Scheduler::from_checkpoint(&self.scheduler)?;
        let randomness = Randomness::from_checkpoint(&self.randomness)?;
        let policy =
            restore_policy(&self.policy.bytes).map_err(ReplayError::PolicyRestoreRejected)?;
        let mut identity = self.identity.clone();
        identity.trace_id = CausalTraceId(Arc::new(()));
        let chance_weights = self.chance_weights.clone();
        *scheduler = restored;
        Ok((
            NativeGame {
                specification: self.specification,
                world: self.world,
                scheduler,
                identity,
                entity: self.entity,
                randomness,
                chance_weights,
                current: self.current,
                revision: self.revision,
                transition_limit: self.transition_limit,
                interrupted: false,
            },
            policy,
        ))
    }
}
