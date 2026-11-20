use std::num::NonZeroU64;

use kairo_ecs_core::Scheduler;
use kairo_ecs_state::World;
use kairo_ecs_types::{EntityId, EventKind, ScheduleRequest, SimTime};
use paignia_core::{
    ActionEdge, Game, GameInstanceId, GameSessionId, GameSpecification, Node, NodeId, PlayerId,
    PolicyArtifactId, GAME_SPEC_SCHEMA_VERSION, STATIC_GAME_PROFILE_V1,
};
use paignia_kairos::{
    CausalTransition, ChanceCoupling, Command, ComparisonRandomness, ExecutionConfig, NativeGame,
    PolicySnapshot, ReplayError, TransitionOutcome, EXECUTION_SCHEMA_VERSION,
};

const POLICY_FORMAT: &str = "test/counter-u64-le/1";

fn node(name: &str) -> NodeId {
    NodeId(name.into())
}

fn policy_id() -> PolicyArtifactId {
    PolicyArtifactId("policy:counter".into())
}

fn model() -> GameSpecification {
    let player = PlayerId("controller".into());
    let decision = |choices: &[(&str, &str)]| Node::Decision {
        player: player.clone(),
        edges: choices
            .iter()
            .map(|(action, target)| ActionEdge {
                action: paignia_core::ActionId((*action).into()),
                target: node(target),
            })
            .collect(),
    };
    let chance = |target: &str| Node::Chance {
        edges: vec![
            paignia_core::ChanceEdge {
                probability: 0.5,
                target: node(target),
            };
            2
        ],
    };
    let payoff = |value| Node::Terminal {
        payoffs: [(player.clone(), value)].into(),
    };
    GameSpecification {
        schema_version: GAME_SPEC_SCHEMA_VERSION.into(),
        id: "game:checkpoint".into(),
        game: Game {
            root: node("start"),
            nodes: [
                (node("start"), decision(&[("begin", "warmup")])),
                (node("warmup"), chance("next draw")),
                (node("next draw"), chance("choose again")),
                (
                    node("choose again"),
                    decision(&[("left", "A"), ("right", "B")]),
                ),
                (node("A"), payoff(-2.0)),
                (node("B"), payoff(5.0)),
            ]
            .into(),
        },
    }
}

fn config(entity: EntityId, seed: u64) -> ExecutionConfig<'static> {
    ExecutionConfig {
        schema_version: EXECUTION_SCHEMA_VERSION,
        semantic_profile: STATIC_GAME_PROFILE_V1,
        game_instance_id: GameInstanceId("instance:checkpoint".into()),
        session_id: GameSessionId("session:checkpoint".into()),
        policy_artifact_id: policy_id(),
        entity,
        seed,
        transition_limit: NonZeroU64::new(4).unwrap(),
    }
}

#[derive(Default)]
struct CounterPolicy(u64);

impl CounterPolicy {
    fn act(&mut self, game: &mut NativeGame<'_>) -> CausalTransition {
        let turn = game.observe().unwrap();
        let choices = &turn.observation().legal_actions;
        let index = usize::try_from(self.0).unwrap() % choices.len();
        self.0 += 1;
        let at = game.next_transition_time().unwrap();
        game.submit(Command::Act {
            turn: &turn,
            action: &choices[index],
            at,
        })
        .unwrap()
    }

    fn snapshot(&self) -> PolicySnapshot {
        PolicySnapshot {
            format: POLICY_FORMAT.into(),
            bytes: self.0.to_le_bytes().to_vec(),
        }
    }

    fn decode(bytes: &[u8]) -> Result<Self, String> {
        let bytes: [u8; 8] = bytes.try_into().map_err(|_| "invalid counter bytes")?;
        Ok(Self(u64::from_le_bytes(bytes)))
    }
}

fn sample(game: &mut NativeGame<'_>) -> CausalTransition {
    let at = game.next_transition_time().unwrap();
    game.realize_chance(at).unwrap()
}

fn assert_replay(mut replay: CausalTransition, original: &CausalTransition) {
    assert_ne!(replay.identity.trace_id, original.identity.trace_id);
    replay.identity.trace_id = original.identity.trace_id.clone();
    assert_eq!(&replay, original);
}

#[test]
fn native_resume_restores_rng_and_policy_and_isolates_siblings() {
    exercise_replay(None);
}

#[test]
fn comparison_resume_preserves_paired_chance_and_future_policy_streams() {
    exercise_replay(Some(ComparisonRandomness {
        scenario_id: "scenario:resume",
        paired_draw_id: "draw:9",
        chance_coupling: ChanceCoupling::SharedAcrossPolicies,
    }));
}

fn exercise_replay(comparison: Option<ComparisonRandomness<'_>>) {
    let source = model();
    let mut world = World::new();
    let retired = world.spawn();
    assert!(world.despawn(retired));
    let entity = world.spawn();
    let mut differing_second_draws = 0;

    for seed in 0..64 {
        let mut original_scheduler = Scheduler::new();
        let mut original = match comparison {
            Some(comparison) => NativeGame::start_comparison(
                &source,
                &world,
                &mut original_scheduler,
                config(entity, seed),
                comparison,
            ),
            None => NativeGame::start(
                &source,
                &world,
                &mut original_scheduler,
                config(entity, seed),
            ),
        }
        .unwrap();
        let mut policy = CounterPolicy::default();
        policy.act(&mut original);
        let first_draw = sample(&mut original);
        let checkpoint = original.checkpoint(policy.snapshot()).unwrap();
        assert_eq!(policy.0, 1);

        let mut sibling_scheduler = Scheduler::new();
        let (mut sibling, mut sibling_policy) = checkpoint
            .resume(
                &mut sibling_scheduler,
                &policy_id(),
                POLICY_FORMAT,
                CounterPolicy::decode,
            )
            .unwrap();
        let sibling_draw = sample(&mut sibling);
        sibling_policy.0 = 0;
        assert_eq!(sibling_policy.act(&mut sibling).to, node("A"));

        let later_draw = sample(&mut original);
        let exploration_draw = comparison.map(|_| {
            let turn = original.observe().unwrap();
            original.exploration_stream(&turn).unwrap().next_u64()
        });
        let later_action = policy.act(&mut original);
        assert!(matches!(
            later_draw.outcome,
            TransitionOutcome::Chance { .. }
        ));
        differing_second_draws += usize::from(first_draw.outcome != later_draw.outcome);
        assert_replay(sibling_draw, &later_draw);
        assert_eq!(later_action.to, node("B"));

        let mut recovered_scheduler = Scheduler::new();
        let (mut recovered, mut recovered_policy) = checkpoint
            .resume(
                &mut recovered_scheduler,
                &policy_id(),
                POLICY_FORMAT,
                CounterPolicy::decode,
            )
            .unwrap();
        assert_eq!(recovered_policy.0, 1);
        assert_replay(sample(&mut recovered), &later_draw);
        if let Some(expected) = exploration_draw {
            let turn = recovered.observe().unwrap();
            assert_eq!(
                recovered.exploration_stream(&turn).unwrap().next_u64(),
                expected
            );
        }
        assert_replay(recovered_policy.act(&mut recovered), &later_action);
        assert_eq!(recovered.terminal_payoffs(), original.terminal_payoffs());
        assert_ne!(recovered.terminal_payoffs(), sibling.terminal_payoffs());
        assert_eq!(recovered_policy.0, 2);
    }
    assert!(differing_second_draws > 0);
}

#[test]
fn invalid_checkpoint_inputs_leave_the_target_scheduler_untouched() {
    let source = model();
    let mut world = World::new();
    let entity = world.spawn();
    let mut from = Scheduler::new();
    let mut game = NativeGame::start(&source, &world, &mut from, config(entity, 0)).unwrap();
    let mut policy = CounterPolicy::default();
    assert!(matches!(
        game.checkpoint(policy.snapshot()),
        Err(ReplayError::UnsupportedBoundary)
    ));
    policy.act(&mut game);
    let checkpoint = game
        .checkpoint(PolicySnapshot {
            format: POLICY_FORMAT.into(),
            bytes: vec![1],
        })
        .unwrap();
    let mut target = Scheduler::new();
    let initial_stats = target.stats();
    let foreign_policy = PolicyArtifactId("policy:other".into());
    assert!(matches!(
        checkpoint.resume(
            &mut target,
            &foreign_policy,
            POLICY_FORMAT,
            CounterPolicy::decode
        ),
        Err(ReplayError::PolicyBindingMismatch)
    ));
    assert!(matches!(
        checkpoint.resume(
            &mut target,
            &policy_id(),
            "wrong-format",
            CounterPolicy::decode
        ),
        Err(ReplayError::PolicyBindingMismatch)
    ));
    assert!(matches!(
        checkpoint.resume(
            &mut target,
            &policy_id(),
            POLICY_FORMAT,
            CounterPolicy::decode
        ),
        Err(ReplayError::PolicyRestoreRejected(_))
    ));
    assert_eq!(target.stats(), initial_stats);

    let mut checkpoint = game.checkpoint(policy.snapshot()).unwrap();
    checkpoint.version += 1;
    assert!(matches!(
        checkpoint.resume(
            &mut target,
            &policy_id(),
            POLICY_FORMAT,
            CounterPolicy::decode
        ),
        Err(ReplayError::UnsupportedVersion(_))
    ));
    assert_eq!(target.stats(), initial_stats);
    checkpoint.version -= 1;
    let pending = target.schedule(ScheduleRequest {
        at: SimTime::from_ticks(7),
        priority: 0,
        entity: None,
        kind: EventKind::custom(123),
    });
    let pending_stats = target.stats();
    assert!(matches!(
        checkpoint.resume(
            &mut target,
            &policy_id(),
            POLICY_FORMAT,
            CounterPolicy::decode
        ),
        Err(ReplayError::SchedulerNotFresh)
    ));
    assert_eq!(target.stats(), pending_stats);
    assert_eq!(target.step().into_dispatched().unwrap().id, pending);
}
