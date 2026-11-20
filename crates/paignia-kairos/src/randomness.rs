use kairo_ecs_rng::{derive_stream_id, rand_distr::weighted::WeightedIndex, DeterministicStream};
use kairo_ecs_types::EntityId;
use paignia_core::NodeId;

use super::{ExecutionConfig, ExecutionError};

pub const COMPARISON_RANDOMNESS_VERSION: &str = "paignia/comparison-rng/1.0.0";
const CHANCE_PURPOSE: &str = "paignia/game-execution/chance/v1";
// Key-derivation namespace only. The real world entity is bound independently.
const COMPARISON_NAMESPACE: EntityId = EntityId::new(0, 0);

/// Whether an exogenous draw at the same logical game node is paired by policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChanceCoupling {
    SharedAcrossPolicies,
    IndependentByPolicy,
}

/// Stable comparison identities supplied by the application. Equal labels and
/// seeds bind a paired model even when its world entity or session changes.
#[derive(Clone, Copy, Debug)]
pub struct ComparisonRandomness<'a> {
    pub scenario_id: &'a str,
    pub paired_draw_id: &'a str,
    pub chance_coupling: ChanceCoupling,
}

/// Draw interface for a policy. The native generator state stays private.
pub struct ExplorationStream(DeterministicStream);

impl ExplorationStream {
    pub fn next_u64(&mut self) -> u64 {
        self.0.next_u64()
    }

    #[must_use]
    pub fn stream_id(&self) -> u64 {
        self.0.stream_id()
    }

    #[must_use]
    pub fn draw_count(&self) -> u128 {
        self.0.word_pos() / 2
    }
}

pub(super) enum Randomness {
    Entity(Box<DeterministicStream>),
    Comparison(ComparisonStreams),
}

pub(super) struct ComparisonStreams {
    seed: u64,
    chance_key: String,
    exploration_key: String,
}

impl Randomness {
    pub(super) fn entity(seed: u64, entity: EntityId) -> Self {
        Self::Entity(Box::new(DeterministicStream::from_entity_purpose(
            seed,
            entity,
            CHANCE_PURPOSE,
        )))
    }

    pub(super) fn comparison(
        specification: &str,
        execution: &ExecutionConfig<'_>,
        comparison: ComparisonRandomness<'_>,
    ) -> Result<Self, ExecutionError> {
        for (field, value) in [
            ("scenario_id", comparison.scenario_id),
            ("paired_draw_id", comparison.paired_draw_id),
        ] {
            if value.trim().is_empty() {
                return Err(ExecutionError::EmptyIdentity(field));
            }
        }
        let context = address(
            COMPARISON_RANDOMNESS_VERSION,
            [
                specification,
                &execution.game_instance_id.0,
                comparison.scenario_id,
                comparison.paired_draw_id,
            ],
        );
        let policy = execution.policy_artifact_id.0.as_str();
        let chance_key = match comparison.chance_coupling {
            ChanceCoupling::SharedAcrossPolicies => address(&context, ["chance/shared"]),
            ChanceCoupling::IndependentByPolicy => {
                address(&context, ["chance/independent", policy])
            }
        };
        Ok(Self::Comparison(ComparisonStreams {
            seed: execution.seed,
            chance_key,
            exploration_key: address(&context, ["exploration", policy]),
        }))
    }

    pub(super) fn sample_chance(&mut self, node: &NodeId, weights: &WeightedIndex<u128>) -> usize {
        match self {
            Self::Entity(stream) => stream.sample(weights),
            Self::Comparison(streams) => streams
                .stream(&streams.chance_key, node, "categorical-u128-120/v1")
                .sample(weights),
        }
    }

    pub(super) fn exploration(&self, node: &NodeId) -> Result<ExplorationStream, ExecutionError> {
        let Self::Comparison(streams) = self else {
            return Err(ExecutionError::ComparisonRandomnessRequired);
        };
        Ok(ExplorationStream(streams.stream(
            &streams.exploration_key,
            node,
            "raw-u64/v1",
        )))
    }
}

impl ComparisonStreams {
    fn stream(&self, prefix: &str, node: &NodeId, distribution: &str) -> DeterministicStream {
        let purpose = address(prefix, [&node.0, distribution]);
        let id = derive_stream_id(COMPARISON_NAMESPACE, &purpose);
        let mut stream = DeterministicStream::from_seed_u64(self.seed, id);
        stream
            .set_distribution_provenance(format!("{COMPARISON_RANDOMNESS_VERSION}/{distribution}"));
        stream
    }
}

fn address<'a>(prefix: &str, fields: impl IntoIterator<Item = &'a str>) -> String {
    let mut key = prefix.to_owned();
    for field in fields {
        key.push('|');
        key.push_str(&field.len().to_string());
        key.push(':');
        key.push_str(field);
    }
    key
}
