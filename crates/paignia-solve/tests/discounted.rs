use paignia_core::discounted::{Error as Admission, Game, Outcome, State, StationaryPolicy};
use paignia_core::{ActionId, NodeId, PlayerId};
use paignia_solve::discounted::{evaluate, solve_zero_sum, Config, Error, Status, MAX_ITERATIONS};

const TOLERANCE: f64 = 1e-10;
const CONFIG: Config = Config {
    max_iterations: 300,
    value_tolerance: TOLERANCE,
};

fn outcome(reward: f64, stable_probability: f64) -> Outcome {
    Outcome {
        reward,
        transition: vec![stable_probability, 1.0 - stable_probability],
    }
}

fn fixture() -> (Game, StationaryPolicy) {
    (
        Game {
            players: ["commissioner", "provider"].map(|id| PlayerId(id.into())),
            actions: [["settle", "defer"], ["accept", "contend"]]
                .map(|seat| seat.map(|id| ActionId(id.into()))),
            states: vec![
                State {
                    id: NodeId("stable".into()),
                    outcomes: [
                        [outcome(2.0, 0.5), outcome(3.0, 1.0)],
                        [outcome(0.0, 0.0), outcome(1.0, 0.5)],
                    ],
                },
                State {
                    id: NodeId("pressure".into()),
                    outcomes: [
                        [outcome(-1.0, 0.25), outcome(-1.0, 1.0)],
                        [outcome(-2.0, 0.0), outcome(1.0, 0.0)],
                    ],
                },
            ],
            initial: vec![0.75, 0.25],
            discount: 0.75,
        },
        StationaryPolicy {
            weights: vec![[[1.0, 0.0]; 2]; 2],
        },
    )
}

#[test]
fn certifies_the_exact_recurring_funding_values_and_policy() {
    let (game, policy) = fixture();
    let result = evaluate(&game, &policy, CONFIG).unwrap();
    // Solve 5v - 3w = 16 and -3v + 7w = -16 by elimination.
    let exact = [32.0 / 13.0, -16.0 / 13.0];
    assert_eq!(result.status, Status::ToleranceReached);
    assert!(result.value_error_bound <= TOLERANCE);
    assert_eq!(result.state_values[0].0, NodeId("stable".into()));
    assert_eq!(result.state_values[1].0, NodeId("pressure".into()));
    for ((_, calculated), reference) in result.state_values.iter().zip(exact) {
        assert!((calculated - reference).abs() <= result.value_error_bound + f64::EPSILON);
    }
    assert!(
        (result.initial_value - 20.0 / 13.0).abs() <= result.initial_error_bound + f64::EPSILON
    );

    // Substitution of these values into the four joint moves in each state
    // gives a saddle at (settle, accept). Neither player can gain against it.
    let equilibrium = solve_zero_sum(&game, CONFIG).unwrap();
    assert_eq!(equilibrium.policy, policy);
    assert_eq!(equilibrium.game_values.status, Status::ToleranceReached);
    assert_eq!(
        equilibrium.row_best_response.status,
        Status::ToleranceReached
    );
    assert_eq!(
        equilibrium.column_best_response.status,
        Status::ToleranceReached
    );
    assert!(equilibrium
        .deviation_bounds
        .iter()
        .all(|gain| (0.0..=2.0 * TOLERANCE).contains(gain)));
}

#[test]
fn verifies_the_mixed_saddle_on_deterministic_alternating_states() {
    let (mut game, _) = fixture();
    for (index, state) in game.states.iter_mut().enumerate() {
        let successor = if index == 0 { 0.0 } else { 1.0 };
        state.outcomes = [
            [outcome(1.0, successor), outcome(-1.0, successor)],
            [outcome(-1.0, successor), outcome(1.0, successor)],
        ];
    }
    let result = solve_zero_sum(&game, CONFIG).unwrap();
    for weights in &result.policy.weights {
        assert!(weights
            .iter()
            .flatten()
            .all(|p| (*p - 0.5).abs() < f64::EPSILON));
    }
    assert!(result.game_values.initial_value.abs() <= result.game_values.initial_error_bound);
    assert!(result
        .deviation_bounds
        .iter()
        .all(|gain| (0.0..=TOLERANCE).contains(gain)));
}

#[test]
fn handles_zero_discount_absorption_and_an_exhausted_budget() {
    let (mut game, policy) = fixture();
    game.discount = 0.0;
    let immediate = evaluate(&game, &policy, CONFIG).unwrap();
    assert_eq!(immediate.status, Status::ToleranceReached);
    assert!((immediate.initial_value - 1.25).abs() <= immediate.initial_error_bound);

    game.discount = 0.75;
    game.states[1].outcomes[0][0] = outcome(-1.0, 0.0);
    let absorbing = evaluate(&game, &policy, CONFIG).unwrap();
    assert!((absorbing.state_values[1].1 + 4.0).abs() <= absorbing.value_error_bound);

    game.discount = f64::from_bits(1.0_f64.to_bits() - 1);
    let stopped = evaluate(
        &game,
        &policy,
        Config {
            max_iterations: 2,
            ..CONFIG
        },
    )
    .unwrap();
    assert_eq!(stopped.status, Status::IterationLimit);
    assert_eq!(stopped.iterations, 2);
    assert!(stopped.value_error_bound.is_finite() && stopped.value_error_bound > TOLERANCE);
}

#[test]
fn retains_a_representable_subnormal_reward_at_zero_discount() {
    let (mut game, mut policy) = fixture();
    let reward = f64::from_bits(1);
    game.states.truncate(1);
    game.states[0].outcomes = std::array::from_fn(|_| {
        std::array::from_fn(|_| Outcome {
            reward,
            transition: vec![1.0],
        })
    });
    game.initial = vec![1.0];
    game.discount = 0.0;
    policy.weights.truncate(1);
    let result = evaluate(
        &game,
        &policy,
        Config {
            max_iterations: 1,
            value_tolerance: 0.0,
        },
    )
    .unwrap();
    assert_eq!(result.status, Status::ToleranceReached);
    assert_eq!(result.state_values[0].1.to_bits(), reward.to_bits());
}

#[test]
fn encloses_nonzero_deviations_after_a_single_update() {
    let (mut game, _) = fixture();
    game.discount = 0.5;
    game.states[0].outcomes = [
        [outcome(1.0, 0.0), outcome(-1.0, 1.0)],
        [outcome(-1.0, 1.0), outcome(1.0, 1.0)],
    ];
    let sink = Outcome {
        reward: 8.0,
        transition: vec![0.0, 1.0 - 2.0_f64.powi(-41)],
    };
    game.states[1].outcomes = [[sink.clone(), sink.clone()], [sink.clone(), sink]];
    let capped = solve_zero_sum(
        &game,
        Config {
            max_iterations: 1,
            ..CONFIG
        },
    )
    .unwrap();
    assert_eq!(capped.game_values.status, Status::IterationLimit);
    assert!(capped.game_values.residual_bound >= 4.0);
    for seat in capped.policy.weights[0] {
        assert!((seat[0] - 0.25).abs() < f64::EPSILON);
    }
    for (evaluation, reference) in [
        (&capped.policy_values, 24.0 / 17.0),
        (&capped.row_best_response, 12.0 / 5.0),
        (&capped.column_best_response, 1.0),
    ] {
        assert!((evaluation.state_values[0].1 - reference).abs() <= evaluation.value_error_bound);
    }
    assert!(capped.deviation_bounds[0] >= 84.0 / 85.0);
    assert!(capped.deviation_bounds[1] >= 7.0 / 17.0);

    let settled = solve_zero_sum(&game, CONFIG).unwrap();
    assert_eq!(settled.game_values.status, Status::ToleranceReached);
    let exact = 13.0 - 137.0_f64.sqrt();
    assert!(
        (settled.game_values.state_values[0].1 - exact).abs()
            <= settled.game_values.value_error_bound + 8.0 * f64::EPSILON
    );
}

#[test]
fn rejects_bad_axes_mass_rewards_and_budgets_before_iterating() {
    let (game, policy) = fixture();
    assert!(matches!(
        evaluate(
            &game,
            &policy,
            Config {
                max_iterations: MAX_ITERATIONS + 1,
                ..CONFIG
            }
        ),
        Err(Error::Config)
    ));
    let mut invalid = game.clone();
    invalid.states[0].outcomes[0][0].transition = vec![0.5, 0.6];
    assert!(matches!(
        evaluate(&invalid, &policy, CONFIG),
        Err(Error::Input(Admission::Transition {
            state: 0,
            row: 0,
            column: 0
        }))
    ));
    invalid = game.clone();
    invalid.states[0].outcomes[0][0].reward = f64::INFINITY;
    assert!(matches!(
        evaluate(&invalid, &policy, CONFIG),
        Err(Error::Input(Admission::Reward { .. }))
    ));
    invalid = game.clone();
    invalid.states[1].id = invalid.states[0].id.clone();
    assert!(matches!(
        evaluate(&invalid, &policy, CONFIG),
        Err(Error::Input(Admission::Identity))
    ));
    invalid = game.clone();
    invalid.discount = 1.0;
    assert!(matches!(
        evaluate(&invalid, &policy, CONFIG),
        Err(Error::Input(Admission::Discount))
    ));
    let mut wrong_policy = policy.clone();
    wrong_policy.weights[1][0][1] = f64::NAN;
    assert!(matches!(
        evaluate(&game, &wrong_policy, CONFIG),
        Err(Error::Input(Admission::Policy {
            state: 1,
            player: 0
        }))
    ));
}
