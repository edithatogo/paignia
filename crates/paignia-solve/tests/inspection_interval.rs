//! Exact Nash projections for one moving inspection parameter.

use paignia_solve::{
    inspection_deviation_gains, inspection_nash_projection, inspection_simultaneous_nash,
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
