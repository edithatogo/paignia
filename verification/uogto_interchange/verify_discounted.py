"""Replay the pinned UOGTO discounted funding case through the Rust wire."""

import argparse
from copy import deepcopy
from fractions import Fraction
import json
from pathlib import Path
import sys

from rdflib import Graph, OWL, RDF

sys.path.insert(0, str(Path(__file__).resolve().parent))
from interchange import digest, stage, verified_bytes
from verify import build_solver, encode, require, run_solver

LOCK = Path(__file__).with_name("discounted-ontology-lock.json")
FIXTURE = "examples/recurring-funding-discounted-markov-v1.ttl"
TERMS = "ontologies/extensions/discounted-markov.ttl"
PROFILE = "paignia/discounted-zero-sum-2x2/1.0.0"


def check_interval(estimate, bound, reference):
    require(
        abs(Fraction(estimate) - reference) <= Fraction(bound) <= Fraction("1e-10"),
        f"Unqualified value bound for {reference}",
    )


def check_values(evaluation, state_ids):
    require(evaluation["status"] == "value_tolerance_reached", "Value tolerance not reached")
    values = evaluation["state_values"]
    require([item["state"] for item in values] == state_ids, "State axis changed")
    for item, reference in zip(values, [Fraction(32, 13), Fraction(-16, 13)], strict=True):
        check_interval(item["value"], evaluation["value_error_bound"], reference)
    check_interval(
        evaluation["initial_value"], evaluation["initial_error_bound"], Fraction(20, 13),
    )


def verify(bundle, output, solver=None):
    lock = json.loads(LOCK.read_bytes())
    pinned = {
        item["path"]: verified_bytes(
            item["path"], item["sha256"], (bundle / item["path"]).read_bytes(),
        )
        for item in lock["files"]
    }
    # Import the exact hash-checked upstream projector and its SHACL.
    sys.path.insert(0, str(bundle.resolve()))
    from uogto.interchange.discounted_markov import (
        FROZEN_REPRESENTATION_IRI,
        FROZEN_SPECIFICATION_IRI,
        project_game,
    )

    source = pinned[FIXTURE]
    graph = Graph().parse(data=source, format="turtle")
    game = project_game(graph, FROZEN_SPECIFICATION_IRI)
    ontology = Graph().parse(data=pinned[TERMS], format="turtle")
    class_ids = set(ontology.subjects(RDF.type, OWL.Class))
    property_ids = set(ontology.subjects(RDF.type, OWL.ObjectProperty))
    property_ids.update(ontology.subjects(RDF.type, OWL.DatatypeProperty))
    require((len(class_ids), len(property_ids)) == (2, 11), "Unexpected T22 vocabulary inventory")
    require(class_ids <= set(graph.objects(None, RDF.type)), "A T22 class lacks fixture evidence")
    require(property_ids <= set(graph.predicates()), "A T22 property lacks fixture evidence")
    spec = str(FROZEN_SPECIFICATION_IRI)
    output.mkdir(parents=True, exist_ok=True)
    solver = build_solver("solve_profile") if solver is None else solver.resolve(strict=True)
    methods = {"evaluate": "stationary_policy_evaluation", "minimax": "discounted_zero_sum_minimax"}
    receipts = {}
    for kind, method in methods.items():
        request = {
            "version": "1.0.0",
            "semantic_profile": PROFILE,
            "source": {
                "rdf_sha256": digest(source),
                "ontology_revision": lock["revision"],
                "representation": str(FROZEN_REPRESENTATION_IRI),
            },
            "specification": spec,
            "instance": f"{spec}/instance",
            "run": f"{spec}/run/{kind}",
            "game": deepcopy(game),
        }
        if kind == "minimax":
            request["game"]["method"] = {"kind": "minimax"}
        payload = encode(request)
        run = run_solver(solver, payload)
        require(run.returncode == 0, f"Discounted native solver failed for {kind}")
        result = json.loads(run.stdout)
        for binding in ("version", "semantic_profile", "source", "specification", "instance", "run"):
            require(result[binding] == request[binding], f"Changed result binding {binding}")
        require(result["method"] == method, "Wrong method identity")
        require(result["method_version"] == "1.0.0", "Wrong method version")
        require(result["status"] == "value_tolerance_reached", "Unexpected result status")
        details = result["result"]
        require(details["players"] == game["players"], "Player or action axis changed")
        require(details["policy"] == game["method"]["policy"], "Stationary policy changed")
        states = [record["id"] for record in game["states"]]
        check_values(details["policy_values"], states)
        if kind == "minimax":
            for key in ("game_values", "row_best_response", "column_best_response"):
                check_values(details[key], states)
            require(
                len(details["deviation_bounds"]) == 2
                and all(0 <= bound <= 2e-10 for bound in details["deviation_bounds"]),
                "Unqualified deviation bound",
            )
        (output / f"{kind}-request.json").write_bytes(payload)
        (output / f"{kind}-response.json").write_bytes(run.stdout)
        receipts[kind] = {"request_sha256": digest(payload), "response_sha256": digest(run.stdout)}
    (output / "receipt.json").write_bytes(encode({
        "ontology_revision": lock["revision"],
        "rdf_sha256": digest(source),
        "classes": sorted(map(str, class_ids)),
        "properties": sorted(map(str, property_ids)),
        "executions": receipts,
    }))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("stage", "replay"))
    parser.add_argument("--bundle", required=True, type=Path)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--solver", type=Path)
    args = parser.parse_args()
    if args.command == "stage":
        stage(args.bundle, LOCK)
    else:
        if args.output is None:
            parser.error("--output is required for replay")
        verify(args.bundle, args.output, args.solver)


if __name__ == "__main__":
    main()
