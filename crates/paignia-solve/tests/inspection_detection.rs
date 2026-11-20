//! Imperfect detection for the binary inspection family.

use paignia_solve::{
    inspection_detection_projection, inspection_expected_payoffs_with_detection,
    inspection_nash_with_detection, inspection_simultaneous_nash, InspectionError,
    InspectionParameters, InspectionRatio,
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
fn a_detection_interval_that_changes_regime_is_rejected() {
    let error = inspection_detection_projection(game(1, 4, 1, 2), ratio(1, 5), ratio(1, 1));
    assert_eq!(error, Err(InspectionError::DetectionRegimeChange));
}
