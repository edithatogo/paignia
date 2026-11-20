//! Exact 2x2 inspection commitment, checked against algebraic certificates.

use paignia_solve::{
    inspection_commitment_at, inspection_commitment_bound, inspection_deviation_gains,
    inspection_expected_payoffs, inspection_simultaneous_nash, InspectionError,
    InspectionFollowerTie, InspectionParameters, InspectionRatio,
};

fn ratio(numerator: i64, denominator: i64) -> InspectionRatio {
    InspectionRatio::new(numerator, denominator).expect("finite ratio")
}

fn game(cost: i64, harm: i64, gain: i64, loss: i64) -> InspectionParameters {
    InspectionParameters::try_new(
        ratio(cost, 1),
        ratio(harm, 1),
        ratio(gain, 1),
        ratio(loss, 1),
    )
    .expect("positive parameters")
}

/// `U_I = -c p - h (1-p) q` and `U_E = q (g - (g+f) p)`, without the cell sum.
fn closed_form(
    cost: i64,
    harm: i64,
    gain: i64,
    loss: i64,
    inspect: InspectionRatio,
    evade: InspectionRatio,
) -> (InspectionRatio, InspectionRatio) {
    let one = ratio(1, 1);
    let cost = ratio(cost, 1);
    let harm = ratio(harm, 1);
    let gain = ratio(gain, 1);
    let loss = ratio(loss, 1);
    let leader = ratio(0, 1)
        .checked_sub(cost.checked_mul(inspect).expect("cost term"))
        .expect("leader cost")
        .checked_sub(
            harm.checked_mul(one.checked_sub(inspect).expect("skip"))
                .expect("harm weight")
                .checked_mul(evade)
                .expect("harm term"),
        )
        .expect("leader payoff");
    let follower = evade
        .checked_mul(
            gain.checked_sub(
                gain.checked_add(loss)
                    .expect("loss weight")
                    .checked_mul(inspect)
                    .expect("threshold term"),
            )
            .expect("follower advantage"),
        )
        .expect("follower payoff");
    (leader, follower)
}

#[test]
fn rejects_parameters_and_commitments_outside_the_family() {
    let error = InspectionParameters::try_new(ratio(0, 1), ratio(1, 1), ratio(1, 1), ratio(1, 1));
    assert_eq!(error, Err(InspectionError::NonPositiveParameter));
    assert_eq!(
        InspectionRatio::new(1, 0),
        Err(InspectionError::ZeroDenominator)
    );

    let interior = game(1, 4, 1, 2);
    let error = inspection_commitment_at(
        interior,
        ratio(1, 2),
        ratio(1, 4),
        InspectionFollowerTie::Optimistic,
    );
    assert_eq!(error, Err(InspectionError::BudgetExceeded));
    let error =
        inspection_commitment_bound(interior, ratio(3, 2), InspectionFollowerTie::Optimistic);
    assert!(matches!(
        error,
        Err(InspectionError::ProbabilityOutOfRange { .. })
    ));
    assert_eq!(
        inspection_simultaneous_nash(game(4, 4, 1, 2)),
        Err(InspectionError::LeaderIndifferent)
    );
}

#[test]
fn subtraction_accepts_a_minimum_numerator_when_the_difference_fits() {
    let minimum = ratio(i64::MIN, 1);
    assert_eq!(minimum.checked_sub(minimum), Ok(ratio(0, 1)));
    assert_eq!(ratio(-1, 1).checked_sub(minimum), Ok(ratio(i64::MAX, 1)));
    assert_eq!(
        ratio(0, 1).checked_sub(minimum),
        Err(InspectionError::Overflow)
    );
}

#[test]
fn deviation_gain_is_exact_for_a_payoff_with_a_minimum_numerator() {
    let costly = game(2, 1, 1, 1);
    let inspect = ratio(1, i64::MAX);
    let evade = ratio(1, 1);
    let payoffs = inspection_expected_payoffs(costly, inspect, evade).expect("payoffs");
    assert_eq!(payoffs.leader, ratio(i64::MIN, i64::MAX));
    let gains = inspection_deviation_gains(costly, inspect, evade).expect("representable gains");
    assert_eq!((gains.leader, gains.follower), (inspect, ratio(0, 1)));
}

#[test]
fn nash_witnesses_match_the_closed_form_and_have_zero_gains() {
    let fixture = include_str!("../../../verification/inspection_oracles/inspection_2x2_v1.json");
    assert!(fixture.contains("\"p\": \"1/3\", \"q\": \"1/4\", \"payoffs\": [\"-1\", \"0\"]"));
    assert!(fixture.contains("\"p\": \"0\", \"q\": \"1\", \"payoffs\": [\"-4\", \"1\"]"));

    let interior = inspection_simultaneous_nash(game(1, 4, 1, 2)).expect("interior Nash");
    assert_eq!(
        (interior.inspect, interior.evade),
        (ratio(1, 3), ratio(1, 4))
    );
    assert_eq!(
        closed_form(1, 4, 1, 2, interior.inspect, interior.evade),
        (ratio(-1, 1), ratio(0, 1))
    );
    assert_eq!(
        (interior.payoffs.leader, interior.payoffs.follower),
        (ratio(-1, 1), ratio(0, 1))
    );
    let gains = inspection_deviation_gains(game(1, 4, 1, 2), ratio(1, 3), ratio(1, 4))
        .expect("interior gains");
    assert_eq!((gains.leader, gains.follower), (ratio(0, 1), ratio(0, 1)));

    let costly = inspection_simultaneous_nash(game(5, 4, 1, 2)).expect("costly Nash");
    assert_eq!((costly.inspect, costly.evade), (ratio(0, 1), ratio(1, 1)));
    assert_eq!(costly.payoffs.leader, ratio(-4, 1));
    assert_ne!(costly.evade, ratio(5, 4));
    let false_commitment =
        inspection_deviation_gains(game(1, 4, 1, 2), ratio(1, 3), ratio(0, 1)).expect("false Nash");
    assert_eq!(
        (false_commitment.leader, false_commitment.follower),
        (ratio(1, 3), ratio(0, 1))
    );
}

#[test]
fn pure_profiles_match_an_independent_closed_form() {
    let interior = game(1, 4, 1, 2);
    for (inspect, evade, leader, follower) in
        [(1, 0, -1, 0), (1, 1, -1, -2), (0, 0, 0, 0), (0, 1, -4, 1)]
    {
        let inspect = ratio(inspect, 1);
        let evade = ratio(evade, 1);
        let payoffs = inspection_expected_payoffs(interior, inspect, evade).expect("pure profile");
        assert_eq!(
            (payoffs.leader, payoffs.follower),
            (ratio(leader, 1), ratio(follower, 1))
        );
        assert_eq!(
            closed_form(1, 4, 1, 2, inspect, evade),
            (payoffs.leader, payoffs.follower)
        );
    }
}

#[test]
fn commitment_bounds_match_the_fourteen_literal_certificates() {
    let fixture = include_str!("../../../verification/inspection_oracles/inspection_2x2_v1.json");
    assert!(fixture.contains("\"value\": \"-1/3\", \"attained\": false"));
    assert!(fixture.contains("\"value\": \"-5/3\", \"attained\": false"));
    assert_eq!(CERTIFICATES.len(), 14);
    for row in CERTIFICATES {
        check_certificate(row);
    }
}

#[test]
fn a_sixteenth_grid_misses_the_exact_optimistic_commitment() {
    let interior = game(1, 4, 1, 2);
    let optimum =
        inspection_commitment_bound(interior, ratio(1, 1), InspectionFollowerTie::Optimistic)
            .expect("optimum");
    assert_eq!(optimum.value, ratio(-1, 3));
    for step in 0..=16 {
        let (_, payoffs) = inspection_commitment_at(
            interior,
            ratio(step, 16),
            ratio(1, 1),
            InspectionFollowerTie::Optimistic,
        )
        .expect("grid point");
        assert!(payoffs.leader < optimum.value);
    }
}

#[test]
fn the_pessimistic_supremum_is_approached_at_two_thirds_and_not_attained() {
    let interior = game(1, 4, 1, 2);
    let bound =
        inspection_commitment_bound(interior, ratio(1, 1), InspectionFollowerTie::Pessimistic)
            .expect("supremum");
    assert!(!bound.attained);
    assert_eq!(bound.value, ratio(-1, 3));
    let (evade, payoffs) = inspection_commitment_at(
        interior,
        ratio(2, 3),
        ratio(1, 1),
        InspectionFollowerTie::Pessimistic,
    )
    .expect("approach");
    assert_eq!(evade, ratio(0, 1));
    assert_eq!(payoffs.leader, ratio(-2, 3));
    assert_eq!(
        closed_form(1, 4, 1, 2, ratio(2, 3), ratio(0, 1)),
        (payoffs.leader, payoffs.follower)
    );
}

fn check_certificate(row: Certificate) {
    let (cost, harm, budget, optimistic, value, attained, witness) = row;
    let tie = if optimistic {
        InspectionFollowerTie::Optimistic
    } else {
        InspectionFollowerTie::Pessimistic
    };
    let bound = inspection_commitment_bound(game(cost, harm, 1, 2), ratio(budget.0, budget.1), tie)
        .expect("bound");
    assert_eq!(bound.value, ratio(value.0, value.1), "{cost} {harm}");
    assert_eq!(bound.attained, attained);
    let witness = witness.map(|((p_n, p_d), (q_n, q_d))| (ratio(p_n, p_d), ratio(q_n, q_d)));
    assert_eq!(bound.inspect.zip(bound.evade), witness);
}

type Pair = (i64, i64);
type Certificate = (i64, i64, Pair, bool, Pair, bool, Option<(Pair, Pair)>);

const CERTIFICATES: [Certificate; 14] = [
    (1, 4, (1, 1), true, (-1, 3), true, Some(((1, 3), (0, 1)))),
    (1, 4, (1, 1), false, (-1, 3), false, None),
    (1, 4, (1, 4), true, (-13, 4), true, Some(((1, 4), (1, 1)))),
    (1, 4, (1, 4), false, (-13, 4), true, Some(((1, 4), (1, 1)))),
    (1, 4, (1, 3), true, (-1, 3), true, Some(((1, 3), (0, 1)))),
    (1, 4, (1, 3), false, (-3, 1), true, Some(((1, 3), (1, 1)))),
    (1, 4, (0, 1), true, (-4, 1), true, Some(((0, 1), (1, 1)))),
    (1, 4, (0, 1), false, (-4, 1), true, Some(((0, 1), (1, 1)))),
    (5, 4, (1, 1), true, (-5, 3), true, Some(((1, 3), (0, 1)))),
    (5, 4, (1, 1), false, (-5, 3), false, None),
    (5, 4, (1, 4), true, (-4, 1), true, Some(((0, 1), (1, 1)))),
    (5, 4, (1, 4), false, (-4, 1), true, Some(((0, 1), (1, 1)))),
    (5, 4, (1, 3), true, (-5, 3), true, Some(((1, 3), (0, 1)))),
    (5, 4, (1, 3), false, (-4, 1), true, Some(((0, 1), (1, 1)))),
];
