use std::num::NonZeroU64;

use kairo_ecs_core::Scheduler;
use kairo_ecs_state::World;
use kairo_ecs_types::{EntityId, EventKind, ScheduleRequest, SimTime, StepOutcome};
use paignia_core::{
    ActionEdge, ActionId, ChanceEdge, Game, GameInstanceId, GameSessionId, GameSpecification, Node,
    NodeId, ObservationRevision, PlayerId, PolicyArtifactId, GAME_SPEC_SCHEMA_VERSION,
    INFORMATION_SET_PROFILE_V1, STATIC_GAME_PROFILE_V1,
};
use paignia_kairos::{
    CausalTransition, Command, ExecutionConfig, ExecutionError, NativeGame, TransitionOutcome,
    CHANCE_EVENT, DECISION_EVENT, EXECUTION_SCHEMA_VERSION,
};

fn node(id: &str) -> NodeId {
    NodeId(id.into())
}

fn player() -> PlayerId {
    PlayerId("row".into())
}

fn start_model() -> GameSpecification {
    let terminal = |value| Node::Terminal {
        payoffs: [(player(), value)].into(),
    };
    GameSpecification {
        schema_version: GAME_SPEC_SCHEMA_VERSION.into(),
        id: "game:finite".into(),
        game: Game {
            root: node("choose"),
            nodes: [
                (
                    node("choose"),
                    Node::Decision {
                        player: player(),
                        edges: vec![
                            ActionEdge {
                                action: ActionId("pass".into()),
                                target: node("unselected"),
                            },
                            ActionEdge {
                                action: ActionId("play".into()),
                                target: node("draw"),
                            },
                        ],
                    },
                ),
                (
                    node("draw"),
                    Node::Chance {
                        edges: vec![
                            ChanceEdge {
                                probability: 0.0,
                                target: node("impossible"),
                            },
                            ChanceEdge {
                                probability: 0.25,
                                target: node("low"),
                            },
                            ChanceEdge {
                                probability: 0.75,
                                target: node("high"),
                            },
                        ],
                    },
                ),
                (node("unselected"), terminal(100.0)),
                (node("impossible"), terminal(-100.0)),
                (node("low"), terminal(2.0)),
                (node("high"), terminal(6.0)),
            ]
            .into(),
        },
    }
}

fn config(entity: EntityId, seed: u64) -> ExecutionConfig<'static> {
    ExecutionConfig {
        schema_version: EXECUTION_SCHEMA_VERSION,
        semantic_profile: STATIC_GAME_PROFILE_V1,
        game_instance_id: GameInstanceId("instance:run".into()),
        session_id: GameSessionId("session:run".into()),
        policy_artifact_id: PolicyArtifactId("policy:run".into()),
        entity,
        seed,
        transition_limit: NonZeroU64::new(2).unwrap(),
    }
}

fn execute(seed: u64, reject_first: bool) -> [CausalTransition; 2] {
    let source = start_model();
    let mut world = World::new();
    let retired = world.spawn();
    assert!(world.despawn(retired));
    let entity = world.spawn();
    assert_ne!(entity.generation, retired.generation);
    let mut scheduler = Scheduler::new();
    let earlier = scheduler.schedule(ScheduleRequest {
        at: SimTime::from_ticks(20),
        priority: 7,
        entity: None,
        kind: EventKind::custom(900),
    });
    assert_eq!(scheduler.step().dispatched_event().unwrap().id, earlier);
    let stats = scheduler.stats();

    let (first, second) = {
        let mut game =
            NativeGame::start(&source, &world, &mut scheduler, config(entity, seed)).unwrap();
        let turn = game.observe().unwrap();
        assert_eq!(turn.observation().binding.player_id, player());
        assert_eq!(turn.observation().binding.revision, ObservationRevision(0));
        assert_eq!(
            turn.observation().legal_actions,
            [ActionId("pass".into()), ActionId("play".into())]
        );
        let play = ActionId("play".into());
        let first = game
            .submit(Command::Act {
                turn: &turn,
                action: &play,
                at: SimTime::from_ticks(21),
            })
            .unwrap();
        let at = game.next_transition_time().unwrap();
        if reject_first {
            assert!(matches!(
                game.realize_chance(SimTime::from_ticks(23)),
                Err(ExecutionError::UnsupportedTime { .. })
            ));
        }
        let second = game.realize_chance(at).unwrap();
        let expected = match second.outcome {
            TransitionOutcome::Chance { edge_index: 1 } => (node("low"), 2.0),
            TransitionOutcome::Chance { edge_index: 2 } => (node("high"), 6.0),
            ref outcome => panic!("unreachable or unchosen branch executed: {outcome:?}"),
        };
        assert_eq!(second.to, expected.0);
        assert_eq!(
            game.terminal_payoffs().unwrap(),
            &[(player(), expected.1)].into()
        );
        assert!(matches!(
            game.realize_chance(SimTime::from_ticks(23)),
            Err(ExecutionError::NotAtChance)
        ));
        (first, second)
    };

    assert_eq!(
        first.outcome,
        TransitionOutcome::Action(ActionId("play".into()))
    );
    assert_eq!(first.from, node("choose"));
    assert_eq!(first.to, node("draw"));
    assert_eq!(second.from, first.to);
    assert_eq!(first.before, ObservationRevision(0));
    assert_eq!(first.after, ObservationRevision(1));
    assert_eq!(second.before, first.after);
    assert_eq!(second.after, ObservationRevision(2));
    assert_eq!(second.identity, first.identity);
    assert_eq!(first.identity.specification_id.0, source.id);
    assert_eq!(first.identity.policy_artifact_id.0, "policy:run");
    assert_eq!(first.event.kind, DECISION_EVENT);
    assert_eq!(second.event.kind, CHANCE_EVENT);
    assert_eq!(first.event.entity, Some(entity));
    assert_eq!(second.event.entity, Some(entity));
    assert_eq!(first.event.at, SimTime::from_ticks(21));
    assert_eq!(second.event.at, SimTime::from_ticks(22));
    assert_eq!(first.event.id.index, earlier.index + 1);
    assert_eq!(second.event.id.index, first.event.id.index + 1);
    assert_eq!(first.event.id.generation, earlier.generation + 1);
    assert_eq!(second.event.sequence, first.event.sequence + 1);
    assert_eq!(
        scheduler.stats().scheduled_events,
        stats.scheduled_events + 2
    );
    assert_eq!(
        scheduler.stats().dispatched_events,
        stats.dispatched_events + 2
    );
    assert_eq!(scheduler.stats().cancelled_events, stats.cancelled_events);
    assert_eq!(scheduler.pending_events(), 0);
    assert_eq!(scheduler.now(), second.event.at);
    assert_eq!(scheduler.step(), StepOutcome::Empty);
    [first, second]
}

#[test]
fn selected_action_and_chance_are_the_only_native_dispatches() {
    let mut counts = [0; 3];
    for seed in 0..64 {
        let expected = execute(seed, false);
        assert_eq!(execute(seed, true), expected);
        if let TransitionOutcome::Chance { edge_index } = expected[1].outcome {
            counts[edge_index] += 1;
        }
    }
    assert!(counts[1] > 0 && counts[2] > 0);
}

#[test]
fn illegal_stale_delayed_and_cancelled_commands_do_not_dispatch() {
    let source = start_model();
    let mut world = World::new();
    let entity = world.spawn();
    let mut scheduler = Scheduler::new();
    let stats = scheduler.stats();
    {
        let mut game =
            NativeGame::start(&source, &world, &mut scheduler, config(entity, 0)).unwrap();
        let turn = game.observe().unwrap();
        let play = ActionId("play".into());
        let at = game.next_transition_time().unwrap();
        let unknown = ActionId("forged".into());
        assert!(matches!(
            game.submit(Command::Act {
                turn: &turn,
                action: &unknown,
                at
            }),
            Err(ExecutionError::UnknownAction(_))
        ));
        for wrong in [SimTime::ZERO, SimTime::from_ticks(2)] {
            assert!(matches!(
                game.submit(Command::Act {
                    turn: &turn,
                    action: &play,
                    at: wrong
                }),
                Err(ExecutionError::UnsupportedTime { .. })
            ));
        }
        assert!(matches!(
            game.submit(Command::Cancel),
            Err(ExecutionError::UnsupportedCancellation)
        ));
        assert_eq!(game.observe().unwrap().observation(), turn.observation());
        game.submit(Command::Act {
            turn: &turn,
            action: &play,
            at,
        })
        .unwrap();
        assert!(matches!(
            game.submit(Command::Act {
                turn: &turn,
                action: &play,
                at: SimTime::from_ticks(2)
            }),
            Err(ExecutionError::StaleDecision)
        ));
    }
    assert_eq!(
        scheduler.stats().scheduled_events,
        stats.scheduled_events + 1
    );
    assert_eq!(
        scheduler.stats().dispatched_events,
        stats.dispatched_events + 1
    );
    assert_eq!(scheduler.stats().cancelled_events, stats.cancelled_events);
    assert_eq!(scheduler.pending_events(), 0);
}

#[test]
fn a_new_session_rejects_a_turn_from_an_earlier_binding_with_equal_labels() {
    let source = start_model();
    let mut world = World::new();
    let entity = world.spawn();
    let mut scheduler = Scheduler::new();
    let previous = NativeGame::start(&source, &world, &mut scheduler, config(entity, 0))
        .unwrap()
        .observe()
        .unwrap();
    let mut game = NativeGame::start(&source, &world, &mut scheduler, config(entity, 0)).unwrap();
    let current = game.observe().unwrap();
    assert_eq!(current.observation(), previous.observation());
    let at = game.next_transition_time().unwrap();
    assert!(matches!(
        game.submit(Command::Act {
            turn: &previous,
            action: &ActionId("play".into()),
            at
        }),
        Err(ExecutionError::StaleDecision)
    ));
    assert_eq!(game.observe().unwrap().observation(), current.observation());
}

#[test]
fn admission_preserves_scheduler_on_stale_entity_or_pending_event() {
    let source = start_model();
    let mut world = World::new();
    let retired = world.spawn();
    assert!(world.despawn(retired));
    let entity = world.spawn();
    assert_eq!(entity.index, retired.index);
    let mut scheduler = Scheduler::new();
    let stats = scheduler.stats();
    assert!(matches!(
        NativeGame::start(&source, &world, &mut scheduler, config(retired, 0)),
        Err(ExecutionError::InactiveEntity(id)) if id == retired
    ));
    assert_eq!(scheduler.stats(), stats);
    let unrelated = scheduler.schedule(ScheduleRequest {
        at: SimTime::from_ticks(4),
        priority: -9,
        entity: Some(entity),
        kind: EventKind::custom(777),
    });
    let stats = scheduler.stats();
    assert!(matches!(
        NativeGame::start(&source, &world, &mut scheduler, config(entity, 0)),
        Err(ExecutionError::SchedulerBusy)
    ));
    assert_eq!(scheduler.stats(), stats);
    assert_eq!(scheduler.step().dispatched_event().unwrap().id, unrelated);
}

#[test]
fn versions_and_budget_are_checked_before_native_mutation() {
    let source = start_model();
    let mut world = World::new();
    let entity = world.spawn();
    let mut scheduler = Scheduler::new();
    let stats = scheduler.stats();
    assert!(matches!(
        NativeGame::start(
            &source,
            &world,
            &mut scheduler,
            ExecutionConfig {
                schema_version: "2.0.0",
                ..config(entity, 0)
            }
        ),
        Err(ExecutionError::UnsupportedSchemaVersion(_))
    ));
    for profile in [INFORMATION_SET_PROFILE_V1, "simultaneous"] {
        assert!(matches!(
            NativeGame::start(
                &source,
                &world,
                &mut scheduler,
                ExecutionConfig {
                    semantic_profile: profile,
                    ..config(entity, 0)
                }
            ),
            Err(ExecutionError::InvalidSpecification(_))
        ));
    }
    assert_eq!(scheduler.stats(), stats);
    {
        let mut game = NativeGame::start(
            &source,
            &world,
            &mut scheduler,
            ExecutionConfig {
                transition_limit: NonZeroU64::new(1).unwrap(),
                ..config(entity, 0)
            },
        )
        .unwrap();
        let turn = game.observe().unwrap();
        let at = game.next_transition_time().unwrap();
        game.submit(Command::Act {
            turn: &turn,
            action: &ActionId("play".into()),
            at,
        })
        .unwrap();
        assert!(matches!(
            game.realize_chance(SimTime::from_ticks(2)),
            Err(ExecutionError::TransitionLimit)
        ));
        assert!(game.terminal_payoffs().is_none());
    }
    assert_eq!(scheduler.stats().scheduled_events, 1);
    assert_eq!(scheduler.stats().dispatched_events, 1);
    assert_eq!(scheduler.now(), SimTime::from_ticks(1));
}
