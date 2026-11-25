"""Measure admission, tree unfolding and equivalent backward induction."""

import json
from random import Random
import sys
from time import perf_counter_ns

from rdflib import Graph, Literal, Namespace, RDF

from benchmark import summarise, timed_process
from compare import BRIDGE, CASE, GT, digest, emit_efg, encode, parse_graph, project, static_case
from profile_scenarios import named_payoffs

WORKLOADS = (
    ("binary-shared-3", 2, 3, True),
    ("binary-tree-3", 2, 3, False),
    ("binary-shared-6", 2, 6, True),
    ("binary-tree-6", 2, 6, False),
    ("quaternary-tree-3", 4, 3, False),
)
SEATS = [CASE.acheteur, CASE["décideur"]]
PAYOFFS = (2, 4)
OPEN_SPIEL_PROGRAM = """
import json
import sys
import pyspiel

root = pyspiel.load_efg_game(sys.stdin.read()).new_initial_state()
choices = []

def solve(state):
    if state.is_terminal():
        return state.returns()
    if state.is_chance_node():
        outcomes = [(prob, solve(state.child(action))) for action, prob in state.chance_outcomes()]
        return [sum(weight * child[seat] for weight, child in outcomes) for seat in range(2)]
    decisions = [(state.action_to_string(action), solve(state.child(action))) for action in state.legal_actions()]
    owner = state.current_player()
    label, values = min(decisions, key=lambda item: (-item[1][owner], item[0]))
    choices.append(label)
    return values

values = solve(root)
print(json.dumps({"root_values": values, "decisions": len(choices), "every_choice_first": all(choice == "edge0" for choice in choices)}))
"""


def layered_graph(name, width, depth, shared):
    graph = Graph()
    ns = Namespace(f"urn:paignia:scale:{name}:")
    graph.add((ns.game, RDF.type, GT.GameSpecification))
    graph.add((ns.tree, RDF.type, GT.ExtensiveFormRepresentation))
    graph.add((ns.game, GT.hasRepresentation, ns.tree))
    graph.add((ns.game, GT.hasInitialState, ns["node:0:0"]))
    for player in SEATS:
        graph.add((ns.game, GT.hasPlayer, player))
        graph.add((player, RDF.type, GT.Player))
    for level in range(depth + 1):
        terminal = level == depth
        chance = level % 2 == 0
        for index in range(1 if shared else width**level):
            path = f"{level}:{index}"
            node = ns[f"node:{path}"]
            graph.add((ns.game, GT.hasState, node))
            if terminal:
                profile = ns[f"payoffs:{path}"]
                graph.add((node, RDF.type, GT.TerminalNode))
                graph.add((node, BRIDGE.payoffProfile, profile))
                named_payoffs(graph, profile, SEATS, PAYOFFS)
                continue
            if chance:
                graph.add((node, RDF.type, GT.ChanceNode))
            else:
                group = ns[f"information:{path}"]
                graph.add((node, RDF.type, GT.DecisionNode))
                graph.add((SEATS[level // 2 % 2], GT.controlsNode, node))
                graph.add((group, RDF.type, GT.InformationSet))
                graph.add((node, GT.belongsToInformationSet, group))
            for column in range(width):
                offset = 0 if shared else index * width + column
                edge = ns[f"edge:{path}:{column}"]
                target = ns[f"node:{level + 1}:{offset}"]
                graph.add((edge, RDF.type, GT.Transition))
                graph.add((node, GT.hasTransition, edge))
                graph.add((edge, GT.transitionsFrom, node))
                graph.add((edge, GT.transitionsTo, target))
                graph.add((edge, BRIDGE.position, Literal(column)))
                if chance:
                    graph.add((edge, GT.probability, Literal(1.0 / width)))
                else:
                    action = ns[f"action:{column}"]
                    graph.add((action, RDF.type, GT.Action))
                    graph.add((node, GT.hasAvailableAction, action))
                    graph.add((edge, BRIDGE.action, action))
    return graph


def measure_static(asserted, expected, executable, tool):
    start = perf_counter_ns()
    request, _ = project(parse_graph(asserted, "nt"), digest(asserted))
    # Match the identifiers used by the retained static oracle.
    for key in ("instance", "run"):
        request[key] += "-shared"
    if tool == "paignia":
        wire, invocation = encode(request), [executable]
    else:
        wire = emit_efg(request, [str(x) for x in SEATS]).encode()
        invocation = [sys.executable, "-c", OPEN_SPIEL_PROGRAM]
    conversion = perf_counter_ns() - start
    if wire != expected:
        raise ValueError(f"{tool}: static translation differs from checked input")
    output, elapsed, rss = timed_process(invocation, wire)
    return output, {
        "tool": tool, "conversion_ns": conversion, "solve_process_ns": elapsed,
        "total_ns": conversion + elapsed, "peak_rss_bytes": rss,
    }


def static_study(executable, destination, seed, repeats):
    shuffle = Random(seed)
    cases = {}
    for name, width, depth, shared in WORKLOADS:
        fixture = layered_graph(name, width, depth, shared)
        oracle = static_case(executable, destination, name, fixture, PAYOFFS)
        case = destination / name
        asserted = (case / "asserted.nt").read_bytes()
        native_response = (case / "paignia.json").read_bytes()
        expected = {
            "paignia": (case / "paignia-request.json").read_bytes(),
            "open_spiel": (case / "game.efg").read_bytes(),
        }
        reference = {"root_values": list(PAYOFFS), "decisions": oracle["tree_states"]["decision"],
                     "every_choice_first": True}
        (case / "open-spiel-backward-reference.json").write_bytes(encode(reference))
        samples = []
        for trial in range(repeats + 1):
            order = list(expected)
            shuffle.shuffle(order)
            for tool in order:
                output, row = measure_static(asserted, expected[tool], executable, tool)
                matches = output == native_response if tool == "paignia" else json.loads(output) == reference
                if not matches:
                    raise ValueError(f"{name} {tool}: backward induction differs from the checked outcome")
                samples.append({"trial": trial, **row})
        cases[name] = {
            "depth": depth, "branching": width, "shared": shared, "correctness": oracle, "samples": samples,
            "repeated": summarise(samples, ("conversion_ns", "solve_process_ns", "total_ns", "peak_rss_bytes")),
        }
    return cases
