"""Run the frozen T21 Gambit and OpenSpiel correctness comparison."""

import argparse
import csv
from fractions import Fraction
from itertools import permutations, product
import json
import math
from pathlib import Path
import platform
import subprocess
import sys
from tempfile import TemporaryDirectory

import pyspiel
from rdflib import Graph, Literal, RDF
from rdflib.namespace import XSD

HERE = Path(__file__).resolve().parent
INTERCHANGE = HERE.parent / "uogto_interchange"
sys.path.insert(0, str(INTERCHANGE))

from interchange import Bundle, digest, parse_graph
from profile_scenarios import CARDS, KUHN, kuhn_graph, normal_graph
from verify import BRIDGE, CASE, GT, ROOT, encode, project, require, source_state
from verify_profiles import asserted_bytes, normal_request, perfect_recall_request

RELEASE = "a04a19da06cc50e310a172574ed66f62000b891a"
ERROR_LIMIT = 1e-9
TIE_TOLERANCE = 1e-12


def command(program, inputs, output, *arguments):
    run = subprocess.run(
        [program, *arguments], input=inputs, capture_output=True, timeout=30, check=False,
    )
    output.write_bytes(run.stdout)
    if run.stderr:
        output.with_suffix(output.suffix + ".stderr").write_bytes(run.stderr)
    require(run.returncode == 0, f"{program.name} failed for {output.parent.name}")
    return run.stdout


def native(program, request, directory, basename="paignia"):
    (directory / f"{basename}-request.json").write_bytes(encode(request))
    response = json.loads(command(program, encode(request), directory / f"{basename}.json"))
    for field in ("version", "semantic_profile", "source", "specification", "instance", "run"):
        require(response[field] == request[field], f"{directory.name} changed {field}")
    return response


def close(actual, expected, description, tolerance=ERROR_LIMIT):
    require(math.isclose(actual, expected, abs_tol=tolerance, rel_tol=0.0),
            f"{description}: {actual} differs from {expected}")


def gambit_case(bundle, executables, evidence, name, matrix, method):
    directory = evidence / name
    directory.mkdir()
    row_actions = tuple(f"r{i}" for i in range(len(matrix)))
    column_actions = tuple(f"c{j}" for j in range(len(matrix[0])))
    graph = normal_graph(name, row_actions, [cell for row in matrix for cell in row],
                         column_actions=column_actions)
    asserted = asserted_bytes(graph)
    (directory / "asserted.nt").write_bytes(asserted)
    request = normal_request(parse_graph(asserted, "nt"), asserted, bundle, method)

    # The RDF projector orders the column-labelled seat first. Check utility
    # identities before producing the NFG, whose first seat varies fastest.
    seats = request["game"]["players"]
    actions = [seat["strategies"] for seat in seats]
    projected = request["game"]["payoffs"]
    by_label = {
        (column, row): matrix[i][j][::-1]
        for i, row in enumerate(row_actions)
        for j, column in enumerate(column_actions)
    }
    nfg_values = []
    for j, second in enumerate(actions[1]):
        for i, first in enumerate(actions[0]):
            expected = by_label[first.rsplit(":", 1)[-1], second.rsplit(":", 1)[-1]]
            actual = tuple(record["value"] for record in projected[i][j]["payoffs"])
            require(actual == expected, f"{name} changed a labelled payoff")
            nfg_values.extend(map(str, expected))
    nfg = (
        f'NFG 1 R "{name}" {{ "column" "row" }} {{ {len(actions[0])} {len(actions[1])} }}\n\n'
        + " ".join(nfg_values) + "\n"
    ).encode()
    (directory / "game.nfg").write_bytes(nfg)
    gambit_output = command(executables[method], nfg, directory / "gambit.csv", "-q")
    expected = set()
    for record in csv.reader(gambit_output.decode().splitlines()):
        require(record[0] == "NE" and len(record) == 1 + sum(map(len, actions)),
                f"{name}: unexpected Gambit record {record}")
        expected.add(tuple(map(Fraction, record[1:])))

    response = native(executables["profile"], request, directory)
    result = response["result"]
    if method == "pure":
        require(response["status"] == "completed", f"{name}: incomplete pure enumeration")
        observed = [
            tuple(int(profile[seat["id"]] == action) for seat in seats for action in seat["strategies"])
            for profile in result["profiles"]
        ]
        require(len(observed) == len(expected) and set(observed) == expected,
                f"{name}: pure equilibrium sets differ")
        return len(expected)

    require(response["status"] in {"completed_within_scope", "unresolved_supports"},
            f"{name}: incomplete mixed enumeration")
    if name == "all-zero":
        require(response["status"] == "unresolved_supports" and len(expected) == 4,
                "The all-zero game requires four vertices and unresolved supports")
    require(len(result["candidates"]) == len(expected), f"{name}: mixed candidate counts differ")
    unmatched = expected.copy()
    max_error = 0.0
    max_payoff_error = 0.0
    max_gain = 0.0
    for candidate in result["candidates"]:
        probabilities = []
        for seat in seats:
            policy = candidate["strategy"][seat["id"]]
            require([action for action, _ in policy] == seat["strategies"], f"{name}: strategy order changed")
            weights = [probability for _, probability in policy]
            require(all(math.isfinite(value) and value >= 0 for value in weights),
                    f"{name}: invalid mixed probability")
            close(math.fsum(weights), 1, f"{name}: probability mass")
            probabilities.extend(weights)
        distances = {
            witness: max(abs(actual - exact) for actual, exact in zip(probabilities, witness, strict=True))
            for witness in unmatched
        }
        match = min(distances, key=distances.get)
        close(distances[match], 0, f"{name}: probability error")
        max_error = max(max_error, distances[match])
        unmatched.remove(match)
        first, second = match[:len(actions[0])], match[len(actions[0]):]
        gains = []
        for seat_index, (own, opponent), seat in zip(
            (0, 1), ((first, second), (second, first)), seats, strict=True,
        ):
            utilities = []
            for i in range(len(own)):
                values = (projected[i][j] if seat_index == 0 else projected[j][i] for j in range(len(opponent)))
                utilities.append(sum(weight * Fraction(cell["payoffs"][seat_index]["value"])
                                     for weight, cell in zip(opponent, values, strict=True)))
            utility = sum(weight * value for weight, value in zip(own, utilities, strict=True))
            gains.append(max(utilities) - utility)
            observed_payoff = candidate["expected_payoffs"][seat["id"]]
            close(observed_payoff, utility, f"{name}: expected payoff")
            max_payoff_error = max(max_payoff_error, abs(observed_payoff - utility))
        close(max(gains), 0, f"{name}: Gambit witness gain")
        close(candidate["max_gain"], max(gains), f"{name}: unilateral gain")
        max_gain = max(max_gain, candidate["max_gain"])
    return {
        "witnesses": len(expected), "status": response["status"],
        "unresolved_support_pairs": result["unresolved_support_pairs"],
        "max_probability_error": max_error,
        "max_payoff_error": max_payoff_error, "max_native_gain": max_gain,
    }


def compare_normal(bundle, executables, evidence):
    pure_counts = []
    for index, flat in enumerate(product((0, 1), repeat=8)):
        a, b, c, d = list(zip(flat[::2], flat[1::2], strict=True))
        pure_counts.append(gambit_case(bundle, executables, evidence, f"binary-{index:03d}", ((a, b), (c, d)), "pure"))
    cases = {
        "prisoners-dilemma": (((3, 3), (0, 5)), ((5, 0), (1, 1))),
        "matching-pennies": (((1, -1), (-1, 1)), ((-1, 1), (1, -1))),
        "asymmetric-coordination": (((4, 3), (0, 1)), ((2, 0), (3, 4))),
        "rock-paper-scissors": (
            ((0, 0), (-1, 1), (1, -1)),
            ((1, -1), (0, 0), (-1, 1)),
            ((-1, 1), (1, -1), (0, 0)),
        ),
        "rectangular": (((4, 3), (0, 1), (1, -3)), ((2, 0), (3, 4), (2, -3))),
        "all-zero": (((0, 0), (0, 0)), ((0, 0), (0, 0))),
    }
    return {
        "pure_games": len(pure_counts), "pure_witnesses": sum(pure_counts),
        "games_without_pure_witness": pure_counts.count(0),
        "mixed": {
            name: gambit_case(bundle, executables, evidence, name, matrix, "mixed")
            for name, matrix in cases.items()
        },
    }


def emit_efg(request, seats):
    nodes = {record["id"]: record["node"] for record in request["nodes"]}
    lines = ['EFG 2 R "Finite Paignia fixture" { "buyer" "seller" }']

    def emit(node_id):
        node = nodes[node_id]
        serial = len(lines)
        if node["kind"] == "terminal":
            payoffs = {row["player"]: row["value"] for row in node["payoffs"]}
            values = ", ".join(str(payoffs[player]) for player in seats)
            lines.append(f't "" {serial} "" {{ {values} }}')
        elif node["kind"] == "chance":
            edges = node["outcomes"]
            options = " ".join(f'"edge{i}" {Fraction(edge["probability"])}' for i, edge in enumerate(edges))
            lines.append(f'c "" {serial} "" {{ {options} }} 0')
            for edge in edges:
                emit(edge["target"])
        else:
            edges = node["actions"]
            options = " ".join(f'"edge{i}"' for i in range(len(edges)))
            lines.append(f'p "" {seats.index(node["player"]) + 1} {serial} "" {{ {options} }} 0')
            for edge in edges:
                emit(edge["target"])

    emit(request["root"])
    return "\n".join(lines) + "\n"


def static_case(executable, evidence, name, graph, expected_payoffs):
    directory = evidence / name
    directory.mkdir()
    asserted = asserted_bytes(graph)
    (directory / "asserted.nt").write_bytes(asserted)
    request, _ = project(parse_graph(asserted, "nt"), digest(asserted))
    if name != "buyer-seller":
        for key in ("instance", "run"):
            request[key] += "-shared"
    response = native(executable, request, directory)
    require(response["method"] == "backward_induction" and response["status"] == "completed"
            and response["tolerance"] == TIE_TOLERANCE, f"{name}: backward induction contract changed")
    seats = [str(CASE.acheteur), str(CASE["décideur"])]
    efg = emit_efg(request, seats)
    (directory / "game.efg").write_text(efg, encoding="utf-8")
    game = pyspiel.load_efg_game(efg)
    root = game.new_initial_state()
    nodes = {record["id"]: record["node"] for record in request["nodes"]}
    policy_table, decisions = {}, []
    reached, branch_counts = set(), {"terminal": 0, "chance": 0, "decision": 0}

    def walk(state, node_id):
        reached.add(node_id)
        node = nodes[node_id]
        branch_counts[node["kind"]] += 1
        if node["kind"] == "terminal":
            values = {item["player"]: item["value"] for item in node["payoffs"]}
            require(state.is_terminal() and state.returns() == [values[seat] for seat in seats],
                    f"{name}: terminal payoff changed in translation")
            return
        chance = node["kind"] == "chance"
        require(state.is_chance_node() == chance and not state.is_terminal(), f"{name}: node type changed")
        edges = node["outcomes" if chance else "actions"]
        actions = {state.action_to_string(action): action for action in state.legal_actions()}
        require(set(actions) == {f"edge{i}" for i in range(len(edges))}, f"{name}: actions changed")
        ids = [actions[f"edge{i}"] for i in range(len(edges))]
        if chance:
            probabilities = dict(state.chance_outcomes())
            for action, edge in zip(ids, edges, strict=True):
                close(probabilities[action], edge["probability"], f"{name}: chance probability")
        else:
            require(seats[state.current_player()] == node["player"], f"{name}: seat order changed")
            candidates = {edge["action"]: action for edge, action in zip(edges, ids, strict=True)}
            choice = response["choices"][node_id]
            policy_table[state.information_state_string()] = [
                (action, float(candidate == choice)) for candidate, action in candidates.items()
            ]
            require(choice in candidates, f"{name}: Paignia selected an unavailable action")
            decisions.append((node_id, state, candidates, choice))
        for action, edge in zip(ids, edges, strict=True):
            walk(state.child(action), edge["target"])

    walk(root, request["root"])
    require(reached == set(nodes), f"{name}: a native node was omitted")
    require(set(response["choices"]) == {item[0] for item in decisions}, f"{name}: incomplete policy")
    policy = pyspiel.TabularPolicy(policy_table)
    values = pyspiel.expected_returns(root, policy, -1, True)
    for player, value, expected in zip(seats, values, expected_payoffs, strict=True):
        close(response["expected_payoffs"][player], value, f"{name}: OpenSpiel root payoff")
        close(expected, value, f"{name}: hand root payoff")
    proof = []
    for node_id, state, candidates, choice in decisions:
        continuations = {label: pyspiel.expected_returns(state.child(action), policy, -1, True)
                         for label, action in candidates.items()}
        owner = state.current_player()
        best = max(value[owner] for value in continuations.values())
        winning = sorted(label for label in continuations if best - continuations[label][owner] <= TIE_TOLERANCE)
        require(choice == winning[0], f"{name}: wrong subgame action at {node_id}")
        proof.append({"node": node_id, "choice": choice, "continuations": continuations, "optimal": winning})
    proof_record = {"root_values": values, "tree_states": branch_counts, "decision_evidence": proof}
    (directory / "open-spiel.json").write_bytes(encode(proof_record))
    return {"root_values": values, "tree_states": branch_counts, "native_nodes": len(nodes)}


def compare_static(executable, evidence):
    graph = Graph().parse(INTERCHANGE / "game.ttl")
    for_stress = Graph() + graph
    for_stress.set((CASE.ad, GT.payoffValue, Literal(3.0, datatype=XSD.double)))
    for_stress.set((CASE.sa, GT.payoffValue, Literal(3.0, datatype=XSD.double)))
    zero = CASE["zero-edge"]
    for fact in (
        (CASE.draw, GT.hasTransition, zero),
        (zero, RDF.type, GT.Transition),
        (zero, GT.transitionsFrom, CASE.draw),
        (zero, GT.transitionsTo, CASE.respond),
        (zero, BRIDGE.position, Literal(2)),
        (zero, GT.probability, Literal(0.0, datatype=XSD.double)),
    ):
        for_stress.add(fact)
    return {
        "buyer-seller": static_case(executable, evidence, "buyer-seller", graph, (2, 4)),
        "buyer-seller-shared": static_case(executable, evidence, "buyer-seller-shared", for_stress, (3, 0)),
    }


def kuhn_mapping(request, game):
    native_game = request["game"]
    nodes = {row["id"]: row["node"] for row in native_game["nodes"]}
    owners = [str(KUHN.p1), str(KUHN.p2)]
    member_to_set = {
        member: item["id"] for item in native_game["declaration"]["sets"] for member in item["members"]
    }
    mapping, visited = {}, {native_game["root"]}

    def visit(state, node_id):
        visited.add(node_id)
        node = nodes[node_id]
        if state.is_terminal():
            expected = {item["player"]: item["value"] for item in node["payoffs"]}
            require(node["kind"] == "terminal" and state.returns() == [expected[seat] for seat in owners],
                    "Kuhn terminal payoff mismatch")
            return
        require(node["kind"] == "decision" and node["player"] == owners[state.current_player()],
                "Kuhn acting seat mismatch")
        actions = state.legal_actions()
        require([state.action_to_string(action) for action in actions] == ["Pass", "Bet"],
                "OpenSpiel Kuhn action order changed")
        edges = node["actions"]
        labels = [edge["action"] for edge in edges]
        names = [label.rsplit(":", 1)[-1] for label in labels]
        expected = ["fold", "call"] if state.history()[-1] == 1 and len(state.history()) > 2 else ["check", "bet"]
        require(names == expected, f"Kuhn action mismatch: {names}")
        item = state.information_state_string(), dict(zip(labels, actions, strict=True))
        set_id = member_to_set[node_id]
        require(mapping.setdefault(set_id, item) == item, "Kuhn information set split")
        for action, edge in zip(actions, edges, strict=True):
            visit(state.child(action), edge["target"])

    root = game.new_initial_state()
    deal = nodes[native_game["root"]]
    native_deals = {item["target"].split(":")[-2]: item for item in deal["outcomes"]}
    require(set(native_deals) == {"".join(pair) for pair in permutations(CARDS, 2)}, "Kuhn deals changed")
    for first, first_weight in root.chance_outcomes():
        child = root.child(first)
        for second, second_weight in child.chance_outcomes():
            edge = native_deals[CARDS[first] + CARDS[second]]
            close(first_weight * second_weight, edge["probability"], "Kuhn deal probability")
            visit(child.child(second), edge["target"])
    require(visited == set(nodes), "Kuhn tree traversal omitted a native state")
    require(set(mapping) == set(member_to_set.values()) and len({entry[0] for entry in mapping.values()}) == 12,
            "Kuhn declaration differs from OpenSpiel's 12 information sets")
    return mapping


def evaluate_kuhn(game, policy):
    return {
        "values": pyspiel.expected_returns(game.new_initial_state(), policy, -1, True),
        "nash_conv": pyspiel.nash_conv(game, policy),
    }


def compare_kuhn(executable, bundle, evidence):
    asserted = asserted_bytes(kuhn_graph())
    graph = parse_graph(asserted, "nt")
    reference = json.loads((ROOT / "fixtures/conformance/cfr-reference-v1.json").read_bytes())["kuhn"]
    require(reference["iteration_counts"] == [100, 1000, 12000], "Kuhn budgets changed")
    game = pyspiel.load_game("kuhn_poker")
    all_results = {}
    with TemporaryDirectory(prefix="paignia-t21-uogto-") as temporary:
        for name, content in bundle.files.items():
            if name.startswith("uogto/"):
                destination = Path(temporary) / name
                destination.parent.mkdir(parents=True, exist_ok=True)
                destination.write_bytes(content)
        sys.path.insert(0, temporary)
        try:
            from uogto.interchange.information_sets import project_information_sets
        finally:
            sys.path.pop(0)

        for method, trainer in (("cfr", pyspiel.CFRSolver), ("cfr_plus", pyspiel.CFRPlusSolver)):
            solver = trainer(game)
            directory = evidence / f"kuhn-{method}"
            directory.mkdir()
            (directory / "asserted.nt").write_bytes(asserted)
            rows, previous = [], 0
            last_conv = {"paignia": math.inf, "open_spiel": math.inf}
            for budget in reference["iteration_counts"]:
                request = perfect_recall_request(graph, asserted, bundle, method, budget, project_information_sets)
                response = native(executable, request, directory, f"paignia-{budget}")
                require(response["status"] == "training_completed" and response["result"]["iterations"] == budget,
                        f"{method}: Paignia did not complete the declared iterations")
                mapping = kuhn_mapping(request, game)
                result = response["result"]
                require(set(result["policy"]) == set(mapping), "Kuhn Paignia policy is incomplete")
                for set_id, entries in result["policy"].items():
                    labels, weights = zip(*entries, strict=True)
                    require(set(labels) == set(mapping[set_id][1]) and len(labels) == 2,
                            "Kuhn Paignia action inventory differs")
                    require(all(0 <= value <= 1 for value in weights), "Kuhn Paignia has an invalid probability")
                    close(math.fsum(weights), 1, "Kuhn Paignia probability mass")
                imported = {
                    mapping[set_id][0]: [(mapping[set_id][1][action], probability) for action, probability in entries]
                    for set_id, entries in result["policy"].items()
                }
                for _ in range(previous, budget):
                    solver.evaluate_and_update_policy()
                paignia_values = evaluate_kuhn(game, pyspiel.TabularPolicy(imported))
                native_policy = solver.tabular_average_policy()
                open_spiel_values = evaluate_kuhn(game, native_policy)
                for seat, actual in zip((str(KUHN.p1), str(KUHN.p2)), paignia_values["values"], strict=True):
                    close(result["expected_payoffs"][seat], actual, f"{method} {budget}: Paignia policy value")
                close(result["nash_conv"], paignia_values["nash_conv"], f"{method} {budget}: Paignia policy NashConv")
                observations = {"paignia": paignia_values, "open_spiel": open_spiel_values}
                for tool, stats in observations.items():
                    close(stats["values"][0], reference["player_one_value_decimal"],
                          f"{tool} {method} {budget}: Kuhn game value", reference["value_absolute_tolerance"])
                    require(0 <= stats["nash_conv"] < last_conv[tool], f"{tool} {method}: NashConv did not shrink")
                    last_conv[tool] = stats["nash_conv"]
                rows.append({"iterations": budget, **observations})
                (directory / f"open-spiel-{budget}.json").write_bytes(encode({
                    "observations": observations,
                    "information_set_map": mapping,
                    "open_spiel_policy": {state: sorted(policy) for state, policy in native_policy.policy_table().items()},
                }))
                previous = budget
            require(all(value < reference["final_nash_conv_upper_bound"] for value in last_conv.values()),
                    f"{method}: final Kuhn NashConv exceeds the bound")
            all_results[method] = rows
    return all_results


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--paignia-bin", required=True, type=Path)
    parser.add_argument("--gambit-bin", required=True, type=Path)
    parser.add_argument("--ontology-dir", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    revision = source_state()
    subprocess.run(["git", "diff", "--exit-code", RELEASE, revision, "--", "Cargo.toml", "Cargo.lock",
                    "crates/paignia-core", "crates/paignia-solve", "crates/paignia-io"],
                   cwd=ROOT, check=True)
    args.output.mkdir(parents=True)
    bundle = Bundle(args.ontology_dir)
    executables = {
        "pure": args.gambit_bin.resolve() / "gambit-enumpure",
        "mixed": args.gambit_bin.resolve() / "gambit-enummixed",
        "profile": args.paignia_bin.resolve() / "solve_profile",
        "static": args.paignia_bin.resolve() / "solve",
        "pyspiel": Path(pyspiel.__file__).resolve(),
    }
    tool_hashes = {name: digest(path.read_bytes()) for name, path in executables.items()}
    findings = {
        "normal_form": compare_normal(bundle, executables, args.output),
        "perfect_information": compare_static(executables["static"], args.output),
        "kuhn_poker": compare_kuhn(executables["profile"], bundle, args.output),
    }
    manifest = {
        path.relative_to(args.output).as_posix(): digest(path.read_bytes())
        for path in sorted(args.output.rglob("*")) if path.is_file()
    }
    require(source_state() == revision, "source changed during comparison")
    require(tool_hashes == {name: digest(path.read_bytes()) for name, path in executables.items()},
            "a tool changed during comparison")
    receipt = {
        "release_commit": RELEASE, "harness_commit": revision,
        "ontology_commit": bundle.revision, "ontology_lock_sha256": bundle.lock_sha256,
        "python": platform.python_version(), "system": platform.system(), "machine": platform.machine(),
        "tools_sha256": tool_hashes, "findings": findings, "files": manifest,
    }
    (args.output / "receipt.json").write_bytes(encode(receipt))
    print(json.dumps(findings, indent=2))


if __name__ == "__main__":
    main()
