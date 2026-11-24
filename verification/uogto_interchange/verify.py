"""Project one asserted finite UOGTO profile and check its native result."""

import argparse
import copy
import hashlib
import json
import math
import os
import re
from pathlib import Path
import subprocess
import sys
import urllib.request
from urllib.parse import urlsplit

import rdflib
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

# Finite lexical forms from https://www.w3.org/TR/xmlschema11-2/.
XSD_DECIMAL = r"[+-]?(?:[0-9]+(?:\.[0-9]*)?|\.[0-9]+)"
NUMERIC_FORMS = {
    XSD.integer: re.compile(r"[+-]?[0-9]+"),
    XSD.decimal: re.compile(XSD_DECIMAL),
    XSD.double: re.compile(XSD_DECIMAL + r"(?:[eE][+-]?[0-9]+)?"),
}



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


def numeric_literal(value):
    form = NUMERIC_FORMS.get(value.datatype) if isinstance(value, Literal) else None
    require(
        form is not None and form.fullmatch(str(value).strip(" \t\r\n")) and value.ill_typed is False,
        f"expected numeric literal: {value}",
    )
    return value.value


def number(value):
    try:
        result = float(numeric_literal(value))
    except OverflowError as error:
        raise ValueError(f"nonfinite numeric literal: {value}") from error
    require(math.isfinite(result), f"nonfinite numeric literal: {value}")
    return result


def position(value):
    result = numeric_literal(value)
    require(type(result) is int and result >= 0, f"invalid position: {value}")
    return result


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


def project(graph, source_hash, *, require_singleton=True, representation=None):
    spec = typed(graph, exactly_one(graph.subjects(RDF.type, GT.GameSpecification), "game"), GT.GameSpecification)
    representations = set(graph.objects(spec, GT.hasRepresentation))
    chosen = representation if representation is not None else exactly_one(representations, "representation")
    require(chosen in representations, "Selected representation is outside the declared game")
    rep = typed(graph, chosen, GT.ExtensiveFormRepresentation)
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
            if require_singleton:
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


def result_graph(result, used, request, source):
    actors = {
        record["id"]: record["node"]["player"]
        for record in request["nodes"] if record["node"]["kind"] == "decision"
    }
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
        graph.add((choice, BRIDGE.action, URIRef(action)))
        graph.add((choice, BRIDGE.player, URIRef(actors[state])))
    for index, (player, payoff) in enumerate(sorted(result["expected_payoffs"].items())):
        link = URIRef(f"{policy}/payoff/{index}")
        graph.add((policy, BRIDGE.expectedPayoff, link))
        graph.add((link, RDF.type, GT.PlayerPayoffLink))
        graph.add((link, GT.payoffPlayer, URIRef(player)))
        for identifier in source.objects(URIRef(player), GT.identifier):
            graph.add((URIRef(player), GT.identifier, identifier))
        graph.add((link, GT.payoffValue, Literal(payoff, datatype=XSD.double)))
    return graph


def encode(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True, allow_nan=False, indent=2).encode() + b"\n"


def run_solver(solver, payload):
    return subprocess.run([solver], input=payload, capture_output=True, timeout=30, check=False)


def check_rejections(solver, graph, fixture, request, exact_bytes):
    rdf_negatives = [
        ((CASE["safe-edge"], BRIDGE.position, Literal("0", datatype=XSD.integer)), "invalid order"),
        ((CASE.respond, GT.belongsToInformationSet, CASE["start-set"]), "non-singleton"),
    ]
    invalid_graphs = []
    for triple, expected_error in rdf_negatives:
        invalid = Graph().parse(data=graph.serialize(format="nt"), format="nt")
        invalid.set(triple)
        invalid_graphs.append((invalid, expected_error, expected_error))
    for lexical_form, label in [
        (b'"3.5"^^xsd:integer', "fractional integer literal"),
        (b'"1_0"^^xsd:integer', "underscore in integer literal"),
        (b'"1e2"^^xsd:decimal', "exponent in decimal literal"),
    ]:
        marker = b'"8"^^xsd:double'
        require(fixture.count(marker) == 1, "numeric rejection fixture marker changed")
        invalid = Graph().parse(data=fixture.replace(marker, lexical_form), format="turtle")
        invalid_graphs.append((invalid, "expected numeric literal", label))
    for invalid, expected_error, label in invalid_graphs:
        try:
            project(invalid, request["source"]["rdf_sha256"])
        except ValueError as error:
            require(expected_error in str(error), f"wrong RDF rejection: {error}")
        else:
            raise ValueError(f"RDF failed to reject {label}")

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
    conflict = copy.deepcopy(request)
    payouts = [item["node"]["payoffs"] for item in conflict["nodes"] if item["node"]["kind"] == "terminal"]
    payouts[0][0]["id"] = payouts[1][0]["id"]
    require(payouts[0][0]["value"] != payouts[1][0]["value"], "conflict fixture needs different utilities")
    unknown = copy.deepcopy(request)
    unknown["nodes"][0]["node"]["unrecognised"] = "extra"
    duplicate_wire_key = exact_bytes.replace(b'"version": "1.0.0"', b'"version": "1.0.0", "version": "1.0.0"')
    cases = [
        (encode(duplicate), "DuplicateNode"),
        (encode(wrong_version), "Version"),
        (encode(invalid_mass), "ProbabilityMass"),
        (encode(bad_payoff), "DuplicatePayoff"),
        (encode(conflict), "SourceIdentity"),
        (encode(unknown), "unknown field"),
        (duplicate_wire_key, "duplicate field"),
    ]
    for payload, error in cases:
        result = run_solver(solver, payload)
        require(result.returncode and not result.stdout and error in result.stderr.decode(), f"expected rejection: {error}")
    return [label for _, _, label in invalid_graphs] + [error for _, error in cases]


def source_state():
    git = ["git", "-c", "core.fsmonitor=false", "-C", str(ROOT)]
    head = subprocess.check_output(git + ["rev-parse", "HEAD"], text=True).strip()
    dirty = bool(subprocess.check_output(git + ["status", "--porcelain"]).strip())
    require(not dirty, "commit the replay sources before recording a receipt")
    expected_revision = os.environ.get("GITHUB_SHA")
    require(not expected_revision or expected_revision == head, "CI replay source differs from its checked revision")
    return head


def build_solver(example="solve"):
    manifest = ROOT / "crates" / "paignia-io" / "Cargo.toml"
    target = ROOT / "target" / "uogto-build"
    command = [
        "cargo", "build", "--release", "--locked", "--manifest-path", str(manifest),
        "--target-dir", str(target), "--example", example, "--message-format=json",
    ]
    build = subprocess.run(command, cwd=ROOT, text=True, stdout=subprocess.PIPE, timeout=900, check=True)
    artifacts = [json.loads(line) for line in build.stdout.splitlines()]
    for item in artifacts:
        if item.get("reason") == "compiler-message" and item["message"].get("rendered"):
            print(item["message"]["rendered"], file=sys.stderr, end="")
    executables = [
        Path(item["executable"]).resolve()
        for item in artifacts
        if item.get("reason") == "compiler-artifact" and item.get("executable")
        and item["target"]["name"] == example and item["target"]["kind"] == ["example"]
    ]
    solver = exactly_one(executables, "built native solver")
    require(solver.is_relative_to(target.resolve()), "Cargo reported a solver outside the build target")
    return solver


def check_shared_profile(solver, graph, expected):
    shared = Graph().parse(data=graph.serialize(format="nt"), format="nt")
    shared.set((CASE.decline, BRIDGE.payoffProfile, CASE["safe-payoffs"]))
    data = shared.serialize(format="nt").encode()
    request, _ = project(shared, sha256(data))
    run = run_solver(solver, encode(request))
    require(run.returncode == 0, run.stderr.decode())
    result = json.loads(run.stdout)
    require({field: result[field] for field in expected} == expected, "shared payoff profile changed the oracle")



def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--ontology-dir", type=Path, help="use local immutable ontology module bytes")
    args = parser.parse_args()
    revision = source_state()
    solver = build_solver()
    binary_digest = sha256(solver.read_bytes())
    # Preserve lexical forms before RDFLib converts numeric values.
    rdflib.NORMALIZE_LITERALS = False
    fixture = (HERE / "game.ttl").read_bytes()
    graph = Graph().parse(data=fixture, format="turtle")
    ontology = load_ontology(args.ontology_dir)
    verify_vocabulary(graph, ontology)
    request, used = project(graph, sha256(fixture))
    exact_bytes = encode(request)
    frozen = HERE / "frozen-static-v1"
    frozen_run = run_solver(solver, (frozen / "request.json").read_bytes())
    require(frozen_run.returncode == 0, frozen_run.stderr.decode())
    require(frozen_run.stdout + b"\n" == (frozen / "result.json").read_bytes(),
            "frozen v1 wire output changed")
    run = run_solver(solver, exact_bytes)
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
    evidence = result_graph(result, used, request, graph)
    verify_vocabulary(evidence, ontology)
    check_shared_profile(solver, graph, expected)
    rejections = check_rejections(solver, graph, fixture, request, exact_bytes)
    triples = ("\n".join(sorted(evidence.serialize(format="nt").splitlines())) + "\n").encode()
    restored = Graph().parse(data=triples, format="nt")
    require(set(restored) == set(evidence), "result RDF changed on reload")
    policy = exactly_one(restored.subjects(RDF.type, GT.GameSolution), "RDF result")
    choices = {
        iri(exactly_one(restored.objects(choice, BRIDGE.state), "policy node")):
        iri(exactly_one(restored.objects(choice, BRIDGE.action), "policy action"))
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
    require(source_state() == revision and sha256(solver.read_bytes()) == binary_digest, "source or binary changed during replay")
    receipt = {
        "provider_revision": revision,
        "provider_worktree_dirty": False,
        "cargo": subprocess.check_output(["cargo", "--version"], cwd=ROOT, text=True).strip(),
        "solver_sha256": binary_digest,
        "shared_payoff_profile": "passed",
        "frozen_static_v1": {name: sha256((frozen / name).read_bytes())
                             for name in ("request.json", "result.json")},
        "ontology_revision": ONTOLOGY,
        "source_sha256": sha256(fixture),
        "outputs": {name: sha256(payload) for name, payload in outputs.items()},
        "negative_controls": rejections,
    }
    (args.output / "receipt.json").write_bytes(encode(receipt))
    print(json.dumps(receipt, indent=2))


if __name__ == "__main__":
    main()
