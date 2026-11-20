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
    first_exploration_draw: u64,
    exploration_draws: u128,
    detour_stream: Option<u64>,
    shared_stream: u64,
    shared_draw: usize,
}

fn run(
    source: &GameSpecification,
    seed: u64,
    policy: &str,
    budget: u64,
    unrelated_first: bool,
    comparison: ComparisonRandomness<'_>,
    fixed_action: Option<usize>,
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
    let first_exploration_draw = exploration.next_u64();
    let score = (1..budget).fold(first_exploration_draw, |best, _| {
        best.max(exploration.next_u64())
    });
    let chosen = fixed_action.unwrap_or(usize::from(score & 1 != 0));
    let action = &turn.observation().legal_actions[chosen];
    let at = run.next_transition_time().unwrap();
    run.submit(Command::Act {
        turn: &turn,
        action,
        at,
    })
    .unwrap();

    let mut detour_stream = None;
    let mut shared_draw = None;
    while run.phase() == Phase::Chance {
        let at = run.next_transition_time().unwrap();
        let event = run.realize_chance(at).unwrap();
        let stream = event.chance_stream_id.unwrap();
        if event.from == node("shared lottery") {
            let TransitionOutcome::Chance { edge_index } = event.outcome else {
                panic!("expected a native chance event");
            };
            shared_draw = Some((edge_index, stream));
        } else {
            assert_eq!(event.from, node("extra draw"));
            assert!(detour_stream.replace(stream).is_none());
        }
    }
    assert!(run.terminal_payoffs().is_some());
    let (shared_draw, shared_stream) = shared_draw.unwrap();
    assert_ne!(detour_stream, Some(shared_stream));
    Outcome {
        native_entity_index: entity.index,
        first_exploration_draw,
        exploration_draws: exploration.draw_count(),
        detour_stream,
        shared_stream,
        shared_draw,
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
        let first = run(&source, seed, "policy A", 1, false, comparison, None);
        let varied_budget = run(&source, seed, "policy A", 9, true, comparison, None);
        let other_policy = run(&source, seed, "policy B", 9, true, comparison, None);
        assert_ne!(first.native_entity_index, varied_budget.native_entity_index);
        assert_eq!(
            first.first_exploration_draw,
            varied_budget.first_exploration_draw
        );
        assert_ne!(
            first.first_exploration_draw,
            other_policy.first_exploration_draw
        );
        assert_eq!(first.exploration_draws, 1);
        assert_eq!(varied_budget.exploration_draws, 9);
        assert_eq!(first.shared_draw, varied_budget.shared_draw);
        assert_eq!(first.shared_draw, other_policy.shared_draw);
        assert_eq!(first.shared_stream, varied_budget.shared_stream);
        assert_eq!(first.shared_stream, other_policy.shared_stream);
        changed_paths +=
            usize::from(first.detour_stream.is_some() != varied_budget.detour_stream.is_some());
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
        let baseline = run(&source, seed, "policy A", 7, false, comparison, Some(0));
        let independent = run(&source, seed, "policy B", 7, false, comparison, Some(0));
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
            Some(0),
        );
        assert_eq!(baseline.detour_stream, None);
        assert_eq!(independent.detour_stream, None);
        assert_eq!(other_draw.detour_stream, None);
        assert_ne!(baseline.shared_stream, independent.shared_stream);
        assert_ne!(baseline.shared_stream, other_draw.shared_stream);
        policy_differences += usize::from(baseline.shared_draw != independent.shared_draw);
        draw_differences += usize::from(baseline.shared_draw != other_draw.shared_draw);
    }
    assert!(policy_differences > 0);
    assert!(draw_differences > 0);
}
