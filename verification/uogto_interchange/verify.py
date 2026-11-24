"""Project one asserted finite UOGTO profile and check its native result."""

import argparse
import copy
import hashlib
import json
import math
import os
from pathlib import Path
import subprocess
import urllib.request
from urllib.parse import urlsplit

from rdflib import Graph, Literal, Namespace, RDF, URIRef, XSD
from rdflib.namespace import OWL, PROV

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
GT = Namespace("https://w3id.org/uogto/core#")
BRIDGE = Namespace("https://github.com/edithatogo/paignia#")
CASE = Namespace("https://example.org/paignia/offer#")
STATIC_PROFILE = "paignia/static-finite-perfect-information-sparse-zero/1.0.0"
ONTOLOGY = "cc0c84201929996580bb4af98774cf2145b224ae"
MODULES = {
    "uogto-core.ttl": "4d752788fcba19d1711d2c619ca4928fa2fb681feb648ed8783c3668f2e80dfc",
    "temporality.ttl": "0d09f73a87b59cf1ac199b72a20e46c41238a4ce92364636418e310da67fd23a",
}
NODE_KIND = {GT.DecisionNode: "decision", GT.ChanceNode: "chance", GT.TerminalNode: "terminal"}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def exactly_one(items, role):
    values = list(items)
    require(len(values) == 1, f"expected one {role}, found {len(values)}")
    return values[0]


def iri(value):
    require(isinstance(value, URIRef) and urlsplit(value).scheme, f"expected named IRI: {value}")
    return str(value)


def typed(graph, resource, role):
    iri(resource)
    require((resource, RDF.type, role) in graph, f"{resource} needs asserted type {role}")
    return resource


def number(value):
    require(
        isinstance(value, Literal) and value.datatype in {XSD.decimal, XSD.double, XSD.integer},
        f"expected numeric literal: {value}",
    )
    result = float(value)
    require(math.isfinite(result), f"nonfinite numeric literal: {value}")
    return result


def position(value):
    require(isinstance(value, Literal) and value.datatype == XSD.integer, f"invalid position: {value}")
    require(type(value.value) is int and value.value >= 0, f"invalid position: {value}")
    return value.value


def transitions(graph, state, states):
    ordered = []
    for edge in graph.objects(state, GT.hasTransition):
        typed(graph, edge, GT.Transition)
        origin = exactly_one(graph.objects(edge, GT.transitionsFrom), "transition origin")
        target = exactly_one(graph.objects(edge, GT.transitionsTo), "transition target")
        require(origin == state and target in states, f"transition outside the declared states: {edge}")
        index = position(exactly_one(graph.objects(edge, BRIDGE.position), "transition position"))
        ordered.append((index, edge, {"id": iri(edge), "target": iri(target)}))
    ordered.sort()
    require([item[0] for item in ordered] == list(range(len(ordered))), f"invalid order at {state}")
    return [(edge, record) for _, edge, record in ordered]


def project(graph, source_hash):
    spec = typed(graph, exactly_one(graph.subjects(RDF.type, GT.GameSpecification), "game"), GT.GameSpecification)
    rep = typed(graph, exactly_one(graph.objects(spec, GT.hasRepresentation), "representation"), GT.ExtensiveFormRepresentation)
    players = {typed(graph, value, GT.Player) for value in graph.objects(spec, GT.hasPlayer)}
    require(players, "missing declared players")
    states = set(graph.objects(spec, GT.hasState))
    root = exactly_one(graph.objects(spec, GT.hasInitialState), "initial state")
    require(root in states, "initial state is outside the declared inventory")
    records = []
    used = {spec, rep}
    for state in sorted(states):
        kind = NODE_KIND[exactly_one(set(graph.objects(state, RDF.type)) & NODE_KIND.keys(), "node kind")]
        node = {"kind": kind}
        edges = transitions(graph, state, states)
        used.update(edge for edge, _ in edges)
        if kind == "decision":
            owner = exactly_one(graph.subjects(GT.controlsNode, state), "decision controller")
            require(owner in players, f"controller outside the declared players: {owner}")
            group = typed(graph, exactly_one(graph.objects(state, GT.belongsToInformationSet), "information set"), GT.InformationSet)
            members = set(graph.subjects(GT.belongsToInformationSet, group)) | set(graph.subjects(GT.hasInformationSet, group))
            alias = set(graph.objects(state, GT.hasInformationSet))
            require(members == {state} and alias <= {group}, f"non-singleton or conflicting information set: {group}")
            actions = []
            for edge, record in edges:
                action = typed(graph, exactly_one(graph.objects(edge, BRIDGE.action), "decision action"), GT.Action)
                require(not list(graph.objects(edge, GT.probability)), "probability on a decision transition")
                actions.append(dict(record, action=iri(action)))
            admitted = {iri(value) for value in graph.objects(state, GT.hasAvailableAction)}
            require(admitted == {action["action"] for action in actions}, "available-action inventory differs from edges")
            node.update(player=iri(owner), actions=actions)
        elif kind == "chance":
            outcomes = []
            for edge, record in edges:
                require(not list(graph.objects(edge, BRIDGE.action)), "player action on a chance transition")
                probability = number(exactly_one(graph.objects(edge, GT.probability), "chance weight"))
                require(0 <= probability <= 1, f"invalid probability on {edge}")
                outcomes.append(dict(record, probability=probability))
            node["outcomes"] = outcomes
        else:
            require(not edges, f"terminal state has outgoing transitions: {state}")
            profile = typed(graph, exactly_one(graph.objects(state, BRIDGE.payoffProfile), "payoff profile"), GT.PayoffProfile)
            used.add(profile)
            values = []
            for link in sorted(graph.objects(profile, GT.hasPayoffForPlayer)):
                typed(graph, link, GT.PlayerPayoffLink)
                player = exactly_one(graph.objects(link, GT.payoffPlayer), "payoff recipient")
                value = number(exactly_one(graph.objects(link, GT.payoffValue), "payoff value"))
                values.append({"id": iri(link), "player": iri(player), "value": value})
                used.add(link)
            require(
                len(values) == len(players) and {value["player"] for value in values} == {iri(player) for player in players},
                f"payoff coverage differs from the declared players at {state}",
            )
            node["payoffs"] = values
        records.append({"id": iri(state), "node": node})
    return {
        "version": "1.0.0",
        "semantic_profile": STATIC_PROFILE,
        "source": {"ontology_revision": ONTOLOGY, "rdf_sha256": source_hash, "representation": iri(rep)},
        "specification": iri(spec),
        "instance": iri(CASE.instance),
        "run": iri(CASE.run),
        "root": iri(root),
        "nodes": records,
    }, used


def load_ontology(directory):
    ontology = Graph()
    for module, digest in MODULES.items():
        if directory is None:
            url = f"https://raw.githubusercontent.com/edithatogo/UOGTO/{ONTOLOGY}/ontologies/core/{module}"
            with urllib.request.urlopen(url, timeout=30) as response:
                data = response.read(1_048_577)
        else:
            data = (directory / module).read_bytes()
        require(sha256(data) == digest, f"ontology identity mismatch for {module}")
        ontology.parse(data=data, format="turtle")
    return ontology


def verify_vocabulary(graph, ontology):
    for subject, predicate, value in graph:
        if predicate == RDF.type and str(value).startswith(str(GT)):
            require((value, RDF.type, OWL.Class) in ontology, f"unknown UOGTO class: {value}")
        if str(predicate).startswith(str(GT)):
            role = OWL.DatatypeProperty if isinstance(value, Literal) else OWL.ObjectProperty
            require((predicate, RDF.type, role) in ontology, f"unknown UOGTO {role}: {predicate} at {subject}")


def result_graph(result, used):
    graph = Graph()
    run = URIRef(result["run"])
    policy = URIRef(f"{run}/policy")
    graph.add((policy, RDF.type, GT.GameSolution))
    graph.add((policy, PROV.wasDerivedFrom, URIRef(result["specification"])))
    graph.add((policy, PROV.wasGeneratedBy, run))
    graph.add((run, RDF.type, PROV.Activity))
    graph.add((run, PROV.used, URIRef(result["instance"])))
    for resource in used:
        graph.add((run, PROV.used, resource))
    for field in ("method", "method_version", "status", "semantic_profile", "native_result_version", "tolerance"):
        graph.add((run, BRIDGE[field], Literal(result[field])))
    for field in ("ontology_revision", "rdf_sha256"):
        graph.add((run, BRIDGE[field], Literal(result["source"][field])))
    for index, (state, action) in enumerate(sorted(result["choices"].items())):
        choice = URIRef(f"{policy}/choice/{index}")
        graph.add((policy, BRIDGE.choice, choice))
        graph.add((choice, BRIDGE.state, URIRef(state)))
        graph.add((choice, GT.selectsAction, URIRef(action)))
    for index, (player, payoff) in enumerate(sorted(result["expected_payoffs"].items())):
        link = URIRef(f"{policy}/payoff/{index}")
        graph.add((policy, BRIDGE.expectedPayoff, link))
        graph.add((link, RDF.type, GT.PlayerPayoffLink))
        graph.add((link, GT.payoffPlayer, URIRef(player)))
        graph.add((link, GT.payoffValue, Literal(payoff, datatype=XSD.double)))
    return graph


def encode(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True, allow_nan=False, indent=2).encode() + b"\n"


def run_solver(solver, payload):
    return subprocess.run([solver], input=payload, capture_output=True, timeout=30, check=False)


def check_rejections(solver, graph, request, exact_bytes):
    rdf_negatives = [
        ((CASE["safe-edge"], BRIDGE.position, Literal(0)), "invalid order"),
        ((CASE.respond, GT.belongsToInformationSet, CASE["start-set"]), "non-singleton"),
    ]
    for triple, expected_error in rdf_negatives:
        invalid_graph = Graph().parse(data=graph.serialize(format="nt"), format="nt")
        invalid_graph.set(triple)
        try:
            project(invalid_graph, request["source"]["rdf_sha256"])
        except ValueError as error:
            require(expected_error in str(error), f"wrong RDF rejection: {error}")
        else:
            raise ValueError(f"RDF failed to reject {expected_error}")

    duplicate = copy.deepcopy(request)
    duplicate["nodes"].append(duplicate["nodes"][0])
    wrong_version = dict(request, version="2.0.0")
    invalid_mass = copy.deepcopy(request)
    chance = next(node["node"] for node in invalid_mass["nodes"] if node["node"]["kind"] == "chance")
    chance["outcomes"][0]["probability"] = 0.125
    bad_payoff = copy.deepcopy(request)
    terminal = next(node["node"] for node in bad_payoff["nodes"] if node["node"]["kind"] == "terminal")
    repeated = dict(terminal["payoffs"][0], id=iri(CASE.repeated))
    terminal["payoffs"].append(repeated)
    unknown = copy.deepcopy(request)
    unknown["nodes"][0]["node"]["unrecognised"] = "extra"
    duplicate_wire_key = exact_bytes.replace(b'"version": "1.0.0"', b'"version": "1.0.0", "version": "1.0.0"')
    cases = [
        (encode(duplicate), "DuplicateNode"),
        (encode(wrong_version), "Version"),
        (encode(invalid_mass), "ProbabilityMass"),
        (encode(bad_payoff), "DuplicatePayoff"),
        (encode(unknown), "unknown field"),
        (duplicate_wire_key, "duplicate field"),
    ]
    for payload, error in cases:
        result = run_solver(solver, payload)
        require(result.returncode and not result.stdout and error in result.stderr.decode(), f"expected rejection: {error}")
    return [error for _, error in rdf_negatives + cases]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--solver", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--ontology-dir", type=Path, help="use local immutable ontology module bytes")
    args = parser.parse_args()
    fixture = (HERE / "game.ttl").read_bytes()
    graph = Graph().parse(data=fixture, format="turtle")
    ontology = load_ontology(args.ontology_dir)
    verify_vocabulary(graph, ontology)
    request, used = project(graph, sha256(fixture))
    exact_bytes = encode(request)
    run = run_solver(args.solver.resolve(), exact_bytes)
    require(run.returncode == 0, run.stderr.decode())
    result = json.loads(run.stdout)
    for field in ("version", "semantic_profile", "source", "specification", "instance", "run"):
        require(result[field] == request[field], f"provenance changed: {field}")
    expected = json.loads((HERE / "expected-policy.json").read_text(encoding="utf-8"))
    require({field: result[field] for field in expected} == expected, "native result differed from the hand oracle")
    decisions = {record["id"]: record["node"] for record in request["nodes"]}
    require(
        [record["action"] for record in decisions[iri(CASE.start)]["actions"]]
        == ["https://example.org/paignia/lottery#choice", "https://example.org/paignia/safe#choice"],
        "full action identities or declared input order changed",
    )
    evidence = result_graph(result, used)
    verify_vocabulary(evidence, ontology)
    rejections = check_rejections(args.solver.resolve(), graph, request, exact_bytes)
    triples = ("\n".join(sorted(evidence.serialize(format="nt").splitlines())) + "\n").encode()
    restored = Graph().parse(data=triples, format="nt")
    require(set(restored) == set(evidence), "result RDF changed on reload")
    policy = exactly_one(restored.subjects(RDF.type, GT.GameSolution), "RDF result")
    choices = {
        iri(exactly_one(restored.objects(choice, BRIDGE.state), "policy node")):
        iri(exactly_one(restored.objects(choice, GT.selectsAction), "policy action"))
        for choice in restored.objects(policy, BRIDGE.choice)
    }
    payoffs = {
        iri(exactly_one(restored.objects(link, GT.payoffPlayer), "result player")):
        number(exactly_one(restored.objects(link, GT.payoffValue), "result payoff"))
        for link in restored.objects(policy, BRIDGE.expectedPayoff)
    }
    require(choices == expected["choices"], "result RDF lost selected action identities")
    require(payoffs == expected["expected_payoffs"], "result RDF lost expected payoffs")
    args.output.mkdir(parents=True, exist_ok=True)
    outputs = {"request.json": exact_bytes, "result.json": run.stdout + b"\n", "result.nt": triples}
    for name, payload in outputs.items():
        (args.output / name).write_bytes(payload)
    revision = subprocess.check_output(["git", "-c", "core.fsmonitor=false", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    dirty = bool(subprocess.check_output(["git", "-c", "core.fsmonitor=false", "status", "--porcelain"], cwd=ROOT).strip())
    expected_revision = os.environ.get("GITHUB_SHA")
    if expected_revision:
        require(revision == expected_revision and not dirty, "CI replay source differs from its checked revision")
    receipt = {
        "provider_revision": revision,
        "provider_worktree_dirty": dirty,
        "ontology_revision": ONTOLOGY,
        "source_sha256": sha256(fixture),
        "outputs": {name: sha256(payload) for name, payload in outputs.items()},
        "negative_controls": rejections,
    }
    (args.output / "receipt.json").write_bytes(encode(receipt))
    print(json.dumps(receipt, indent=2))


if __name__ == "__main__":
    main()
