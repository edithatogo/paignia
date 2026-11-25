use paignia_core::{ActionId, PlayerId};
use paignia_solve::{
    evaluate_mixed_nash_candidate, evaluate_pure_nash_candidate, MixedNashConfig, NormalFormGame,
    PureNashEvaluationError, StrategyProfile,
};

fn normal_form(payoff: [[f64; 2]; 2]) -> (NormalFormGame, StrategyProfile) {
    // Both players choose between two actions, with zero-sum payoffs.
    let row = PlayerId("row".into());
    let column = PlayerId("column".into());
    let heads = ActionId("heads".into());
    let tails = ActionId("tails".into());
    let actions = [heads.clone(), tails];
    let mut payoffs = std::collections::BTreeMap::new();
    for (i, a) in actions.iter().enumerate() {
        for (j, b) in actions.iter().enumerate() {
            payoffs.insert(
                [(row.clone(), a.clone()), (column.clone(), b.clone())].into(),
                [(row.clone(), payoff[i][j]), (column.clone(), -payoff[i][j])].into(),
            );
        }
    }
    let profile = [(row.clone(), heads.clone()), (column.clone(), heads)].into();
    let strategies = [
        (row.clone(), actions.to_vec()),
        (column.clone(), actions.to_vec()),
    ]
    .into();
    (
        NormalFormGame {
            players: vec![row, column],
            strategies,
            payoffs,
        },
        profile,
    )
}

#[test]
fn pure_and_one_hot_mixed_profiles_report_the_same_deviations() {
    let (game, profile) = normal_form([[1.0, -1.0], [-1.0, 1.0]]);
    let pure = evaluate_pure_nash_candidate(&game, &profile).unwrap();
    let mixed = evaluate_mixed_nash_candidate(
        &[vec![1.0, -1.0], vec![-1.0, 1.0]],
        &[vec![-1.0, 1.0], vec![1.0, -1.0]],
        &[1.0, 0.0],
        &[1.0, 0.0],
        MixedNashConfig::default(),
    )
    .unwrap()
    .deviation_report();

    assert_eq!(pure.players.len(), 2);
    for (index, (payoff, gain)) in [(1.0, 0.0), (-1.0, 2.0)].into_iter().enumerate() {
        let row = &pure.players[index];
        assert_eq!(row.player, game.players[index]);
        assert!((row.expected_payoff - payoff).abs() <= f64::EPSILON);
        assert!((row.gain - gain).abs() <= f64::EPSILON);
        assert!((row.expected_payoff - mixed.players[index].expected_payoff).abs() <= f64::EPSILON);
        assert!((row.gain - mixed.players[index].gain).abs() <= f64::EPSILON);
    }
    assert!((pure.max_gain - 2.0).abs() <= f64::EPSILON);
    assert!((pure.nash_conv - 2.0).abs() <= f64::EPSILON);
    assert!((mixed.max_gain - pure.max_gain).abs() <= f64::EPSILON);
    assert!((mixed.nash_conv - pure.nash_conv).abs() <= f64::EPSILON);
}

#[test]
fn pure_diagnostics_reject_out_of_domain_strategies_and_overflow() {
    let (game, mut profile) = normal_form([[f64::MAX, -f64::MAX], [0.0, 0.0]]);
    assert_eq!(
        evaluate_pure_nash_candidate(&game, &profile),
        Err(PureNashEvaluationError::NonFiniteGain)
    );
    profile.insert(game.players[0].clone(), ActionId("invalid".into()));
    assert_eq!(
        evaluate_pure_nash_candidate(&game, &profile),
        Err(PureNashEvaluationError::InvalidProfile)
    );
}
