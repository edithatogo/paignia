//! Imperfect detection for the binary inspection family.

use paignia_solve::{
    inspection_commitment_at, inspection_commitment_bound,
    inspection_commitment_bound_with_detection, inspection_commitment_projection,
    inspection_detection_commitment_projection, inspection_detection_projection,
    inspection_detection_rectangle, inspection_expected_payoffs_with_detection,
    inspection_nash_with_detection, inspection_simultaneous_nash, InspectionError,
    InspectionFollowerTie, InspectionParameter, InspectionParameters, InspectionRatio,
    ParameterInterval,
};

fn ratio(numerator: i64, denominator: i64) -> InspectionRatio {
    InspectionRatio::new(numerator, denominator).expect("ratio")
}

fn game(cost: i64, harm: i64, gain: i64, loss: i64) -> InspectionParameters {
    InspectionParameters::try_new(
        ratio(cost, 1),
        ratio(harm, 1),
        ratio(gain, 1),
        ratio(loss, 1),
    )
    .expect("parameters")
}

#[test]
fn perfect_detection_matches_the_existing_nash_witness() {
    let interior = inspection_nash_with_detection(game(1, 4, 1, 2), ratio(1, 1)).expect("interior");
    let existing = inspection_simultaneous_nash(game(1, 4, 1, 2)).expect("existing");
    assert_eq!(interior.inspect, existing.inspect);
    assert_eq!(interior.evade, existing.evade);
    assert_eq!(interior.payoffs.leader, existing.payoffs.leader);
    assert_eq!(
        inspection_nash_with_detection(game(4, 4, 1, 2), ratio(1, 1)),
        Err(InspectionError::LeaderIndifferent)
    );
}

#[test]
fn half_detection_matches_the_independent_interior_value() {
    let detection = ratio(1, 2);
    let witness = inspection_nash_with_detection(game(1, 4, 1, 2), detection).expect("half");
    assert_eq!(witness.inspect, ratio(2, 3));
    assert_eq!(witness.evade, ratio(1, 2));
    assert_eq!(witness.payoffs.leader, ratio(-2, 1));
    assert_eq!(witness.payoffs.follower, ratio(0, 1));
    let cells = inspection_expected_payoffs_with_detection(
        game(1, 4, 1, 2),
        detection,
        witness.inspect,
        witness.evade,
    )
    .expect("cells");
    assert_eq!(cells.leader, ratio(-2, 1));
    assert_eq!(cells.follower, ratio(0, 1));
}

#[test]
fn weak_detection_inspects_for_sure_when_harm_still_justifies_it() {
    let detection = ratio(1, 4);
    let witness = inspection_nash_with_detection(game(1, 8, 1, 2), detection).expect("inspect");
    assert_eq!((witness.inspect, witness.evade), (ratio(1, 1), ratio(1, 1)));
    assert_eq!(witness.payoffs.leader, ratio(-7, 1));
    assert_eq!(witness.payoffs.follower, ratio(1, 4));
}

#[test]
fn weak_detection_skips_when_scaled_harm_is_below_cost() {
    let witness = inspection_nash_with_detection(game(1, 4, 1, 2), ratio(1, 5)).expect("skip");
    assert_eq!((witness.inspect, witness.evade), (ratio(0, 1), ratio(1, 1)));
    assert_eq!(witness.payoffs.leader, ratio(-4, 1));
    assert_eq!(witness.payoffs.follower, ratio(1, 1));
}

#[test]
fn a_detection_boundary_has_no_unique_witness() {
    let error = inspection_nash_with_detection(game(2, 4, 1, 2), ratio(1, 2));
    assert_eq!(error, Err(InspectionError::LeaderIndifferent));
    assert_eq!(
        inspection_nash_with_detection(game(1, 4, 1, 2), ratio(0, 1)),
        Err(InspectionError::DetectionOutOfRange {
            numerator: 0,
            denominator: 1
        })
    );
}

#[test]
fn a_full_inspection_follower_tie_depends_on_detection_weighted_harm() {
    let half = ratio(1, 2);
    let costly = inspection_nash_with_detection(game(3, 4, 1, 1), half).expect("unique skip");
    assert_eq!((costly.inspect, costly.evade), (ratio(0, 1), ratio(1, 1)));
    assert_eq!(
        (costly.payoffs.leader, costly.payoffs.follower),
        (ratio(-4, 1), ratio(1, 1))
    );
    assert_eq!(
        inspection_nash_with_detection(game(2, 4, 1, 1), half),
        Err(InspectionError::LeaderIndifferent)
    );

    let cheap = game(1, 4, 1, 1);
    let error = inspection_nash_with_detection(cheap, half).expect_err("follower segment");
    assert_eq!(error, InspectionError::FollowerIndifferentAtFullInspection);
    assert_eq!(
        error.to_string(),
        "follower indifference at full inspection gives multiple Nash witnesses"
    );
    assert_eq!(
        inspection_detection_projection(cheap, half, half),
        Err(error)
    );

    // At full inspection the follower gains zero. The leader's gain over skipping
    // changes sign at q = 1/2, so the Nash segment extends from q = 1/2 to q = 1.
    for (evade, advantage) in [
        (ratio(1, 4), ratio(-1, 2)),
        (half, ratio(0, 1)),
        (ratio(1, 1), ratio(1, 1)),
    ] {
        let inspect = inspection_expected_payoffs_with_detection(cheap, half, ratio(1, 1), evade)
            .expect("inspect");
        let skip = inspection_expected_payoffs_with_detection(cheap, half, ratio(0, 1), evade)
            .expect("skip");
        assert_eq!(inspect.follower, ratio(0, 1));
        assert_eq!(inspect.leader.checked_sub(skip.leader), Ok(advantage));
    }
}

#[test]
fn a_pure_skip_projection_uses_the_solved_endpoint_regimes() {
    let large = 1_i64 << 62;
    let costly = game(2, 1, large, large);
    let one = ratio(1, 1);
    for lower in [one, ratio(1, 2)] {
        let projection = inspection_detection_projection(costly, lower, one).expect("pure skip");
        for (bounds, expected) in [
            (projection.inspect, ratio(0, 1)),
            (projection.evade, one),
            (projection.leader, ratio(-1, 1)),
            (projection.follower, ratio(large, 1)),
        ] {
            assert_eq!((bounds.lower, bounds.upper), (expected, expected));
        }
        assert_eq!(projection.endpoint_gap, ratio(0, 1));
    }
    // The interior witness would need inspection share MAX / (MAX + 1).
    assert_eq!(
        inspection_detection_projection(game(1, 4, i64::MAX, 1), one, one),
        Err(InspectionError::Overflow)
    );
}

#[test]
fn an_interior_detection_interval_spans_the_endpoint_witnesses() {
    let projection =
        inspection_detection_projection(game(1, 4, 1, 2), ratio(1, 2), ratio(1, 1)).expect("span");
    assert_eq!(projection.endpoint_gap, ratio(0, 1));
    assert_eq!(projection.inspect.lower, ratio(1, 3));
    assert_eq!(projection.inspect.upper, ratio(2, 3));
    assert_eq!(projection.evade.lower, ratio(1, 4));
    assert_eq!(projection.evade.upper, ratio(1, 2));
    assert_eq!(projection.leader.lower, ratio(-2, 1));
    assert_eq!(projection.leader.upper, ratio(-1, 1));
    assert_eq!(projection.follower.lower, ratio(0, 1));
    assert_eq!(projection.follower.upper, ratio(0, 1));
}

#[test]
fn perfect_detection_commitment_matches_the_original_bound() {
    let interior = game(1, 4, 1, 2);
    let original =
        inspection_commitment_bound(interior, ratio(1, 1), InspectionFollowerTie::Pessimistic)
            .expect("original");
    let detected = inspection_commitment_bound_with_detection(
        interior,
        ratio(1, 1),
        ratio(1, 1),
        InspectionFollowerTie::Pessimistic,
    )
    .expect("detected");
    assert_eq!(original, detected);
    assert!(!original.attained);
    assert_eq!(original.value, ratio(-1, 3));
}

#[test]
fn half_detection_commitment_uses_the_two_thirds_threshold() {
    let interior = game(1, 4, 1, 2);
    let optimistic = inspection_commitment_bound_with_detection(
        interior,
        ratio(1, 2),
        ratio(1, 1),
        InspectionFollowerTie::Optimistic,
    )
    .expect("optimistic");
    assert!(optimistic.attained);
    assert_eq!(optimistic.value, ratio(-2, 3));
    assert_eq!(optimistic.inspect, Some(ratio(2, 3)));
    assert_eq!(optimistic.evade, Some(ratio(0, 1)));

    let pessimistic = inspection_commitment_bound_with_detection(
        interior,
        ratio(1, 2),
        ratio(1, 1),
        InspectionFollowerTie::Pessimistic,
    )
    .expect("pessimistic");
    assert!(!pessimistic.attained);
    assert_eq!(pessimistic.value, ratio(-2, 3));

    let capped = inspection_commitment_bound_with_detection(
        interior,
        ratio(1, 2),
        ratio(1, 2),
        InspectionFollowerTie::Optimistic,
    )
    .expect("capped");
    assert!(capped.attained);
    assert_eq!(capped.value, ratio(-7, 2));
    assert_eq!(capped.inspect, Some(ratio(1, 2)));
    assert_eq!(capped.evade, Some(ratio(1, 1)));
}

#[test]
fn a_detection_interval_that_changes_regime_is_rejected() {
    let error = inspection_detection_projection(game(1, 4, 1, 2), ratio(1, 5), ratio(1, 1));
    assert_eq!(error, Err(InspectionError::DetectionRegimeChange));
    let error = inspection_detection_projection(game(1, 8, 1, 2), ratio(1, 4), ratio(1, 2));
    assert_eq!(error, Err(InspectionError::DetectionRegimeChange));
}

fn cost_interval(lower: i64, upper: i64) -> ParameterInterval {
    ParameterInterval {
        parameter: InspectionParameter::InspectionCost,
        lower: ratio(lower, 1),
        upper: ratio(upper, 1),
    }
}

#[test]
fn detection_and_cost_span_the_interior_corners() {
    let projection = inspection_detection_rectangle(
        game(1, 4, 1, 2),
        ParameterInterval {
            parameter: InspectionParameter::InspectionCost,
            lower: ratio(1, 1),
            upper: ratio(3, 2),
        },
        ratio(1, 2),
        ratio(1, 1),
    )
    .expect("rectangle");
    assert_eq!(projection.endpoint_gap, ratio(0, 1));
    assert_eq!(projection.inspect.lower, ratio(1, 3));
    assert_eq!(projection.inspect.upper, ratio(2, 3));
    assert_eq!(projection.evade.lower, ratio(1, 4));
    assert_eq!(projection.evade.upper, ratio(3, 4));
    assert_eq!(projection.leader.lower, ratio(-3, 1));
    assert_eq!(projection.leader.upper, ratio(-1, 1));
    assert_eq!(projection.follower.lower, ratio(0, 1));
    let weak = inspection_nash_with_detection(
        InspectionParameters::try_new(ratio(3, 2), ratio(4, 1), ratio(1, 1), ratio(2, 1))
            .expect("corner parameters"),
        ratio(1, 2),
    )
    .expect("weak corner");
    assert_eq!(weak.evade, ratio(3, 4));
    assert_eq!(weak.payoffs.leader, ratio(-3, 1));
}

#[test]
fn a_detection_rectangle_that_changes_regime_is_rejected() {
    let error = inspection_detection_rectangle(
        game(1, 4, 1, 2),
        cost_interval(1, 1),
        ratio(1, 5),
        ratio(1, 1),
    );
    assert_eq!(error, Err(InspectionError::RectangleRegimeChange));
}

#[test]
fn commitment_value_spans_the_cost_endpoints_above_the_threshold() {
    let optimistic = inspection_commitment_projection(
        game(1, 4, 1, 2),
        cost_interval(1, 2),
        ratio(1, 1),
        InspectionFollowerTie::Optimistic,
    )
    .expect("optimistic");
    assert!(optimistic.attained);
    assert_eq!(optimistic.value.lower, ratio(-2, 3));
    assert_eq!(optimistic.value.upper, ratio(-1, 3));
    let inspect = optimistic.inspect.expect("attained inspection");
    assert_eq!(inspect.lower, ratio(1, 3));
    assert_eq!(inspect.upper, ratio(1, 3));
    let pessimistic = inspection_commitment_projection(
        game(1, 4, 1, 2),
        cost_interval(1, 2),
        ratio(1, 1),
        InspectionFollowerTie::Pessimistic,
    )
    .expect("pessimistic");
    assert!(!pessimistic.attained);
    assert_eq!(pessimistic.value.lower, ratio(-2, 3));
    assert_eq!(pessimistic.value.upper, ratio(-1, 3));
    assert!(pessimistic.inspect.is_none());
    assert!(pessimistic.evade.is_none());
}

#[test]
fn commitment_below_the_threshold_moves_with_cost() {
    let projection = inspection_commitment_projection(
        game(1, 4, 1, 2),
        cost_interval(1, 2),
        ratio(1, 4),
        InspectionFollowerTie::Optimistic,
    )
    .expect("below threshold");
    assert!(projection.attained);
    assert_eq!(projection.value.lower, ratio(-7, 2));
    assert_eq!(projection.value.upper, ratio(-13, 4));
    let inspect = projection.inspect.expect("budget commitment");
    assert_eq!(inspect.lower, ratio(1, 4));
    assert_eq!(inspect.upper, ratio(1, 4));
}

#[test]
fn detection_commitment_spans_the_scaled_threshold() {
    let projection = inspection_detection_commitment_projection(
        game(1, 4, 1, 2),
        ratio(1, 2),
        ratio(1, 1),
        ratio(1, 1),
        InspectionFollowerTie::Optimistic,
    )
    .expect("detection commitment");
    assert_eq!(projection.value.lower, ratio(-2, 3));
    assert_eq!(projection.value.upper, ratio(-1, 3));
    let inspect = projection.inspect.expect("threshold commitments");
    assert_eq!(inspect.lower, ratio(1, 3));
    assert_eq!(inspect.upper, ratio(2, 3));
}

#[test]
fn a_commitment_interval_that_crosses_the_threshold_is_rejected() {
    let error = inspection_commitment_projection(
        game(1, 4, 1, 2),
        ParameterInterval {
            parameter: InspectionParameter::EvasionGain,
            lower: ratio(1, 1),
            upper: ratio(5, 1),
        },
        ratio(1, 2),
        InspectionFollowerTie::Optimistic,
    );
    assert_eq!(error, Err(InspectionError::CommitmentRegimeChange));
}

#[test]
fn gain_projection_rejects_a_switch_or_tie_between_skipping_and_deterrence() {
    use InspectionFollowerTie::{Optimistic, Pessimistic};
    let full = ratio(1, 1);
    let zero = ratio(0, 1);
    for (gain, value, inspect, evade) in [
        (1, ratio(-1, 2), ratio(1, 4), zero),
        (2, ratio(-4, 5), ratio(2, 5), zero),
        (5, ratio(-1, 1), zero, full),
    ] {
        let bound = inspection_commitment_bound(game(2, 1, gain, 3), full, Optimistic)
            .expect("exact optimistic witness");
        assert!(bound.attained);
        assert_eq!(
            (bound.value, bound.inspect, bound.evade),
            (value, Some(inspect), Some(evade))
        );
    }

    let tied_game = game(2, 1, 3, 3);
    for (tie, threshold_evade, threshold_value) in [
        (Optimistic, zero, ratio(-1, 1)),
        (Pessimistic, full, ratio(-3, 2)),
    ] {
        let (evade, payoff) = inspection_commitment_at(tied_game, ratio(1, 2), full, tie)
            .expect("follower tie at the threshold");
        assert_eq!((evade, payoff.leader), (threshold_evade, threshold_value));
        let tied = inspection_commitment_bound(tied_game, full, tie).expect("tie at gain 3");
        assert!(tied.attained);
        assert_eq!(
            (tied.value, tied.inspect, tied.evade),
            (ratio(-1, 1), Some(zero), Some(full))
        );
        for (lower, upper) in [(1, 5), (1, 3), (3, 5), (3, 3)] {
            let gain = ParameterInterval {
                parameter: InspectionParameter::EvasionGain,
                lower: ratio(lower, 1),
                upper: ratio(upper, 1),
            };
            assert_eq!(
                inspection_commitment_projection(game(2, 1, 1, 3), gain, full, tie),
                Err(InspectionError::CommitmentRegimeChange),
                "gain in [{lower}, {upper}], {tie:?}"
            );
        }
    }
}

#[test]
fn detection_projection_rejects_a_change_in_optimizer_or_attainment() {
    use InspectionFollowerTie::{Optimistic, Pessimistic};
    let baseline = game(2, 1, 1, 3);
    let full = ratio(1, 1);
    let skip = Some((ratio(0, 1), full));
    for (detection, value, witness) in [
        (ratio(1, 3), ratio(-1, 1), skip),
        (ratio(1, 2), ratio(-1, 1), skip),
        (ratio(5, 8), ratio(-4, 5), None),
        (full, ratio(-1, 2), None),
    ] {
        let bound =
            inspection_commitment_bound_with_detection(baseline, detection, full, Pessimistic)
                .expect("exact pessimistic bound");
        assert_eq!(bound.value, value);
        assert_eq!(bound.attained, witness.is_some());
        assert_eq!(bound.inspect, witness.map(|(inspect, _)| inspect));
        assert_eq!(bound.evade, witness.map(|(_, evade)| evade));
    }
    let interior =
        inspection_commitment_bound_with_detection(baseline, ratio(5, 8), full, Optimistic)
            .expect("interior optimistic witness");
    assert_eq!(interior.inspect, Some(ratio(2, 5)));
    assert_eq!(interior.value, ratio(-4, 5));
    for tie in [Optimistic, Pessimistic] {
        for (lower, upper) in [
            (ratio(1, 3), full),
            (ratio(1, 3), ratio(1, 2)),
            (ratio(1, 2), full),
        ] {
            assert_eq!(
                inspection_detection_commitment_projection(baseline, lower, upper, full, tie),
                Err(InspectionError::CommitmentRegimeChange),
                "detection in [{lower:?}, {upper:?}], {tie:?}"
            );
        }
    }
}

#[test]
fn skipping_is_attained_when_deterrence_is_feasible_but_costlier() {
    for tie in [
        InspectionFollowerTie::Optimistic,
        InspectionFollowerTie::Pessimistic,
    ] {
        let projection = inspection_detection_commitment_projection(
            game(2, 1, 1, 3),
            ratio(1, 3),
            ratio(2, 5),
            ratio(1, 1),
            tie,
        )
        .expect("skipping strictly preferred throughout");
        assert!(projection.attained);
        for (bounds, expected) in [
            (projection.value, ratio(-1, 1)),
            (projection.inspect.expect("inspection"), ratio(0, 1)),
            (projection.evade.expect("evasion"), ratio(1, 1)),
        ] {
            assert_eq!((bounds.lower, bounds.upper), (expected, expected));
        }
    }
}

#[test]
fn commitment_projection_rejects_budget_equality_and_flat_evasion() {
    for (cost, budget) in [
        (cost_interval(1, 1), ratio(1, 3)),
        (cost_interval(3, 5), ratio(1, 4)),
        (cost_interval(4, 4), ratio(1, 4)),
    ] {
        assert_eq!(
            inspection_commitment_projection(
                game(1, 4, 1, 2),
                cost,
                budget,
                InspectionFollowerTie::Optimistic
            ),
            Err(InspectionError::CommitmentRegimeChange)
        );
    }
}
