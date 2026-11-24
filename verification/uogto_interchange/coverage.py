"""Replay the UOGTO term witnesses and fail on uncovered declarations."""

import argparse
import json
import os
from pathlib import Path
import posixpath
import subprocess
import sys

from rdflib import Namespace, RDF, RDFS

sys.path.insert(0, str(Path(__file__).resolve().parent))
from interchange import Bundle, FORMATS, HERE, SOURCE, digest, parse_graph

SKOS = Namespace("http://www.w3.org/2004/02/skos/core#")


def inline_context(bundle, value, directory):
    if isinstance(value, str):
        path = posixpath.normpath(posixpath.join(directory, value))
        if not path.startswith("jsonld/") or path not in bundle.files:
            raise ValueError(f"JSON-LD context is outside the locked UOGTO bundle: {value}")
        content = json.loads(bundle.files[path])
        return inline_context(bundle, content["@context"], posixpath.dirname(path))
    if isinstance(value, list):
        return [inline_context(bundle, item, directory) for item in value]
    if "@import" in value:
        raise ValueError("JSON-LD context imports must be staged explicitly")
    return value


def fixtures(bundle):
    for path, data in sorted(bundle.files.items()):
        if not path.startswith("examples/"):
            continue
        iri = f"{SOURCE}/{bundle.revision}/{path}"
        if path.endswith(".ttl"):
            yield f"UOGTO/{path}", data, data, FORMATS[".ttl"], iri
        elif path.endswith(".jsonld"):
            document = json.loads(data)
            document["@context"] = inline_context(bundle, document["@context"], "examples")
            adapted = parse_graph(json.dumps(document), "json-ld", iri).serialize(format="nt", encoding="utf-8")
            yield f"UOGTO/{path}", data, adapted, FORMATS[".nt"], iri

    for path in (HERE / "game.ttl", *sorted((HERE / "scenarios").glob("*.ttl"))):
        source = path.read_bytes()
        yield str(path.relative_to(HERE)), source, source, FORMATS[".ttl"], path.as_uri()


def term_proof(bundle, term, graph, triple, fixture, declarations):
    subject = triple[0]
    definition = bundle.ontology.value(term, SKOS.definition) or bundle.ontology.value(term, RDFS.comment)
    if definition is None:
        raise ValueError(f"No source definition for {term}")
    subject_types = sorted(str(value) for value in graph.objects(subject, RDF.type))
    return {
        "fixture": fixture,
        "assertion": [part.n3() for part in triple],
        "subject_types": subject_types,
        "declared_in": declarations,
        "definition": str(definition),
    }


def replay(bundle):
    classes = {}
    properties = {}
    cases = {}
    inventory = bundle.inventory()
    for name, original, source, format_name, iri in fixtures(bundle):
        try:
            output, graph = bundle.roundtrip(source, format_name, iri)
        except (ValueError, SyntaxError) as error:
            raise ValueError(f"{name}: {error}") from error
        cases[name] = {"source_sha256": digest(original), "roundtrip_sha256": digest(output)}
        for triple in sorted(graph, key=lambda triple: tuple(part.n3() for part in triple)):
            subject, predicate, value = triple
            if predicate == RDF.type and value in bundle.classes and value not in classes:
                # An instance witness must participate in the case beyond its class assertion.
                grounded = any(p != RDF.type for p in graph.predicates(subject)) or any(graph.subjects(None, subject))
                if grounded:
                    classes[value] = term_proof(bundle, value, graph, triple, name, inventory["classes"][value])
            if predicate in bundle.properties and predicate not in properties:
                properties[predicate] = term_proof(
                    bundle, predicate, graph, triple, name, inventory["properties"][predicate],
                )
    missing = sorted(bundle.classes.keys() - classes.keys()) + sorted(bundle.properties.keys() - properties.keys())
    if missing:
        raise ValueError(
            f"{len(classes)}/{len(bundle.classes)} classes and {len(properties)}/{len(bundle.properties)} properties. "
            f"Missing {len(missing)} witnesses:\n" + "\n".join(map(str, missing)),
        )
    revision = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=HERE, text=True).strip()
    if os.environ.get("GITHUB_SHA", revision) != revision:
        raise ValueError("Coverage checkout differs from the CI revision")
    return {
        "schema": "paignia.uogto-term-coverage.v1",
        "provider_revision": revision,
        "ontology_revision": bundle.revision,
        "ontology_lock_sha256": bundle.lock_sha256,
        "fixtures": cases,
        "classes": dict(sorted(classes.items())),
        "properties": dict(sorted(properties.items())),
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--ontology-dir", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    try:
        report = replay(Bundle(args.ontology_dir))
    except (OSError, ValueError) as error:
        parser.exit(1, f"{error}\n")
    args.output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"{len(report['classes'])} classes and {len(report['properties'])} properties passed", file=sys.stderr)


if __name__ == "__main__":
    main()
