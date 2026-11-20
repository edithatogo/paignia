use std::{collections::BTreeSet, num::NonZeroU64};

use kairo_ecs_core::Scheduler;
use kairo_ecs_state::World;
use paignia_core::{
    ActionEdge, ActionId, ChanceEdge, Game, GameInstanceId, GameSessionId, GameSpecification, Node,
    NodeId, PlayerId, PolicyArtifactId, GAME_SPEC_SCHEMA_VERSION, STATIC_GAME_PROFILE_V1,
};
use paignia_kairos::{
    ChanceCoupling, Command, ComparisonRandomness, ExecutionConfig, NativeGame, Phase,
    TransitionOutcome, EXECUTION_SCHEMA_VERSION,
};

fn node(name: &str) -> NodeId {
    NodeId(name.into())
}

fn model() -> GameSpecification {
    let lottery = (0..8).map(|index| {
        let terminal = node(&format!("outcome-{index}"));
        let payoff = Node::Terminal {
            payoffs: [(PlayerId("controller".into()), f64::from(index))].into(),
        };
        (
            ChanceEdge {
                probability: 0.125,
                target: terminal.clone(),
            },
            (terminal, payoff),
        )
    });
    let (edges, mut nodes): (Vec<_>, std::collections::BTreeMap<_, _>) = lottery.unzip();
    nodes.insert(node("shared lottery"), Node::Chance { edges });
    nodes.insert(
        node("extra draw"),
        Node::Chance {
            edges: vec![ChanceEdge {
                probability: 1.0,
                target: node("shared lottery"),
            }],
        },
    );
    nodes.insert(
        node("choose"),
        Node::Decision {
            player: PlayerId("controller".into()),
            edges: vec![
                ActionEdge {
                    action: ActionId("direct".into()),
                    target: node("shared lottery"),
                },
                ActionEdge {
                    action: ActionId("detour".into()),
                    target: node("extra draw"),
                },
            ],
        },
    );
    GameSpecification {
        schema_version: GAME_SPEC_SCHEMA_VERSION.into(),
        id: "game:paired lottery".into(),
        game: Game {
            root: node("choose"),
            nodes,
        },
    }
}

struct Outcome {
    native_entity_index: u64,
    exploration_stream: u64,
    exploration_draws: u128,
    chance_events: usize,
    shared_draw: usize,
}

fn run(
    source: &GameSpecification,
    seed: u64,
    policy: &str,
    budget: u64,
    unrelated_first: bool,
    comparison: ComparisonRandomness<'_>,
) -> Outcome {
    let mut world = World::new();
    if unrelated_first {
        world.spawn();
    }
    let entity = world.spawn();
    let mut scheduler = Scheduler::new();
    let mut run = NativeGame::start_comparison(
        source,
        &world,
        &mut scheduler,
        ExecutionConfig {
            schema_version: EXECUTION_SCHEMA_VERSION,
            semantic_profile: STATIC_GAME_PROFILE_V1,
            game_instance_id: GameInstanceId("logical consumer".into()),
            session_id: GameSessionId(format!("{policy}:{budget}:{unrelated_first}")),
            policy_artifact_id: PolicyArtifactId(policy.into()),
            entity,
            seed,
            transition_limit: NonZeroU64::new(3).unwrap(),
        },
        comparison,
    )
    .unwrap();

    let turn = run.observe().unwrap();
    let mut exploration = run.exploration_stream(&turn).unwrap();
    let score = (0..budget).map(|_| exploration.next_u64()).max().unwrap();
    let action = &turn.observation().legal_actions[usize::from(score & 1 != 0)];
    let at = run.next_transition_time().unwrap();
    run.submit(Command::Act {
        turn: &turn,
        action,
        at,
    })
    .unwrap();

    let mut chance_events = 0;
    let mut shared_draw = None;
    while run.phase() == Phase::Chance {
        let at = run.next_transition_time().unwrap();
        let event = run.realize_chance(at).unwrap();
        chance_events += 1;
        if event.from == node("shared lottery") {
            let TransitionOutcome::Chance { edge_index } = event.outcome else {
                panic!("expected a native chance event");
            };
            shared_draw = Some(edge_index);
        }
    }
    assert!(run.terminal_payoffs().is_some());
    Outcome {
        native_entity_index: entity.index,
        exploration_stream: exploration.stream_id(),
        exploration_draws: exploration.draw_count(),
        chance_events,
        shared_draw: shared_draw.unwrap(),
    }
}

#[test]
fn exploration_budget_policy_path_and_entity_order_preserve_the_paired_chance_draw() {
    let source = model();
    let comparison = ComparisonRandomness {
        scenario_id: "scenario:paired",
        paired_draw_id: "draw:42",
        chance_coupling: ChanceCoupling::SharedAcrossPolicies,
    };
    let mut changed_paths = 0;
    let mut sampled = BTreeSet::new();
    for seed in 0..64 {
        let first = run(&source, seed, "policy A", 1, false, comparison);
        let varied_budget = run(&source, seed, "policy A", 9, true, comparison);
        let other_policy = run(&source, seed, "policy B", 9, true, comparison);
        assert_ne!(first.native_entity_index, varied_budget.native_entity_index);
        assert_eq!(first.exploration_stream, varied_budget.exploration_stream);
        assert_ne!(first.exploration_stream, other_policy.exploration_stream);
        assert_eq!(first.exploration_draws, 1);
        assert_eq!(varied_budget.exploration_draws, 9);
        assert_eq!(first.shared_draw, varied_budget.shared_draw);
        assert_eq!(first.shared_draw, other_policy.shared_draw);
        changed_paths += usize::from(first.chance_events != varied_budget.chance_events);
        sampled.insert(first.shared_draw);
    }
    assert!(changed_paths > 0);
    assert!(sampled.len() > 1);
}

#[test]
fn independent_policies_and_distinct_draws_produce_distinct_native_sequences() {
    let source = model();
    let comparison = ComparisonRandomness {
        scenario_id: "scenario:paired",
        paired_draw_id: "draw:42",
        chance_coupling: ChanceCoupling::IndependentByPolicy,
    };
    let mut policy_differences = 0;
    let mut draw_differences = 0;
    for seed in 0..32 {
        let baseline = run(&source, seed, "policy A", 7, false, comparison);
        let independent = run(&source, seed, "policy B", 7, false, comparison);
        let other_draw = run(
            &source,
            seed,
            "policy A",
            7,
            false,
            ComparisonRandomness {
                paired_draw_id: "draw:43",
                ..comparison
            },
        );
        policy_differences += usize::from(baseline.shared_draw != independent.shared_draw);
        draw_differences += usize::from(baseline.shared_draw != other_draw.shared_draw);
    }
    assert!(policy_differences > 0);
    assert!(draw_differences > 0);
}
