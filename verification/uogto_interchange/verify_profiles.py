"""Replay RDF through the native perfect-recall and two-player Nash profiles."""

import argparse
from copy import deepcopy
import json
from pathlib import Path
import sys
from tempfile import TemporaryDirectory

from rdflib import Graph, Literal, RDF, URIRef
from rdflib.namespace import PROV

sys.path.insert(0, str(Path(__file__).resolve().parent))
from interchange import Bundle, FORMATS, digest
from profile_scenarios import KUHN, check_kuhn, kuhn_graph, normal_graph
from verify import (
    BRIDGE, GT, ROOT, build_solver, encode, exactly_one, iri, number, project,
    require, run_solver, source_state, typed,
)

NORMAL_FORM = "paignia/finite-two-player-normal-form/1.0.0"


def asserted_bytes(graph):
    lines = graph.serialize(format="nt", encoding="utf-8").split(b"\n")
    return b"".join(sorted(line + b"\n" for line in lines if line))


def bindings(specification, representation, source, bundle, profile, method):
    spec = iri(specification)
    return {
        "version": "1.0.0", "semantic_profile": profile,
        "specification": spec, "instance": f"{spec}:instance", "run": f"{spec}:run:{method}",
        "source": {
            "ontology_revision": bundle.revision, "rdf_sha256": digest(source),
            "representation": iri(representation),
        },
    }


def perfect_recall_request(graph, source, bundle, method, iterations, projector):
    specification = exactly_one(graph.subjects(RDF.type, GT.GameSpecification), "game")
    partition = projector(graph, specification)
    representation = URIRef(partition["representation_iri"])
    topology, _ = project(graph, digest(source), require_singleton=False, representation=representation)
    require(
        partition["specification_iri"] == topology["specification"]
        and partition["representation_iri"] == topology["source"]["representation"]
        and partition["root_iri"] == topology["root"]
        and set(partition["state_types"]) == {node["id"] for node in topology["nodes"]},
        "UOGTO partition and projected topology use different identities",
    )
    declaration = partition["declaration"]
    request = bindings(specification, representation, source, bundle, declaration["semantic_profile"], method)
    request["game"] = {
        "kind": "perfect_recall", "root": topology["root"], "nodes": topology["nodes"],
        "declaration": declaration, "method": method, "iterations": iterations,
    }
    return request


def normal_request(graph, source, bundle, method):
    specification = exactly_one(graph.subjects(RDF.type, GT.GameSpecification), "game")
    representation = exactly_one(
        (candidate for candidate in graph.objects(specification, GT.hasRepresentation)
         if (candidate, RDF.type, GT.NormalFormRepresentation) in graph),
        "normal-form representation",
    )
    iri(representation)
    players = sorted(typed(graph, player, GT.Player) for player in graph.objects(specification, GT.hasPlayer))
    require(len(players) == 2, "The normal-form profile requires two players")
    strategies = [
        sorted(typed(graph, action, GT.Strategy) for action in graph.objects(player, GT.hasStrategy))
        for player in players
    ]
    require(all(1 <= len(items) <= 8 for items in strategies), "Expected one to eight strategies per player")
    require(not (set(strategies[0]) & set(strategies[1])), "Normal-form strategies have ambiguous ownership")

    matrix = {}
    for profile in graph.objects(representation, BRIDGE.payoffCell):
        typed(graph, profile, GT.StrategyProfile)
        components = set(graph.objects(profile, GT.comprisesStrategy))
        declared = set(graph.objects(profile, GT.hasStrategy))
        require(not components or not declared or components == declared, f"Conflicting strategies for {profile}")
        actions = components or declared
        require(len(actions) == 2, f"Expected two strategies for {profile}")
        pair = tuple(exactly_one(actions.intersection(seat), f"one owned strategy for {profile}") for seat in strategies)
        require(pair not in matrix, f"Duplicate normal-form payoff cell: {profile}")
        payoff_profile = typed(graph, exactly_one(graph.objects(profile, GT.hasPayoff), "payoff profile"), GT.PayoffProfile)
        values = {}
        for link in graph.objects(payoff_profile, GT.hasPayoffForPlayer):
            typed(graph, link, GT.PlayerPayoffLink)
            player = exactly_one(graph.objects(link, GT.payoffPlayer), "payoff recipient")
            require(player in players and player not in values, f"Foreign or repeated payoff recipient at {link}")
            values[player] = {
                "id": iri(link), "player": iri(player),
                "value": number(exactly_one(graph.objects(link, GT.payoffValue), "payoff value")),
            }
        require(set(values) == set(players), f"Incomplete payoffs at {profile}")
        matrix[pair] = {"profile": iri(profile), "payoff_profile": iri(payoff_profile),
                        "payoffs": [values[player] for player in players]}
    require(len(matrix) == len(strategies[0]) * len(strategies[1]), "Incomplete normal-form payoff matrix")
    request = bindings(specification, representation, source, bundle, NORMAL_FORM, method)
    request["game"] = {
        "kind": "normal_form", "method": method,
        "players": [{"id": iri(player), "strategies": list(map(iri, ids))}
                    for player, ids in zip(players, strategies, strict=True)],
        "payoffs": [[matrix[(row, column)] for column in strategies[1]] for row in strategies[0]],
    }
    return request


def add_policy(graph, solution, actor, actor_role, choices, tag):
    for index, (action, probability) in enumerate(choices):
        record = URIRef(f"{solution}:choice:{tag}:{index}")
        graph.add((solution, BRIDGE.choice, record))
        graph.add((record, actor_role, URIRef(actor)))
        graph.add((record, BRIDGE.action, URIRef(action)))
        graph.add((record, BRIDGE.position, Literal(index)))
        graph.add((record, GT.probability, Literal(probability)))


def add_expected_payoffs(graph, target, payoffs, source):
    for index, (player, payoff) in enumerate(sorted(payoffs.items())):
        link, player_iri = URIRef(f"{target}:payoff:{index}"), URIRef(player)
        graph.add((target, BRIDGE.expectedPayoff, link))
        graph.add((link, RDF.type, GT.PlayerPayoffLink))
        graph.add((link, GT.payoffPlayer, player_iri))
        graph.add((link, GT.payoffValue, Literal(payoff)))
        for identifier in source.objects(player_iri, GT.identifier):
            graph.add((player_iri, GT.identifier, identifier))


def result_graph(response, source):
    graph = Graph()
    run = URIRef(response["run"])
    solution = URIRef(f"{run}:result")
    graph.add((solution, RDF.type, GT.GameSolution))
    graph.add((solution, PROV.wasGeneratedBy, run))
    graph.add((solution, PROV.wasDerivedFrom, URIRef(response["specification"])))
    graph.add((run, RDF.type, PROV.Activity))
    graph.add((run, PROV.used, URIRef(response["instance"])))
    graph.add((run, PROV.used, URIRef(response["source"]["representation"])))
    for key in ("method", "method_version", "status", "semantic_profile"):
        graph.add((run, BRIDGE[key], Literal(response[key])))
    for key in ("ontology_revision", "rdf_sha256"):
        graph.add((run, BRIDGE[key], Literal(response["source"][key])))
    result = response["result"]
    for row in result.get("source_cells", ()):
        for cell in row:
            for identity in (cell["profile"], cell["payoff_profile"], *(link["id"] for link in cell["payoffs"])):
                graph.add((run, PROV.used, URIRef(identity)))
    if "policy" in result:
        for index, (actor, choices) in enumerate(sorted(result["policy"].items())):
            add_policy(graph, solution, actor, BRIDGE.informationSet, choices, index)
        add_expected_payoffs(graph, solution, result["expected_payoffs"], source)
        graph.add((run, BRIDGE.nashConv, Literal(result["nash_conv"])))
        graph.add((run, BRIDGE.iterations, Literal(result["iterations"])))
    elif "profiles" in result:
        if result["profiles"]:
            graph.add((solution, RDF.type, GT.Equilibrium))
        for index, profile in enumerate(result["profiles"]):
            member = URIRef(f"{solution}:joint:{index}")
            graph.add((solution, GT.hasStrategyProfile, member))
            graph.add((member, RDF.type, GT.StrategyProfile))
            for seat, (player, action) in enumerate(sorted(profile.items())):
                graph.add((member, GT.hasStrategy, URIRef(action)))
                graph.add((member, GT.comprisesStrategy, URIRef(action)))
                add_policy(graph, member, player, BRIDGE.player, [(action, 1.0)], seat)
    else:
        for index, candidate in enumerate(result["candidates"]):
            member = URIRef(f"{solution}:candidate:{index}")
            graph.add((solution, BRIDGE.candidate, member))
            for seat, (actor, choices) in enumerate(sorted(candidate["strategy"].items())):
                add_policy(graph, member, actor, BRIDGE.player, choices, seat)
            add_expected_payoffs(graph, member, candidate["expected_payoffs"], source)
            graph.add((member, BRIDGE.maxGain, Literal(candidate["max_gain"])))
    return graph


def check_normal(response, request, method):
    result = response["result"]
    players = request["game"]["players"]
    require(result["source_cells"] == request["game"]["payoffs"], "Normal-form payoff provenance changed")
    if method == "pure":
        expected = {seat["id"]: exactly_one(
            (action for action in seat["strategies"] if action.endswith(":defect")), "defect strategy")
            for seat in players}
        require(result["profiles"] == [expected], "Prisoner's Dilemma must yield mutual defection")
        require(response["method"] == "paignia/pure-nash" and response["status"] == "completed",
                "Incorrect pure Nash method or status")
    else:
        require(response["method"] == "paignia/equal-support-enumeration"
                and response["status"] == "completed_within_scope", "Incorrect mixed Nash method or status")
        candidate = exactly_one(result["candidates"], "matching pennies witness")
        require(candidate["strategy"] == {
            seat["id"]: [[action, 0.5] for action in seat["strategies"]] for seat in players
        }, "Matching pennies probabilities or strategy identities changed")
        require(candidate["expected_payoffs"] == {seat["id"]: 0.0 for seat in players},
                "Matching pennies value must be zero for each player")
        require(candidate["max_gain"] == 0.0, "Matching pennies witness admits a profitable deviation")


def check_result_provenance(response, graph):
    result = response["result"]
    run = URIRef(response["run"])
    referenced = set(graph.objects(run, PROV.used))
    for row in result.get("source_cells", ()):
        for cell in row:
            identities = [cell["profile"], cell["payoff_profile"], *(link["id"] for link in cell["payoffs"])]
            require(all(URIRef(identity) in referenced for identity in identities),
                    "Normal-form result lost source payoff records")
    if "profiles" in result:
        solution = URIRef(f"{run}:result")
        mapped = []
        for member in graph.objects(solution, GT.hasStrategyProfile):
            choices = {}
            for record in graph.objects(member, BRIDGE.choice):
                player = iri(exactly_one(graph.objects(record, BRIDGE.player), "result player"))
                require(player not in choices, "Duplicated player in the pure RDF policy")
                choices[player] = iri(exactly_one(graph.objects(record, BRIDGE.action), "result strategy"))
            mapped.append(choices)
        require(sorted(map(encode, mapped)) == sorted(map(encode, result["profiles"])),
                "Pure Nash RDF lost player-to-strategy bindings")


def check_rejections(solver, perfect_recall, normal):
    incomplete = deepcopy(perfect_recall)
    incomplete["game"]["declaration"]["sets"].pop()
    reordered = deepcopy(perfect_recall)
    member = reordered["game"]["declaration"]["sets"][0]["members"][0]
    node = next(record["node"] for record in reordered["game"]["nodes"] if record["id"] == member)
    node["actions"].reverse()
    incomplete_table = deepcopy(normal)
    incomplete_table["game"]["payoffs"][0].pop()
    wrong_recipient = deepcopy(normal)
    wrong_recipient["game"]["payoffs"][0][0]["payoffs"].reverse()
    wrong_profile = dict(normal, semantic_profile=perfect_recall["semantic_profile"])
    wrong_recall_version = dict(perfect_recall, version="2.0.0")
    wrong_normal_version = dict(normal, version="2.0.0")
    for request, error in (
        (wrong_recall_version, "Version"), (wrong_normal_version, "Version"),
        (incomplete, "UnassignedDecisionNode"), (reordered, "ActionMismatch"),
        (incomplete_table, "PayoffTable"), (wrong_recipient, "PayoffTable"), (wrong_profile, "Profile"),
    ):
        result = run_solver(solver, encode(request))
        require(result.returncode and not result.stdout and error in result.stderr.decode(),
                f"Native profile did not reject {error}")
    return ["UnassignedDecisionNode", "ActionMismatch", "PayoffTable", "PayoffRecipientOrder", "Profile",
            "PerfectRecallVersion", "NormalFormVersion"]


def replay(bundle, solver, destination):
    destination.mkdir(parents=True, exist_ok=True)
    reference = json.loads((ROOT / "fixtures/conformance/cfr-reference-v1.json").read_bytes())["kuhn"]
    outputs = {}
    requests = {}
    with TemporaryDirectory(prefix="paignia-uogto-projector-") as temp:
        for path, content in bundle.files.items():
            if path.startswith("uogto/"):
                target = Path(temp) / path
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(content)
        sys.path.insert(0, temp)
        try:
            from uogto.interchange.information_sets import project_information_sets

            cases = [
                ("kuhn-cfr", kuhn_graph(), "cfr"),
                ("kuhn-cfr-plus", kuhn_graph(), "cfr_plus"),
                ("prisoners-dilemma", normal_graph("prisoners-dilemma", ("cooperate", "defect"),
                                                   [(3, 3), (0, 5), (5, 0), (1, 1)]), "pure"),
                ("matching-pennies", normal_graph("matching-pennies", ("heads", "tails"),
                                                 [(1, -1), (-1, 1), (-1, 1), (1, -1)]), "mixed"),
            ]
            for name, example, method in cases:
                source = asserted_bytes(example)
                _, graph = bundle.roundtrip(source, FORMATS[".nt"])
                if method in {"cfr", "cfr_plus"}:
                    request = perfect_recall_request(graph, source, bundle, method, reference["iteration_counts"][-1],
                                                     project_information_sets)
                else:
                    request = normal_request(graph, source, bundle, method)
                requests[name] = request
                wire = encode(request)
                run = run_solver(solver, wire)
                require(run.returncode == 0, run.stderr.decode())
                response = json.loads(run.stdout)
                for field in ("version", "semantic_profile", "specification", "source", "instance", "run"):
                    require(response[field] == request[field], f"{name} changed {field}")
                if method in {"cfr", "cfr_plus"}:
                    require(response["status"] == "training_completed", "CFR training status changed")
                    require(response["method"] == ("paignia/vanilla-cfr" if method == "cfr" else "paignia/cfr-plus"),
                            "CFR variant changed")
                    require(response["result"]["iterations"] == reference["iteration_counts"][-1],
                            "CFR iteration budget changed")
                    check_kuhn(response["result"], reference)
                else:
                    check_normal(response, request, method)
                result, restored = bundle.roundtrip(asserted_bytes(result_graph(response, graph)), FORMATS[".nt"])
                check_result_provenance(response, restored)
                files = {"game.nt": source, "request.json": wire, "result.json": run.stdout, "result.nt": result}
                directory = destination / name
                directory.mkdir(exist_ok=True)
                for path, data in files.items():
                    (directory / path).write_bytes(data)
                outputs[name] = {"method": response["method"], "method_version": response["method_version"],
                                 "status": response["status"], "files": {key: digest(data) for key, data in files.items()}}
        finally:
            sys.path.pop(0)
    rejections = check_rejections(solver, requests["kuhn-cfr"], requests["matching-pennies"])
    return {"outputs": outputs, "rejections": rejections}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--ontology-dir", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    revision = source_state()
    bundle = Bundle(args.ontology_dir)
    solver = build_solver("solve_profile")
    solver_hash = digest(solver.read_bytes())
    evidence = replay(bundle, solver, args.output)
    require(source_state() == revision and digest(solver.read_bytes()) == solver_hash,
            "Source or solver changed during executable profile replay")
    evidence.update({
        "provider_revision": revision, "solver_sha256": solver_hash,
        "ontology_revision": bundle.revision, "ontology_lock_sha256": bundle.lock_sha256,
    })
    (args.output / "receipt.json").write_bytes(encode(evidence))
    print("CFR, CFR+, pure Nash and mixed Nash profile replays passed")


if __name__ == "__main__":
    main()
