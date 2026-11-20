//! CFR/CFR+ on admitted information-set graphs, checked against Kuhn's value.

use paignia_core::{
    build_information_set_graph, ActionEdge, ActionId, ChanceEdge, Game, GameSpecification,
    InformationSetDeclaration, InformationSetDefinition, InformationSetError, InformationSetId,
    Node, NodeId, PlayerId, INFORMATION_SET_PROFILE_V1, INFORMATION_SET_SCHEMA_VERSION,
};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use paignia_solve::{
    cfr_best_response_value, cfr_nash_conv, cfr_profile_values, train_counterfactual_regret,
    CfrConfig, CfrError, CfrStrategy, CfrVariant,
};

const KUHN_VALUE: f64 = -1.0 / 18.0;
const KUHN_ITERATIONS: u32 = 12_000;

#[test]
fn rejects_an_empty_iteration_budget_before_training() {
    let (source, declaration) = one_action_chance_game();
    let error = train_counterfactual_regret(
        &source,
        &declaration,
        &CfrConfig {
            variant: CfrVariant::Cfr,
            iterations: 0,
        },
    )
    .expect_err("zero iterations are outside the budget");

    assert_eq!(error, CfrError::IterationBudget { iterations: 0 });
}

#[test]
fn rejects_imperfect_recall_before_any_strategy_is_returned() {
    let (source, declaration) = imperfect_recall();
    let error = train_counterfactual_regret(
        &source,
        &declaration,
        &CfrConfig {
            variant: CfrVariant::Cfr,
            iterations: 1,
        },
    )
    .expect_err("imperfect recall is not a trainable profile");

    assert!(matches!(
        error,
        CfrError::InformationSet(InformationSetError::ImperfectRecall { .. })
    ));
}

#[test]
fn chance_weights_a_one_action_game_exactly() {
    let (source, declaration) = one_action_chance_game();
    let trained = train_counterfactual_regret(
        &source,
        &declaration,
        &CfrConfig {
            variant: CfrVariant::Cfr,
            iterations: 1,
        },
    )
    .expect("one legal action trains");
    let alice = player("alice");

    assert!((trained.root_values[&alice] - 1.0).abs() < 1e-12);
    for probabilities in trained.average_strategy.probabilities.values() {
        assert!((probabilities.iter().sum::<f64>() - 1.0).abs() < 1e-12);
    }
}

#[test]
fn learns_the_strictly_better_shared_action() {
    let (source, declaration) = dominated_chance_game();
    let trained = train_counterfactual_regret(
        &source,
        &declaration,
        &CfrConfig {
            variant: CfrVariant::CfrPlus,
            iterations: 2_000,
        },
    )
    .expect("dominated action trains");
    let set = InformationSetId("urn:paignia:cfr:set:shared".into());
    let good = trained.average_strategy.action_order[&set]
        .iter()
        .position(|action| action.0.ends_with(":good"))
        .expect("good action is declared");

    assert!(trained.average_strategy.probabilities[&set][good] > 0.9);
    assert!(trained.root_values[&player("alice")] > 2.45);
    assert!(trained.root_values[&player("alice")] <= 2.5 + 1e-9);
}

#[test]
fn preserves_declared_action_order() {
    let (source, declaration) = one_action_chance_game();
    let trained = train_counterfactual_regret(
        &source,
        &declaration,
        &CfrConfig {
            variant: CfrVariant::Cfr,
            iterations: 1,
        },
    )
    .expect("order probe trains");
    let set = InformationSetId("urn:paignia:cfr:set:low".into());

    assert_eq!(
        trained.average_strategy.action_order[&set],
        vec![action("zebra"), action("alpha")]
    );
}

#[test]
fn kuhn_average_value_and_nash_conv_match_the_independent_reference() {
    let fixture = include_str!("../../../fixtures/conformance/cfr-reference-v1.json");
    assert!(fixture.contains("\"player_one_value\": \"-1/18\""));

    let (source, declaration) = kuhn();
    let graph = build_information_set_graph(&source, &declaration).expect("Kuhn is admissible");
    for variant in [CfrVariant::Cfr, CfrVariant::CfrPlus] {
        let trained = train_counterfactual_regret(
            &source,
            &declaration,
            &CfrConfig {
                variant,
                iterations: KUHN_ITERATIONS,
            },
        )
        .expect("Kuhn trains");
        let probabilities = kuhn_probabilities(&trained.average_strategy);
        let independent_value = kuhn_value(&probabilities);
        let library_value = trained.root_values[&player("one")];
        let independent_nash_conv = kuhn_best_responses(&probabilities).iter().sum::<f64>();
        let library_nash_conv =
            cfr_nash_conv(&source, &graph, &trained.average_strategy).expect("nash conv");

        assert!(
            (library_value - independent_value).abs() < 1e-9,
            "{variant:?} library {library_value} independent {independent_value}"
        );
        assert!(
            (independent_value - KUHN_VALUE).abs() < 5e-3,
            "{variant:?} value {independent_value} target {KUHN_VALUE}"
        );
        assert!(
            independent_nash_conv < 2e-2,
            "{variant:?} NashConv {independent_nash_conv}"
        );
        assert!(
            (library_nash_conv - independent_nash_conv).abs() < 1e-8,
            "{variant:?} library {library_nash_conv} independent {independent_nash_conv}"
        );
    }
}

#[test]
fn kuhn_nash_conv_shrinks_and_value_stays_within_tolerance_at_three_budgets() {
    let fixture = include_str!("../../../fixtures/conformance/cfr-reference-v1.json");
    assert!(fixture.contains("\"iteration_counts\": [100, 1000, 12000]"));

    let (source, declaration) = kuhn();
    let graph = build_information_set_graph(&source, &declaration).expect("Kuhn is admissible");
    for variant in [CfrVariant::Cfr, CfrVariant::CfrPlus] {
        let mut observed = Vec::new();
        for iterations in [100_u32, 1_000, 12_000] {
            let trained = train_counterfactual_regret(
                &source,
                &declaration,
                &CfrConfig {
                    variant,
                    iterations,
                },
            )
            .expect("Kuhn trains at this budget");
            let probabilities = kuhn_probabilities(&trained.average_strategy);
            let independent_value = kuhn_value(&probabilities);
            let library_value = trained.root_values[&player("one")];
            let responses = kuhn_best_responses(&probabilities);
            for (seat, expected) in [player("one"), player("two")].iter().zip(responses) {
                let actual =
                    cfr_best_response_value(&source, &graph, &trained.average_strategy, seat)
                        .expect("Kuhn best response");
                assert!(
                    (actual - expected).abs() < 1e-9,
                    "{variant:?} {iterations} {seat:?}: {actual} vs {expected}"
                );
            }
            let independent_nash_conv = responses.iter().sum::<f64>();
            let library_nash_conv =
                cfr_nash_conv(&source, &graph, &trained.average_strategy).expect("nash conv");
            assert!(
                (library_value - independent_value).abs() < 1e-9,
                "{variant:?} {iterations} library {library_value} independent {independent_value}"
            );
            assert!(
                (library_nash_conv - independent_nash_conv).abs() < 1e-8,
                "{variant:?} {iterations} library {library_nash_conv} independent {independent_nash_conv}"
            );
            let value_error = (independent_value - KUHN_VALUE).abs();
            assert!(
                value_error < 5e-3,
                "{variant:?} {iterations} value error {value_error}"
            );
            observed.push(independent_nash_conv);
        }
        assert!(
            observed[1] < observed[0] && observed[2] < observed[1],
            "{variant:?} NashConv {observed:?} does not shrink across 100, 1000, and 12000"
        );
    }
}

fn node(value: &str) -> NodeId {
    NodeId(format!("urn:paignia:cfr:node:{value}"))
}

fn action(value: &str) -> ActionId {
    ActionId(format!("urn:paignia:cfr:action:{value}"))
}

fn player(value: &str) -> PlayerId {
    PlayerId(format!("urn:paignia:cfr:player:{value}"))
}

fn decision(owner: &str, edges: &[(&str, &str)]) -> Node {
    Node::Decision {
        player: player(owner),
        edges: edges
            .iter()
            .map(|(label, target)| ActionEdge {
                action: action(label),
                target: node(target),
            })
            .collect(),
    }
}

fn chance(edges: &[(f64, &str)]) -> Node {
    Node::Chance {
        edges: edges
            .iter()
            .map(|(probability, target)| ChanceEdge {
                probability: *probability,
                target: node(target),
            })
            .collect(),
    }
}

fn terminal(payoffs: &[(&str, f64)]) -> Node {
    Node::Terminal {
        payoffs: payoffs
            .iter()
            .map(|(owner, payoff)| (player(owner), *payoff))
            .collect(),
    }
}

fn specification(root: &str, nodes: Vec<(&str, Node)>) -> GameSpecification {
    GameSpecification {
        schema_version: "1.0.0".into(),
        id: "urn:paignia:cfr:specification".into(),
        game: Game {
            root: node(root),
            nodes: nodes
                .into_iter()
                .map(|(id, value)| (node(id), value))
                .collect(),
        },
    }
}

fn declared(sets: Vec<(&str, &str, &[&str])>) -> InformationSetDeclaration {
    InformationSetDeclaration {
        schema_version: INFORMATION_SET_SCHEMA_VERSION.into(),
        semantic_profile: INFORMATION_SET_PROFILE_V1.into(),
        sets: sets
            .into_iter()
            .map(|(id, owner, members)| InformationSetDefinition {
                id: InformationSetId(format!("urn:paignia:cfr:set:{id}")),
                player: player(owner),
                members: members.iter().map(|member| node(member)).collect(),
            })
            .collect(),
    }
}

fn one_action_chance_game() -> (GameSpecification, InformationSetDeclaration) {
    let source = specification(
        "root",
        vec![
            ("root", chance(&[(0.25, "low"), (0.75, "high")])),
            (
                "low",
                decision("alice", &[("zebra", "low-pay"), ("alpha", "low-pay")]),
            ),
            ("high", decision("alice", &[("take", "high-pay")])),
            ("low-pay", terminal(&[("alice", 4.0)])),
            ("high-pay", terminal(&[("alice", 0.0)])),
        ],
    );
    let declaration = declared(vec![
        ("low", "alice", &["low"]),
        ("high", "alice", &["high"]),
    ]);
    (source, declaration)
}

fn dominated_chance_game() -> (GameSpecification, InformationSetDeclaration) {
    let source = specification(
        "root",
        vec![
            ("root", chance(&[(0.25, "low"), (0.75, "high")])),
            (
                "low",
                decision("alice", &[("good", "low-good"), ("bad", "low-bad")]),
            ),
            (
                "high",
                decision("alice", &[("good", "high-good"), ("bad", "high-bad")]),
            ),
            ("low-good", terminal(&[("alice", 10.0)])),
            ("low-bad", terminal(&[("alice", 0.0)])),
            ("high-good", terminal(&[("alice", 0.0)])),
            ("high-bad", terminal(&[("alice", 3.0)])),
        ],
    );
    let declaration = declared(vec![("shared", "alice", &["low", "high"])]);
    (source, declaration)
}

fn imperfect_recall() -> (GameSpecification, InformationSetDeclaration) {
    let source = specification(
        "root",
        vec![
            (
                "root",
                decision("alice", &[("a", "after-a"), ("b", "after-b")]),
            ),
            ("after-a", decision("alice", &[("x", "a-x"), ("y", "a-y")])),
            ("after-b", decision("alice", &[("x", "b-x"), ("y", "b-y")])),
            ("a-x", terminal(&[("alice", 1.0)])),
            ("a-y", terminal(&[("alice", 2.0)])),
            ("b-x", terminal(&[("alice", 3.0)])),
            ("b-y", terminal(&[("alice", 4.0)])),
        ],
    );
    let declaration = declared(vec![
        ("first", "alice", &["root"]),
        ("forgotten", "alice", &["after-a", "after-b"]),
    ]);
    (source, declaration)
}

fn kuhn() -> (GameSpecification, InformationSetDeclaration) {
    let cards = ["J", "Q", "K"];
    let mut deals = Vec::new();
    for first in cards {
        for second in cards {
            if first != second {
                deals.push((first, second));
            }
        }
    }
    let deal_labels: Vec<(String, String, String)> = deals
        .iter()
        .map(|(first, second)| {
            (
                (*first).to_owned(),
                (*second).to_owned(),
                format!("{first}{second}"),
            )
        })
        .collect();
    let chance_edges: Vec<(f64, String)> = deal_labels
        .iter()
        .map(|(_, _, label)| (1.0 / 6.0, format!("{label}-p1")))
        .collect();
    let chance_refs: Vec<(f64, &str)> = chance_edges
        .iter()
        .map(|(probability, label)| (*probability, label.as_str()))
        .collect();
    let mut owned_nodes = vec![("root".to_owned(), chance(&chance_refs))];
    for (first, second, label) in &deal_labels {
        owned_nodes.extend(kuhn_deal_nodes(first, second, label));
    }
    let source = GameSpecification {
        schema_version: "1.0.0".into(),
        id: "urn:paignia:cfr:kuhn".into(),
        game: Game {
            root: node("root"),
            nodes: owned_nodes
                .into_iter()
                .map(|(id, value)| (node(&id), value))
                .collect(),
        },
    };
    let sets = cards
        .into_iter()
        .flat_map(|card| kuhn_card_sets(&deal_labels, card))
        .collect();
    let declaration = InformationSetDeclaration {
        schema_version: INFORMATION_SET_SCHEMA_VERSION.into(),
        semantic_profile: INFORMATION_SET_PROFILE_V1.into(),
        sets,
    };
    (source, declaration)
}

fn kuhn_deal_nodes(first: &str, second: &str, label: &str) -> Vec<(String, Node)> {
    let stake_one = showdown(first, second, 1.0);
    let stake_two = showdown(first, second, 2.0);
    let p2_check = format!("{label}-p2-check");
    let p2_bet = format!("{label}-p2-bet");
    let checked = format!("{label}-cc");
    let face = format!("{label}-p1-face");
    let folded = format!("{label}-bf");
    let called = format!("{label}-bc");
    let check_fold = format!("{label}-cbf");
    let check_call = format!("{label}-cbc");
    vec![
        (
            format!("{label}-p1"),
            choice("one", &[("check", &p2_check), ("bet", &p2_bet)]),
        ),
        (
            format!("{label}-p2-check"),
            choice("two", &[("check", &checked), ("bet", &face)]),
        ),
        (
            format!("{label}-p2-bet"),
            choice("two", &[("fold", &folded), ("call", &called)]),
        ),
        (
            format!("{label}-p1-face"),
            choice("one", &[("fold", &check_fold), ("call", &check_call)]),
        ),
        (format!("{label}-cc"), terminal_pair(stake_one)),
        (format!("{label}-bc"), terminal_pair(stake_two)),
        (format!("{label}-cbc"), terminal_pair(stake_two)),
        (format!("{label}-bf"), terminal_pair(1.0)),
        (format!("{label}-cbf"), terminal_pair(-1.0)),
    ]
}

fn choice(owner: &str, edges: &[(&str, &str)]) -> Node {
    Node::Decision {
        player: player(owner),
        edges: edges
            .iter()
            .map(|(label, target)| ActionEdge {
                action: action(label),
                target: node(target),
            })
            .collect(),
    }
}

fn kuhn_card_sets(deals: &[(String, String, String)], card: &str) -> Vec<InformationSetDefinition> {
    let members = |suffix: &str, by_first: bool| {
        deals
            .iter()
            .filter(|(first, second, _)| {
                if by_first {
                    first == card
                } else {
                    second == card
                }
            })
            .map(|(_, _, label)| format!("{label}-{suffix}"))
            .collect::<Vec<_>>()
    };
    vec![
        set_owned(&format!("p1-{card}"), "one", &members("p1", true)),
        set_owned(&format!("p1-{card}-face"), "one", &members("p1-face", true)),
        set_owned(
            &format!("p2-{card}-check"),
            "two",
            &members("p2-check", false),
        ),
        set_owned(&format!("p2-{card}-bet"), "two", &members("p2-bet", false)),
    ]
}

fn set_owned(id: &str, owner: &str, members: &[String]) -> InformationSetDefinition {
    InformationSetDefinition {
        id: InformationSetId(format!("urn:paignia:cfr:set:{id}")),
        player: player(owner),
        members: members.iter().map(|member| node(member)).collect(),
    }
}

fn terminal_pair(player_one: f64) -> Node {
    terminal(&[("one", player_one), ("two", -player_one)])
}

fn rank(card: &str) -> u8 {
    match card {
        "J" => 0,
        "Q" => 1,
        "K" => 2,
        _ => panic!("unknown card"),
    }
}

fn showdown(first: &str, second: &str, stake: f64) -> f64 {
    if rank(first) > rank(second) {
        stake
    } else {
        -stake
    }
}

#[derive(Clone, Copy)]
struct KuhnProbabilities {
    p1_bet: [f64; 3],
    p1_call: [f64; 3],
    p2_bet: [f64; 3],
    p2_call: [f64; 3],
}

fn kuhn_probabilities(strategy: &CfrStrategy) -> KuhnProbabilities {
    let mut probabilities = KuhnProbabilities {
        p1_bet: [0.0; 3],
        p1_call: [0.0; 3],
        p2_bet: [0.0; 3],
        p2_call: [0.0; 3],
    };
    for card in 0..3 {
        let name = ["J", "Q", "K"][card];
        probabilities.p1_bet[card] = probability(strategy, &format!("p1-{name}"), "bet");
        probabilities.p1_call[card] = probability(strategy, &format!("p1-{name}-face"), "call");
        probabilities.p2_bet[card] = probability(strategy, &format!("p2-{name}-check"), "bet");
        probabilities.p2_call[card] = probability(strategy, &format!("p2-{name}-bet"), "call");
    }
    probabilities
}

fn probability(strategy: &CfrStrategy, set: &str, action_name: &str) -> f64 {
    let id = InformationSetId(format!("urn:paignia:cfr:set:{set}"));
    let index = strategy.action_order[&id]
        .iter()
        .position(|action| action.0.ends_with(&format!(":{action_name}")))
        .expect("action is present");
    strategy.probabilities[&id][index]
}

fn kuhn_value(probabilities: &KuhnProbabilities) -> f64 {
    let deals = [(0, 1), (0, 2), (1, 0), (1, 2), (2, 0), (2, 1)];
    let mut total = 0.0;
    for (first, second) in deals {
        let bet = probabilities.p1_bet[first];
        let check = 1.0 - bet;
        let second_call = probabilities.p2_call[second];
        let second_bet = probabilities.p2_bet[second];
        let first_call = probabilities.p1_call[first];
        let show = |stake: f64| if first > second { stake } else { -stake };
        let after_bet = (1.0 - second_call) * 1.0 + second_call * show(2.0);
        let after_check = (1.0 - second_bet) * show(1.0)
            + second_bet * (-(1.0 - first_call) + first_call * show(2.0));
        total += bet * after_bet + check * after_check;
    }
    total / 6.0
}

fn kuhn_best_responses(probabilities: &KuhnProbabilities) -> [f64; 2] {
    let mut best_for_one = f64::NEG_INFINITY;
    let mut worst_for_one = f64::INFINITY;
    for mask in 0..64_u32 {
        let mut deviating = *probabilities;
        for card in 0..3 {
            deviating.p1_bet[card] = f64::from((mask >> card) & 1);
            deviating.p1_call[card] = f64::from((mask >> (card + 3)) & 1);
        }
        best_for_one = best_for_one.max(kuhn_value(&deviating));
        deviating = *probabilities;
        for card in 0..3 {
            deviating.p2_bet[card] = f64::from((mask >> card) & 1);
            deviating.p2_call[card] = f64::from((mask >> (card + 3)) & 1);
        }
        worst_for_one = worst_for_one.min(kuhn_value(&deviating));
    }
    [best_for_one, -worst_for_one]
}

const LEDUC_VALUE: f64 = -0.085_606;
const LEDUC_ITERATIONS: u32 = 4_000;

#[test]
fn leduc_all_call_matches_both_seat_best_responses_and_nash_conv() {
    let (source, declaration) = leduc();
    let graph = build_information_set_graph(&source, &declaration).expect("Leduc is admissible");
    let mut calling = CfrStrategy {
        action_order: BTreeMap::new(),
        probabilities: BTreeMap::new(),
    };
    for (set_id, set) in &graph.sets {
        let call_index = set
            .legal_actions
            .iter()
            .position(|action| action.0.ends_with(":call"))
            .expect("call is legal");
        let mut probabilities = vec![0.0; set.legal_actions.len()];
        probabilities[call_index] = 1.0;
        calling
            .action_order
            .insert(set_id.clone(), set.legal_actions.clone());
        calling.probabilities.insert(set_id.clone(), probabilities);
    }
    let (showdown_sum, best_response_sum, deals) = all_call_deal_totals();
    assert_eq!((showdown_sum, best_response_sum, deals), (0, 176, 120));
    let expected_response = f64::from(best_response_sum) / f64::from(deals);
    let values = cfr_profile_values(&source, &graph, &calling).expect("all-call values");
    for seat in [player("zero"), player("one")] {
        assert!(values[&seat].abs() < 1e-12);
        let response = cfr_best_response_value(&source, &graph, &calling, &seat)
            .expect("all-call best response");
        assert!(
            (response - expected_response).abs() < 1e-12,
            "{seat:?}: {response}"
        );
    }
    let nash_conv = cfr_nash_conv(&source, &graph, &calling).expect("all-call NashConv");
    assert!((nash_conv - 44.0 / 15.0).abs() < 1e-12, "{nash_conv}");
}

#[test]
fn leduc_cfr_plus_checks_published_value_and_nash_conv() {
    let fixture = include_str!("../../../fixtures/conformance/cfr-reference-v1.json");
    assert!(fixture.contains("\"player_one_value\": -0.085606"));
    assert!(fixture.contains("\"information_states\": 936"));

    let (source, declaration) = leduc();
    assert_eq!(declaration.sets.len(), 936);
    let graph =
        build_information_set_graph(&source, &declaration).expect("Leduc is perfect recall");
    let trained = train_counterfactual_regret(
        &source,
        &declaration,
        &CfrConfig {
            variant: CfrVariant::CfrPlus,
            iterations: LEDUC_ITERATIONS,
        },
    )
    .expect("Leduc trains");
    let value = trained.root_values[&player("zero")];
    let nash_conv = cfr_nash_conv(&source, &graph, &trained.average_strategy).expect("leduc nash");
    assert!(
        (value - LEDUC_VALUE).abs() < 2e-2,
        "leduc value {value} target {LEDUC_VALUE} NashConv {nash_conv}"
    );
    assert!(nash_conv < 0.05, "leduc NashConv {nash_conv}");
}

// Against an opponent who always calls, a raise is matched exactly once per
// round. Maximize the extra 2 or 4 chips using only cards visible at that turn.
fn all_call_deal_totals() -> (i32, i32, i32) {
    let mut first_round = [0_i32; 6];
    let mut second_round = [[0_i32; 6]; 6];
    let mut deals = 0;
    for (private, boards) in second_round.iter_mut().enumerate() {
        for (board, board_total) in boards.iter_mut().enumerate() {
            if board == private {
                continue;
            }
            for other in (0..6).filter(|&card| card != private && card != board) {
                let rank = |card: usize| (card / 2 == board / 2, card / 2);
                let result = match rank(private).cmp(&rank(other)) {
                    Ordering::Greater => 1,
                    Ordering::Equal => 0,
                    Ordering::Less => -1,
                };
                deals += 1;
                first_round[private] += result;
                *board_total += result;
            }
        }
    }
    let showdown = first_round.iter().sum::<i32>();
    let first_raise = first_round.iter().copied().map(|v| v.max(0)).sum::<i32>();
    let second_raise = second_round
        .iter()
        .flatten()
        .copied()
        .map(|v| v.max(0))
        .sum::<i32>();
    (
        showdown,
        showdown + 2 * first_raise + 4 * second_raise,
        deals,
    )
}

fn leduc_hand_rank(private_card: i32, public_card: i32) -> i32 {
    let mut low = public_card;
    let mut high = private_card;
    if low > high {
        std::mem::swap(&mut low, &mut high);
    }
    if low % 2 == 0 && high == low + 1 {
        36 + low
    } else {
        (high / 2) * 6 + (low / 2)
    }
}

#[derive(Clone)]
struct LeducState {
    deck: [bool; 6],
    deck_size: i32,
    private_cards: [i32; 2],
    dealt: i32,
    public_card: i32,
    round: i32,
    stakes: i32,
    ante: [i32; 2],
    pot: i32,
    money: [i32; 2],
    raises: i32,
    calls: i32,
    folded: [bool; 2],
    remaining: i32,
    cursor: i32,
    first_round: Vec<&'static str>,
    second_round: Vec<&'static str>,
    resolved: bool,
}

fn fresh_leduc() -> LeducState {
    LeducState {
        deck: [true; 6],
        deck_size: 6,
        private_cards: [-1, -1],
        dealt: 0,
        public_card: -1,
        round: 1,
        stakes: 1,
        ante: [1, 1],
        pot: 2,
        money: [99, 99],
        raises: 0,
        calls: 0,
        folded: [false, false],
        remaining: 2,
        cursor: -1,
        first_round: Vec::new(),
        second_round: Vec::new(),
        resolved: false,
    }
}

fn leduc() -> (GameSpecification, InformationSetDeclaration) {
    let mut nodes = BTreeMap::new();
    let mut sets: BTreeMap<String, (usize, BTreeSet<String>)> = BTreeMap::new();
    expand_leduc(&fresh_leduc(), "root", &mut nodes, &mut sets);
    let source = GameSpecification {
        schema_version: "1.0.0".into(),
        id: "urn:paignia:cfr:leduc".into(),
        game: Game {
            root: node("root"),
            nodes: nodes
                .into_iter()
                .map(|(id, value)| (node(&id), value))
                .collect(),
        },
    };
    let declaration = InformationSetDeclaration {
        schema_version: INFORMATION_SET_SCHEMA_VERSION.into(),
        semantic_profile: INFORMATION_SET_PROFILE_V1.into(),
        sets: sets
            .into_iter()
            .map(|(id, (owner, members))| InformationSetDefinition {
                id: InformationSetId(format!("urn:paignia:cfr:set:{id}")),
                player: player(if owner == 0 { "zero" } else { "one" }),
                members: members.into_iter().map(|member| node(&member)).collect(),
            })
            .collect(),
    };
    (source, declaration)
}

fn expand_leduc(
    state: &LeducState,
    id: &str,
    nodes: &mut BTreeMap<String, Node>,
    sets: &mut BTreeMap<String, (usize, BTreeSet<String>)>,
) {
    if nodes.contains_key(id) {
        return;
    }
    if state.resolved {
        nodes.insert(
            id.to_owned(),
            terminal(&[
                ("zero", f64::from(state.money[0] - 100)),
                ("one", f64::from(state.money[1] - 100)),
            ]),
        );
        return;
    }
    if state.cursor < 0 {
        let probability = 1.0 / f64::from(state.deck_size);
        let mut edges = Vec::new();
        for card in 0..6_i32 {
            if !state.deck[usize::try_from(card).expect("card")] {
                continue;
            }
            let mut child = state.clone();
            child.deal(card);
            let child_id = child.identity();
            expand_leduc(&child, &child_id, nodes, sets);
            edges.push(ChanceEdge {
                probability,
                target: node(&child_id),
            });
        }
        nodes.insert(id.to_owned(), Node::Chance { edges });
        return;
    }
    let owner = usize::try_from(state.cursor).expect("player cursor");
    let actions = state.legal_actions();
    let mut edges = Vec::new();
    for label in &actions {
        let mut child = state.clone();
        child.act(label);
        let child_id = child.identity();
        expand_leduc(&child, &child_id, nodes, sets);
        edges.push(ActionEdge {
            action: action(label),
            target: node(&child_id),
        });
    }
    sets.entry(state.information_set())
        .or_insert_with(|| (owner, BTreeSet::new()))
        .1
        .insert(id.to_owned());
    nodes.insert(
        id.to_owned(),
        Node::Decision {
            player: player(if owner == 0 { "zero" } else { "one" }),
            edges,
        },
    );
}

impl LeducState {
    fn deal(&mut self, card: i32) {
        let index = usize::try_from(card).expect("card");
        self.deck[index] = false;
        self.deck_size -= 1;
        if self.dealt < 2 {
            let player_index = usize::try_from(self.dealt).expect("dealt player");
            self.private_cards[player_index] = card;
            self.dealt += 1;
            if self.dealt == 2 {
                self.cursor = 0;
            }
        } else {
            self.public_card = card;
            self.cursor = 0;
        }
    }

    fn act(&mut self, label: &str) {
        let player_index = usize::try_from(self.cursor).expect("acting player");
        if self.round == 1 {
            self.first_round.push(leduc_action_label(label));
        } else {
            self.second_round.push(leduc_action_label(label));
        }
        match label {
            "fold" => {
                self.folded[player_index] = true;
                self.remaining -= 1;
                self.finish_action(false);
            }
            "call" => {
                let amount = self.stakes - self.ante[player_index];
                self.contribute(player_index, amount);
                self.calls += 1;
                self.finish_action(false);
            }
            "raise" => {
                let call_amount = self.stakes - self.ante[player_index];
                self.contribute(player_index, call_amount);
                let raise_amount = if self.round == 1 { 2 } else { 4 };
                self.stakes += raise_amount;
                self.contribute(player_index, raise_amount);
                self.raises += 1;
                self.calls = 0;
                self.finish_action(true);
            }
            _ => panic!("unknown leduc action"),
        }
    }

    fn contribute(&mut self, player_index: usize, amount: i32) {
        self.pot += amount;
        self.ante[player_index] += amount;
        self.money[player_index] -= amount;
    }

    fn finish_action(&mut self, raised: bool) {
        if self.remaining == 1 || (self.round == 2 && self.ready()) {
            self.resolve();
            return;
        }
        if !raised && self.ready() {
            self.round = 2;
            self.raises = 0;
            self.calls = 0;
            self.cursor = -1;
            return;
        }
        self.cursor = self.next_player();
    }

    fn ready(&self) -> bool {
        (self.raises == 0 && self.calls == self.remaining)
            || (self.raises > 0 && self.calls == self.remaining - 1)
    }

    fn next_player(&self) -> i32 {
        for offset in 1..=2 {
            let candidate = (self.cursor + offset) % 2;
            let index = usize::try_from(candidate).expect("candidate");
            if !self.folded[index] {
                return candidate;
            }
        }
        panic!("no acting player");
    }

    fn resolve(&mut self) {
        if self.remaining == 1 {
            for player_index in 0..2 {
                if !self.folded[player_index] {
                    self.money[player_index] += self.pot;
                }
            }
        } else {
            let left = leduc_hand_rank(self.private_cards[0], self.public_card);
            let right = leduc_hand_rank(self.private_cards[1], self.public_card);
            match left.cmp(&right) {
                Ordering::Greater => self.money[0] += self.pot,
                Ordering::Less => self.money[1] += self.pot,
                Ordering::Equal => {
                    self.money[0] += self.pot / 2;
                    self.money[1] += self.pot / 2;
                }
            }
        }
        self.pot = 0;
        self.resolved = true;
    }

    fn legal_actions(&self) -> Vec<&'static str> {
        let player_index = usize::try_from(self.cursor).expect("legal player");
        let mut actions = Vec::new();
        if self.stakes > self.ante[player_index] {
            actions.push("fold");
        }
        actions.push("call");
        if self.raises < 2 {
            actions.push("raise");
        }
        actions
    }

    fn information_set(&self) -> String {
        format!(
            "leduc-p{}-c{}-b{}-{}",
            self.cursor,
            self.private_cards[usize::try_from(self.cursor).expect("infoset player")],
            self.public_card,
            self.sequence()
        )
    }

    fn sequence(&self) -> String {
        format!(
            "{}|{}",
            self.first_round.join(","),
            self.second_round.join(",")
        )
    }

    fn identity(&self) -> String {
        format!(
            "leduc-{}-{}-b{}-{}",
            self.private_cards[0],
            self.private_cards[1],
            self.public_card,
            self.sequence()
        )
    }
}

fn leduc_action_label(label: &str) -> &'static str {
    match label {
        "fold" => "f",
        "call" => "c",
        "raise" => "r",
        _ => panic!("unknown leduc action"),
    }
}

#[test]
fn diagnostics_reject_bad_mass_width_keys_action_order_and_graph() {
    use paignia_solve::cfr_best_response_value;
    let (source, declaration) = dominated_chance_game();
    let graph = build_information_set_graph(&source, &declaration).unwrap();
    let strategy = train_counterfactual_regret(
        &source,
        &declaration,
        &CfrConfig {
            variant: CfrVariant::Cfr,
            iterations: 1,
        },
    )
    .unwrap()
    .average_strategy;
    let set = declaration.sets[0].id.clone();
    for invalid in [
        vec![],
        vec![1.0],
        vec![1.0, 0.0, 0.0],
        vec![2.0, -1.0],
        vec![0.1, 0.1],
        vec![f64::NAN, 0.0],
        vec![f64::INFINITY, 0.0],
    ] {
        let mut malformed = strategy.clone();
        malformed.probabilities.insert(set.clone(), invalid);
        assert_eq!(
            cfr_nash_conv(&source, &graph, &malformed),
            Err(CfrError::InvalidDistribution(set.clone()))
        );
    }
    let mut missing = strategy.clone();
    missing.probabilities.clear();
    assert_eq!(
        cfr_profile_values(&source, &graph, &missing),
        Err(CfrError::StrategyDomain)
    );
    let mut reordered = strategy.clone();
    reordered.action_order.get_mut(&set).unwrap().reverse();
    assert!(matches!(
        cfr_best_response_value(&source, &graph, &reordered, &player("alice")),
        Err(CfrError::InconsistentActionOrder { .. })
    ));
    let mut stale = graph;
    stale.assignments.clear();
    assert_eq!(
        cfr_nash_conv(&source, &stale, &strategy),
        Err(CfrError::GraphMismatch)
    );
}

#[test]
fn a_deep_shared_chance_dag_uses_the_same_counterfactual_mass_as_one_edge() {
    use paignia_solve::cfr_best_response_value;
    let (mut source, declaration) = dominated_chance_game();
    // Two edges to the same child at each stage give 2^4096 histories with total mass one.
    for i in 0..4096 {
        let next = node(&format!("shared-chance-{i}"));
        let edges = [0.25, 0.75].map(|probability| ChanceEdge {
            probability,
            target: source.game.root.clone(),
        });
        source.game.nodes.insert(
            next.clone(),
            Node::Chance {
                edges: edges.to_vec(),
            },
        );
        source.game.root = next;
    }
    let graph = build_information_set_graph(&source, &declaration).unwrap();
    for variant in [CfrVariant::Cfr, CfrVariant::CfrPlus] {
        let trained = train_counterfactual_regret(
            &source,
            &declaration,
            &CfrConfig {
                variant,
                iterations: 5,
            },
        )
        .unwrap();
        // The shared actions yield 2.5 and 2.25. One uniform iteration and
        // four iterations of the first action give an average (0.9, 0.1).
        let row = &trained.average_strategy.probabilities[&declaration.sets[0].id];
        assert!((row[0] - 0.9).abs() < 1e-12);
        let best =
            cfr_best_response_value(&source, &graph, &trained.average_strategy, &player("alice"))
                .unwrap();
        assert!((best - 2.5).abs() < 1e-12);
        assert!((trained.root_values[&player("alice")] - 2.475).abs() < 1e-12);
    }
}

#[test]
fn extreme_payoffs_preserve_regret_matching_proportions() {
    let source = specification(
        "root",
        vec![
            (
                "root",
                decision(
                    "alice",
                    &[("w1", "up"), ("w2", "up"), ("l1", "down"), ("l2", "down")],
                ),
            ),
            ("up", terminal(&[("alice", 1e308)])),
            ("down", terminal(&[("alice", -1e308)])),
        ],
    );
    let declaration = declared(vec![("root", "alice", &["root"])]);
    for variant in [CfrVariant::Cfr, CfrVariant::CfrPlus] {
        let trained = train_counterfactual_regret(
            &source,
            &declaration,
            &CfrConfig {
                variant,
                iterations: 2,
            },
        )
        .unwrap();
        assert_eq!(
            trained.average_strategy.probabilities[&declaration.sets[0].id],
            [0.375, 0.375, 0.125, 0.125]
        );
        assert!((trained.root_values[&player("alice")] / 1e308 - 0.5).abs() < 1e-14);
    }
}

#[test]
fn common_state_payoffs_cancel_before_scoring_a_best_response() {
    use paignia_solve::cfr_best_response_value;
    let large = 2_f64.powi(55);
    let source = specification(
        "root",
        vec![
            ("root", chance(&[(0.25, "a"), (0.25, "c"), (0.5, "b")])),
            ("a", decision("alice", &[("L", "pos"), ("R", "pos")])),
            ("b", decision("alice", &[("L", "zero"), ("R", "two")])),
            ("c", decision("alice", &[("L", "neg"), ("R", "neg")])),
            ("pos", terminal(&[("alice", large)])),
            ("neg", terminal(&[("alice", -large)])),
            ("zero", terminal(&[])),
            ("two", terminal(&[("alice", 2.0)])),
        ],
    );
    let declaration = declared(vec![("hidden", "alice", &["a", "b", "c"])]);
    let graph = build_information_set_graph(&source, &declaration).unwrap();
    let id = declaration.sets[0].id.clone();
    let mut strategy = CfrStrategy {
        action_order: BTreeMap::from([(id.clone(), vec![action("L"), action("R")])]),
        probabilities: BTreeMap::from([(id.clone(), vec![0.0, 1.0])]),
    };
    let alice = player("alice");
    assert!(
        (cfr_best_response_value(&source, &graph, &strategy, &alice).unwrap() - 1.0).abs()
            < f64::EPSILON
    );
    assert!(cfr_nash_conv(&source, &graph, &strategy).unwrap().abs() < f64::EPSILON);
    strategy.probabilities.insert(id, vec![1.0, 0.0]);
    assert!((cfr_nash_conv(&source, &graph, &strategy).unwrap() - 1.0).abs() < f64::EPSILON);
}

#[test]
fn positive_reach_that_underflows_is_reported() {
    use paignia_solve::cfr_best_response_value;
    let mut source = specification(
        "choose",
        vec![
            (
                "choose",
                decision("alice", &[("L", "zero"), ("R", "large")]),
            ),
            ("zero", terminal(&[])),
            ("large", terminal(&[("alice", 2_f64.powi(1023))])),
        ],
    );
    let declaration = declared(vec![("choose", "alice", &["choose"])]);
    let probability = 2_f64.powi(-49);
    for i in 0..22 {
        let root = node(&format!("tiny-chance-{i}"));
        source.game.nodes.insert(
            root.clone(),
            Node::Chance {
                edges: vec![
                    ChanceEdge {
                        probability,
                        target: source.game.root.clone(),
                    },
                    ChanceEdge {
                        probability: 1.0 - probability,
                        target: node("zero"),
                    },
                ],
            },
        );
        source.game.root = root;
    }
    let graph = build_information_set_graph(&source, &declaration).unwrap();
    let id = declaration.sets[0].id.clone();
    let strategy = CfrStrategy {
        action_order: BTreeMap::from([(id.clone(), vec![action("L"), action("R")])]),
        probabilities: BTreeMap::from([(id, vec![0.0, 1.0])]),
    };
    assert_eq!(
        cfr_best_response_value(&source, &graph, &strategy, &player("alice")),
        Err(CfrError::Underflow)
    );
    assert_eq!(
        cfr_nash_conv(&source, &graph, &strategy),
        Err(CfrError::Underflow)
    );
    for variant in [CfrVariant::Cfr, CfrVariant::CfrPlus] {
        assert_eq!(
            train_counterfactual_regret(
                &source,
                &declaration,
                &CfrConfig {
                    variant,
                    iterations: 1
                }
            ),
            Err(CfrError::Underflow)
        );
    }
}

#[test]
fn unilateral_gain_survives_rounding_of_the_two_root_values() {
    let base = 2_f64.powi(55);
    let source = specification(
        "root",
        vec![
            ("root", decision("alice", &[("L", "left"), ("R", "right")])),
            ("left", terminal(&[("alice", base)])),
            ("right", terminal(&[("alice", base + 8.0)])),
        ],
    );
    let declaration = declared(vec![("root", "alice", &["root"])]);
    let graph = build_information_set_graph(&source, &declaration).unwrap();
    let id = declaration.sets[0].id.clone();
    let strategy = CfrStrategy {
        action_order: BTreeMap::from([(id.clone(), vec![action("L"), action("R")])]),
        probabilities: BTreeMap::from([(id, vec![0.25, 0.75])]),
    };
    // Both f64 root payoffs round to base+8. Switching the 1/4 played
    // on the left to the right earns 1/4 * 8 = 2 in the original units.
    assert!((cfr_nash_conv(&source, &graph, &strategy).unwrap() - 2.0).abs() < f64::EPSILON);
}

#[test]
fn nested_lottery_preserves_the_gain_below_one_ulp_of_the_payoff() {
    let base = 2_f64.powi(54);
    let source = specification(
        "root",
        vec![
            (
                "root",
                decision("alice", &[("lottery", "lottery"), ("sure", "high")]),
            ),
            ("lottery", chance(&[(0.5, "high"), (0.5, "low")])),
            ("high", terminal(&[("alice", base)])),
            ("low", terminal(&[("alice", base - 2.0)])),
        ],
    );
    let declaration = declared(vec![("choice", "alice", &["root"])]);
    let graph = build_information_set_graph(&source, &declaration).unwrap();
    let id = declaration.sets[0].id.clone();
    let strategy = CfrStrategy {
        action_order: BTreeMap::from([(id.clone(), vec![action("lottery"), action("sure")])]),
        probabilities: BTreeMap::from([(id, vec![1.0, 0.0])]),
    };
    assert!((cfr_nash_conv(&source, &graph, &strategy).unwrap() - 1.0).abs() < f64::EPSILON);
}

#[test]
fn a_rounded_subnormal_cannot_supply_a_deviation_certificate() {
    let source = specification(
        "root",
        vec![
            (
                "root",
                chance(&[(f64::from_bits(1), "choice"), (1.0, "zero")]),
            ),
            (
                "choice",
                decision("alice", &[("L", "zero"), ("R", "positive")]),
            ),
            ("zero", terminal(&[])),
            ("positive", terminal(&[("alice", 1.5 * 2_f64.powi(1023))])),
        ],
    );
    let declaration = declared(vec![("choice", "alice", &["choice"])]);
    let graph = build_information_set_graph(&source, &declaration).unwrap();
    let id = declaration.sets[0].id.clone();
    let strategy = CfrStrategy {
        action_order: BTreeMap::from([(id.clone(), vec![action("L"), action("R")])]),
        probabilities: BTreeMap::from([(id, vec![1.0, 0.0])]),
    };
    assert_eq!(
        cfr_nash_conv(&source, &graph, &strategy),
        Err(CfrError::Underflow)
    );
}

#[test]
fn admission_accounts_for_small_probabilities_after_a_unit_mass() {
    let order: Vec<_> = (0..=16_384).map(|i| action(&i.to_string())).collect();
    let edges = order
        .iter()
        .cloned()
        .map(|action| ActionEdge {
            action,
            target: node("end"),
        })
        .collect();
    let source = specification(
        "root",
        vec![
            (
                "root",
                Node::Decision {
                    player: player("alice"),
                    edges,
                },
            ),
            ("end", terminal(&[])),
        ],
    );
    let declaration = declared(vec![("wide", "alice", &["root"])]);
    let graph = build_information_set_graph(&source, &declaration).unwrap();
    let id = declaration.sets[0].id.clone();
    let mut row = vec![2_f64.powi(-53); order.len()];
    row[0] = 1.0;
    let strategy = CfrStrategy {
        action_order: BTreeMap::from([(id.clone(), order)]),
        probabilities: BTreeMap::from([(id.clone(), row)]),
    };
    assert_eq!(
        cfr_profile_values(&source, &graph, &strategy),
        Err(CfrError::InvalidDistribution(id))
    );
}
