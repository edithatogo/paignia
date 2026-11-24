"""Synthetic asserted game graphs and independent numerical checks."""

from itertools import permutations, product
import math

from rdflib import Graph, Literal, Namespace, RDF, URIRef

from verify import BRIDGE, GT, require

KUHN = Namespace("urn:paignia:coverage:kuhn:")
CARDS = ("J", "Q", "K")
DEALS = tuple(permutations(CARDS, 2))


def named_payoffs(graph, profile, players, values):
    graph.add((profile, RDF.type, GT.PayoffProfile))
    for index, (player, value) in enumerate(zip(players, values, strict=True)):
        # Each recipient and value have a separate source record.
        link = URIRef(f"{profile}:payoff:{index}")
        graph.add((profile, GT.hasPayoffForPlayer, link))
        graph.add((link, RDF.type, GT.PlayerPayoffLink))
        graph.add((link, GT.payoffPlayer, player))
        graph.add((link, GT.payoffValue, Literal(float(value))))


def kuhn_graph():
    graph = Graph()
    spec, rep, root = KUHN.game, KUHN.tree, KUHN.deal
    players = KUHN.p1, KUHN.p2
    graph.add((spec, RDF.type, GT.GameSpecification))
    graph.add((spec, GT.hasRepresentation, rep))
    graph.add((rep, RDF.type, GT.ExtensiveFormRepresentation))
    graph.add((spec, GT.hasRepresentation, KUHN.strategic_form))
    graph.add((KUHN.strategic_form, RDF.type, GT.NormalFormRepresentation))
    graph.add((spec, GT.hasInitialState, root))
    graph.add((root, RDF.type, GT.ChanceNode))
    graph.add((spec, GT.hasState, root))
    for player in players:
        graph.add((spec, GT.hasPlayer, player))
        graph.add((player, RDF.type, GT.Player))
        graph.add((player, GT.identifier, Literal(str(player).rsplit(":", 1)[-1])))
    for action in ("check", "bet", "fold", "call"):
        graph.add((KUHN[action], RDF.type, GT.Action))

    def transition(start, end, position, action=None):
        edge = KUHN[f"edge:{start.removeprefix(str(KUHN))}:{position}"]
        graph.add((edge, RDF.type, GT.Transition))
        graph.add((start, GT.hasTransition, edge))
        graph.add((edge, GT.transitionsFrom, start))
        graph.add((edge, GT.transitionsTo, end))
        graph.add((edge, BRIDGE.position, Literal(position)))
        if action is None:
            graph.add((edge, GT.probability, Literal(1.0 / 6.0)))
        else:
            graph.add((edge, BRIDGE.action, KUHN[action]))
            graph.add((start, GT.hasAvailableAction, KUHN[action]))

    for deal_index, (first, second) in enumerate(DEALS):
        prefix = f"{first}{second}:"
        transition(root, KUHN[prefix + "p1"], deal_index)
        for name, player, set_id, moves in [
            ("p1", KUHN.p1, f"p1-{first}", [("check", "p2-pass"), ("bet", "p2-face")]),
            ("p2-pass", KUHN.p2, f"p2-{second}-check", [("check", "cc"), ("bet", "p1-face")]),
            ("p2-face", KUHN.p2, f"p2-{second}-bet", [("fold", "bf"), ("call", "bc")]),
            ("p1-face", KUHN.p1, f"p1-{first}-face", [("fold", "cbf"), ("call", "cbc")]),
        ]:
            node, group = KUHN[prefix + name], KUHN[set_id]
            graph.add((node, RDF.type, GT.DecisionNode))
            graph.add((spec, GT.hasState, node))
            graph.add((node, GT.belongsToInformationSet, group))
            graph.add((group, RDF.type, GT.InformationSet))
            graph.add((player, GT.controlsNode, node))
            for index, (action, target) in enumerate(moves):
                transition(node, KUHN[prefix + target], index, action)
        sign = 1 if CARDS.index(first) > CARDS.index(second) else -1
        for name, value in [("cc", sign), ("bf", 1), ("bc", 2 * sign), ("cbf", -1), ("cbc", 2 * sign)]:
            node, profile = KUHN[prefix + name], KUHN[prefix + name + ":payoffs"]
            graph.add((spec, GT.hasState, node))
            graph.add((node, RDF.type, GT.TerminalNode))
            graph.add((node, BRIDGE.payoffProfile, profile))
            named_payoffs(graph, profile, players, (value, -value))
    return graph


def normal_graph(case, actions, cells):
    ns = Namespace(f"urn:paignia:coverage:{case}:")
    graph = Graph()
    graph.add((ns.game, RDF.type, GT.GameSpecification))
    graph.add((ns.game, GT.hasRepresentation, ns.matrix))
    graph.add((ns.matrix, RDF.type, GT.NormalFormRepresentation))
    if case == "matching-pennies":
        graph.add((ns.game, GT.hasRepresentation, ns.extensive))
        graph.add((ns.extensive, RDF.type, GT.ExtensiveFormRepresentation))
    players = (ns.row, ns.column)
    strategies = []
    for player, seat in zip(players, ("row", "column"), strict=True):
        graph.add((ns.game, GT.hasPlayer, player))
        graph.add((player, RDF.type, GT.Player))
        graph.add((player, GT.identifier, Literal(seat)))
        ids = tuple(ns[f"{seat}:{action}"] for action in actions)
        for strategy in ids:
            graph.add((player, GT.hasStrategy, strategy))
            graph.add((strategy, RDF.type, GT.Strategy))
        strategies.append(ids)
    for index, (joint, cell) in enumerate(zip(product(*strategies), cells, strict=True)):
        profile, payoffs = ns[f"cell:{index}"], ns[f"cell:{index}:payoffs"]
        graph.add((ns.matrix, BRIDGE.payoffCell, profile))
        graph.add((profile, RDF.type, GT.StrategyProfile))
        for strategy in joint:
            graph.add((profile, GT.hasStrategy, strategy))
            if case != "matching-pennies":
                graph.add((profile, GT.comprisesStrategy, strategy))
        graph.add((profile, GT.hasPayoff, payoffs))
        named_payoffs(graph, payoffs, players, cell)
    return graph


def kuhn_value(strategy):
    outcomes = []
    for first, second in DEALS:
        showdown = 1 if CARDS.index(first) > CARDS.index(second) else -1
        bet = strategy[f"p1-{first}"]
        call = strategy[f"p2-{second}-bet"]
        continued_bet = (1 - call) + call * 2 * showdown
        reply_bet = strategy[f"p2-{second}-check"]
        reply_call = strategy[f"p1-{first}-face"]
        continued_check = (1 - reply_bet) * showdown + reply_bet * ((1 - reply_call) * -1 + reply_call * 2 * showdown)
        outcomes.append(bet * continued_bet + (1 - bet) * continued_check)
    return math.fsum(outcomes) / len(DEALS)


def check_kuhn(result, reference):
    rows = result["policy"]
    strategy = {}
    for set_id, choices in rows.items():
        short = set_id.removeprefix(str(KUHN))
        expected_actions = ("fold", "call") if short.endswith(("-face", "-bet")) else ("check", "bet")
        require([row[0] for row in choices] == [str(KUHN[action]) for action in expected_actions],
                f"Wrong ordered action identities at {set_id}")
        require(all(0.0 <= probability <= 1.0 for _, probability in choices), "Invalid CFR action probability")
        require(abs(math.fsum(prob for _, prob in choices) - 1.0) < 1e-9, "Wrong CFR probability mass")
        strategy[short] = choices[1][1]
    seats = tuple(sorted(key for key in strategy if key.startswith(seat)) for seat in ("p1-", "p2-"))
    require(tuple(map(len, seats)) == (6, 6), "Expected six information sets for each Kuhn player")

    value = kuhn_value(strategy)
    best_responses = [
        extremum(kuhn_value(strategy | dict(zip(keys, deviations, strict=True)))
                 for deviations in product((0.0, 1.0), repeat=len(keys)))
        for keys, extremum in zip(seats, (max, min), strict=True)
    ]
    gap = best_responses[0] - best_responses[1]
    require(abs(result["expected_payoffs"][str(KUHN.p1)] - value) < 1e-9, "Kuhn value disagrees with the oracle")
    require(abs(result["expected_payoffs"][str(KUHN.p2)] + value) < 1e-9, "Kuhn zero-sum value changed")
    require(abs(result["nash_conv"] - gap) < 1e-8, "Kuhn NashConv disagrees with pure-deviation enumeration")
    require(abs(value - (-1.0 / 18.0)) < reference["value_absolute_tolerance"], "Kuhn value outside tolerance")
    require(gap < reference["final_nash_conv_upper_bound"], "Kuhn strategy exceeds the exploitability bound")
