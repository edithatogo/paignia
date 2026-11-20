//! Exact Nash projections for one moving inspection parameter.

use paignia_solve::{
    inspection_deviation_gains, inspection_nash_box, inspection_nash_hull,
    inspection_nash_projection, inspection_nash_rectangle, inspection_simultaneous_nash,
    InspectionError, InspectionParameter, InspectionParameters, InspectionRatio, ParameterInterval,
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

fn interval(
    parameter: InspectionParameter,
    lower: (i64, i64),
    upper: (i64, i64),
) -> ParameterInterval {
    ParameterInterval {
        parameter,
        lower: ratio(lower.0, lower.1),
        upper: ratio(upper.0, upper.1),
    }
}

#[test]
fn rejects_an_unordered_or_nonpositive_interval() {
    let baseline = game(1, 4, 1, 2);
    let reversed = interval(InspectionParameter::DetectedLoss, (2, 1), (1, 1));
    assert_eq!(
        inspection_nash_projection(baseline, reversed),
        Err(InspectionError::UnorderedInterval)
    );
    let nonpositive = interval(InspectionParameter::DetectedLoss, (0, 1), (2, 1));
    assert_eq!(
        inspection_nash_projection(baseline, nonpositive),
        Err(InspectionError::NonPositiveParameter)
    );
}

#[test]
fn detected_loss_moves_only_the_inspection_share() {
    let baseline = game(1, 4, 1, 2);
    let projection = inspection_nash_projection(
        baseline,
        interval(InspectionParameter::DetectedLoss, (2, 1), (8, 1)),
    )
    .expect("projection");
    assert_eq!(projection.endpoint_gap, ratio(0, 1));
    assert_eq!(projection.inspect.lower, ratio(1, 9));
    assert_eq!(projection.inspect.upper, ratio(1, 3));
    assert_eq!(
        (projection.evade.lower, projection.evade.upper),
        (ratio(1, 4), ratio(1, 4))
    );
    assert_eq!(
        (projection.leader.lower, projection.leader.upper),
        (ratio(-1, 1), ratio(-1, 1))
    );
    assert_eq!(
        (projection.follower.lower, projection.follower.upper),
        (ratio(0, 1), ratio(0, 1))
    );
    let low = inspection_simultaneous_nash(game(1, 4, 1, 8)).expect("high loss");
    let high = inspection_simultaneous_nash(game(1, 4, 1, 2)).expect("low loss");
    assert_eq!(low.inspect, projection.inspect.lower);
    assert_eq!(high.inspect, projection.inspect.upper);
}

#[test]
fn cost_below_harm_moves_evasion_and_leader_payoff() {
    let projection = inspection_nash_projection(
        game(1, 4, 1, 2),
        interval(InspectionParameter::InspectionCost, (1, 1), (2, 1)),
    )
    .expect("projection");
    assert_eq!(projection.inspect.lower, ratio(1, 3));
    assert_eq!(projection.inspect.upper, ratio(1, 3));
    assert_eq!(projection.evade.lower, ratio(1, 4));
    assert_eq!(projection.evade.upper, ratio(1, 2));
    assert_eq!(projection.leader.lower, ratio(-2, 1));
    assert_eq!(projection.leader.upper, ratio(-1, 1));
    assert_eq!(projection.follower.lower, ratio(0, 1));
    assert_eq!(projection.endpoint_gap, ratio(0, 1));
}

#[test]
fn cost_above_harm_stays_at_the_pure_witness() {
    let projection = inspection_nash_projection(
        game(5, 4, 1, 2),
        interval(InspectionParameter::InspectionCost, (5, 1), (6, 1)),
    )
    .expect("projection");
    assert_eq!(
        (
            projection.inspect.lower,
            projection.evade.lower,
            projection.leader.lower,
            projection.follower.lower
        ),
        (ratio(0, 1), ratio(1, 1), ratio(-4, 1), ratio(1, 1))
    );
    assert_eq!(projection.inspect.lower, projection.inspect.upper);
    assert_eq!(projection.leader.lower, projection.leader.upper);
}

#[test]
fn harm_below_cost_moves_only_the_leader_payoff() {
    let projection = inspection_nash_projection(
        game(5, 1, 1, 2),
        interval(InspectionParameter::UndetectedHarm, (1, 1), (2, 1)),
    )
    .expect("projection");
    assert_eq!(projection.inspect.lower, ratio(0, 1));
    assert_eq!(projection.evade.lower, ratio(1, 1));
    assert_eq!(projection.leader.lower, ratio(-2, 1));
    assert_eq!(projection.leader.upper, ratio(-1, 1));
    assert_eq!(projection.follower.lower, ratio(1, 1));
    assert_eq!(projection.endpoint_gap, ratio(0, 1));
}

#[test]
fn crossing_cost_projects_the_indifference_segment_without_filling_the_box() {
    let projection = inspection_nash_projection(
        game(1, 4, 1, 2),
        interval(InspectionParameter::InspectionCost, (1, 1), (5, 1)),
    )
    .expect("projection");
    assert_eq!(projection.inspect.lower, ratio(0, 1));
    assert_eq!(projection.inspect.upper, ratio(1, 3));
    assert_eq!(projection.evade.lower, ratio(1, 4));
    assert_eq!(projection.evade.upper, ratio(1, 1));
    assert_eq!(projection.leader.lower, ratio(-4, 1));
    assert_eq!(projection.leader.upper, ratio(-1, 1));
    assert_eq!(projection.follower.lower, ratio(0, 1));
    assert_eq!(projection.follower.upper, ratio(1, 1));
    assert_eq!(projection.endpoint_gap, ratio(0, 1));

    for cost in [1, 4, 5] {
        let gains = inspection_deviation_gains(game(cost, 4, 1, 2), ratio(1, 6), ratio(1, 4))
            .expect("gains");
        assert!(gains.leader > ratio(0, 1) || gains.follower > ratio(0, 1));
    }
}

#[test]
fn a_point_interval_matches_the_unique_witness() {
    let projection = inspection_nash_projection(
        game(1, 4, 1, 2),
        interval(InspectionParameter::EvasionGain, (1, 1), (1, 1)),
    )
    .expect("point");
    let witness = inspection_simultaneous_nash(game(1, 4, 1, 2)).expect("Nash");
    assert_eq!(projection.inspect.lower, witness.inspect);
    assert_eq!(projection.inspect.upper, witness.inspect);
    assert_eq!(projection.evade.lower, witness.evade);
    assert_eq!(projection.leader.lower, witness.payoffs.leader);
    assert_eq!(projection.follower.lower, witness.payoffs.follower);
}

#[test]
fn an_interior_rectangle_spans_the_corner_witnesses() {
    let projection = inspection_nash_rectangle(
        game(1, 4, 1, 2),
        interval(InspectionParameter::InspectionCost, (1, 1), (2, 1)),
        interval(InspectionParameter::UndetectedHarm, (4, 1), (5, 1)),
    )
    .expect("rectangle");
    assert_eq!(projection.endpoint_gap, ratio(0, 1));
    assert_eq!(projection.inspect.lower, ratio(1, 3));
    assert_eq!(projection.inspect.upper, ratio(1, 3));
    assert_eq!(projection.evade.lower, ratio(1, 5));
    assert_eq!(projection.evade.upper, ratio(1, 2));
    assert_eq!(projection.leader.lower, ratio(-2, 1));
    assert_eq!(projection.leader.upper, ratio(-1, 1));
    assert_eq!(projection.follower.lower, ratio(0, 1));
    let low = inspection_simultaneous_nash(game(1, 5, 1, 2)).expect("low corner");
    let high = inspection_simultaneous_nash(game(2, 4, 1, 2)).expect("high corner");
    assert_eq!(low.evade, projection.evade.lower);
    assert_eq!(high.evade, projection.evade.upper);
    assert_eq!(low.payoffs.leader, projection.leader.upper);
    assert_eq!(high.payoffs.leader, projection.leader.lower);
}

#[test]
fn a_pure_rectangle_moves_harm_and_gain_at_the_corners() {
    let projection = inspection_nash_rectangle(
        game(5, 1, 1, 2),
        interval(InspectionParameter::UndetectedHarm, (1, 1), (2, 1)),
        interval(InspectionParameter::EvasionGain, (1, 1), (3, 1)),
    )
    .expect("pure rectangle");
    assert_eq!(projection.inspect.lower, ratio(0, 1));
    assert_eq!(projection.evade.lower, ratio(1, 1));
    assert_eq!(projection.leader.lower, ratio(-2, 1));
    assert_eq!(projection.leader.upper, ratio(-1, 1));
    assert_eq!(projection.follower.lower, ratio(1, 1));
    assert_eq!(projection.follower.upper, ratio(3, 1));
}

#[test]
fn a_rectangle_that_mixes_regimes_or_repeats_a_parameter_is_rejected() {
    let mixed = inspection_nash_rectangle(
        game(1, 4, 1, 2),
        interval(InspectionParameter::InspectionCost, (1, 1), (5, 1)),
        interval(InspectionParameter::UndetectedHarm, (4, 1), (4, 1)),
    );
    assert_eq!(mixed, Err(InspectionError::RectangleRegimeChange));
    let repeated = inspection_nash_rectangle(
        game(1, 4, 1, 2),
        interval(InspectionParameter::InspectionCost, (1, 1), (2, 1)),
        interval(InspectionParameter::InspectionCost, (1, 1), (2, 1)),
    );
    assert_eq!(repeated, Err(InspectionError::RepeatedParameter));
}

fn family_box() -> [ParameterInterval; 4] {
    [
        interval(InspectionParameter::InspectionCost, (1, 1), (2, 1)),
        interval(InspectionParameter::UndetectedHarm, (6, 1), (8, 1)),
        interval(InspectionParameter::EvasionGain, (1, 1), (2, 1)),
        interval(InspectionParameter::DetectedLoss, (2, 1), (4, 1)),
    ]
}

#[test]
fn a_four_parameter_box_spans_its_sixteen_corners() {
    let projection = inspection_nash_box(family_box()).expect("box");
    assert_eq!(projection.endpoint_gap, ratio(0, 1));
    assert_eq!(projection.inspect.lower, ratio(1, 5));
    assert_eq!(projection.inspect.upper, ratio(1, 2));
    assert_eq!(projection.evade.lower, ratio(1, 8));
    assert_eq!(projection.evade.upper, ratio(1, 3));
    assert_eq!(projection.leader.lower, ratio(-2, 1));
    assert_eq!(projection.leader.upper, ratio(-1, 1));
    assert_eq!(projection.follower.lower, ratio(0, 1));
    let corner = inspection_simultaneous_nash(game(1, 8, 1, 4)).expect("corner");
    assert_eq!(corner.inspect, ratio(1, 5));
    assert_eq!(corner.evade, ratio(1, 8));
    let gains =
        inspection_deviation_gains(game(1, 8, 1, 4), corner.inspect, corner.evade).expect("gains");
    assert_eq!(gains.leader, ratio(0, 1));
    assert_eq!(gains.follower, ratio(0, 1));
}

#[test]
fn a_box_projection_is_not_a_joint_equilibrium() {
    let projection = inspection_nash_box(family_box()).expect("box");
    assert_eq!(projection.leader.upper, ratio(-1, 1));
    assert_eq!(projection.evade.upper, ratio(1, 3));
    let low_cost = inspection_simultaneous_nash(game(1, 6, 1, 2)).expect("low cost");
    assert_eq!(low_cost.payoffs.leader, ratio(-1, 1));
    assert!(low_cost.evade < ratio(1, 3));
}

#[test]
fn a_box_that_mixes_regimes_or_repeats_a_parameter_is_rejected() {
    let mut mixed = family_box();
    mixed[0] = interval(InspectionParameter::InspectionCost, (1, 1), (8, 1));
    mixed[1] = interval(InspectionParameter::UndetectedHarm, (4, 1), (6, 1));
    assert_eq!(
        inspection_nash_box(mixed),
        Err(InspectionError::BoxRegimeChange)
    );
    let mut repeated = family_box();
    repeated[3] = interval(InspectionParameter::InspectionCost, (1, 1), (2, 1));
    assert_eq!(
        inspection_nash_box(repeated),
        Err(InspectionError::RepeatedParameter)
    );
}

#[test]
fn a_convex_hull_spans_its_vertices_and_covers_an_interior_point() {
    let vertices = [game(1, 6, 1, 4), game(2, 8, 2, 2), game(1, 8, 1, 2)];
    let projection = inspection_nash_hull(&vertices).expect("hull");
    assert_eq!(projection.inspect.lower, ratio(1, 5));
    assert_eq!(projection.inspect.upper, ratio(1, 2));
    assert_eq!(projection.evade.lower, ratio(1, 8));
    assert_eq!(projection.evade.upper, ratio(1, 4));
    assert_eq!(projection.leader.lower, ratio(-2, 1));
    assert_eq!(projection.leader.upper, ratio(-1, 1));
    assert_eq!(projection.follower.lower, ratio(0, 1));
    let midpoint =
        InspectionParameters::try_new(ratio(3, 2), ratio(7, 1), ratio(3, 2), ratio(3, 1))
            .expect("midpoint");
    let witness = inspection_simultaneous_nash(midpoint).expect("midpoint nash");
    assert_eq!(witness.inspect, ratio(1, 3));
    assert_eq!(witness.evade, ratio(3, 14));
    assert!(witness.inspect >= projection.inspect.lower);
    assert!(witness.inspect <= projection.inspect.upper);
    assert!(witness.evade >= projection.evade.lower);
    assert!(witness.evade <= projection.evade.upper);
}

#[test]
fn a_hull_that_is_empty_or_mixed_is_rejected() {
    assert_eq!(inspection_nash_hull(&[]), Err(InspectionError::EmptyHull));
    let mixed = [game(1, 4, 1, 2), game(5, 4, 1, 2)];
    assert_eq!(
        inspection_nash_hull(&mixed),
        Err(InspectionError::HullRegimeChange)
    );
}
