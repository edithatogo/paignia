//! Imperfect detection for the binary inspection family.

use paignia_solve::{
    inspection_commitment_bound, inspection_commitment_bound_with_detection,
    inspection_detection_projection, inspection_expected_payoffs_with_detection,
    inspection_nash_with_detection, inspection_simultaneous_nash, InspectionError,
    InspectionFollowerTie, InspectionParameters, InspectionRatio,
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
