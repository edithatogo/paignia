"""Validate and round-trip asserted RDF against the pinned UOGTO source."""

import argparse
from concurrent.futures import ThreadPoolExecutor
from hashlib import sha256
import json
from pathlib import Path
import re
from urllib.request import urlopen

from pyoxigraph import RdfFormat, parse as parse_rdf, serialize as serialize_rdf
from pyshacl import validate as shacl_validate
import rdflib
from rdflib import Graph, Literal, OWL, RDF, RDFS, URIRef, XSD

HERE = Path(__file__).resolve().parent
LOCK = HERE / "ontology-lock.json"
SOURCE = "https://raw.githubusercontent.com/edithatogo/UOGTO"
UOGTO = "https://w3id.org/uogto/"
CLASS_TYPES = {OWL.Class, RDFS.Class}
PROPERTY_TYPES = {OWL.ObjectProperty, OWL.DatatypeProperty, OWL.AnnotationProperty, RDF.Property}
FORMATS = {".ttl": RdfFormat.TURTLE, ".nt": RdfFormat.N_TRIPLES}
SCHEMA_PREDICATES = {RDFS.subClassOf, RDFS.subPropertyOf, RDFS.domain, RDFS.range}
INTEGER = re.compile(r"[+-]?[0-9]+")
DECIMAL = re.compile(r"[+-]?(?:[0-9]+(?:\.[0-9]*)?|\.[0-9]+)")
REAL = re.compile(DECIMAL.pattern + r"(?:[eE][+-]?[0-9]+)?")


def digest(data):
    return sha256(data).hexdigest()


def owned(term):
    return isinstance(term, URIRef) and term.startswith(UOGTO)


def parse_graph(data, format_name, base=None):
    # XSD lexical forms are part of an asserted RDF term.
    rdflib.NORMALIZE_LITERALS = False
    return Graph().parse(data=data, format=format_name, publicID=base)


def verified_bytes(path, expected, data):
    if digest(data) != expected:
        raise ValueError(f"UOGTO file hash mismatch: {path}")
    return data


def stage(directory):
    lock = json.loads(LOCK.read_bytes())

    def fetch(entry):
        path, expected = entry["path"], entry["sha256"]
        destination = directory / path
        if destination.exists():
            verified_bytes(path, expected, destination.read_bytes())
            return
        url = f"{SOURCE}/{lock['revision']}/{path}"
        with urlopen(url, timeout=30) as response:
            data = verified_bytes(path, expected, response.read(1_048_577))
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes(data)

    with ThreadPoolExecutor(max_workers=12) as pool:
        list(pool.map(fetch, lock["files"]))


def valid_literal(literal):
    if literal.ill_typed:
        return False
    datatype = literal.datatype
    if not datatype or not datatype.startswith(str(XSD)):
        return True
    lexical = str(literal).strip(" \t\r\n")
    if datatype == XSD.decimal:
        return DECIMAL.fullmatch(lexical) is not None
    if datatype in {XSD.double, XSD.float}:
        return lexical in {"INF", "-INF", "NaN"} or REAL.fullmatch(lexical) is not None
    if type(literal.value) is int:
        return INTEGER.fullmatch(lexical) is not None
    return True


class Bundle:
    def __init__(self, directory):
        lock_bytes = LOCK.read_bytes()
        lock = json.loads(lock_bytes)
        self.revision = lock["revision"]
        self.lock_sha256 = digest(lock_bytes)
        self.files = {
            entry["path"]: verified_bytes(
                entry["path"], entry["sha256"], (directory / entry["path"]).read_bytes(),
            )
            for entry in lock["files"]
        }
        if len(self.files) != len(lock["files"]):
            raise ValueError("Duplicate UOGTO lock file path")
        self.ontology = Graph()
        self.shapes = Graph()
        self.classes = {}
        self.properties = {}
        for path, data in self.files.items():
            if path.startswith("shapes/"):
                self.shapes += parse_graph(data, "turtle", f"{SOURCE}/{self.revision}/{path}")
            if not path.startswith("ontologies/"):
                continue
            graph = parse_graph(data, "turtle", f"{SOURCE}/{self.revision}/{path}")
            self.ontology += graph
            for term, role in graph.subject_objects(RDF.type):
                if not owned(term):
                    continue
                if role in CLASS_TYPES:
                    self.classes.setdefault(term, set()).add(path)
                if role in PROPERTY_TYPES:
                    self.properties.setdefault(term, {}).setdefault(role, set()).add(path)
        if (len(self.classes), len(self.properties)) != (lock["classes"], lock["properties"]):
            raise ValueError("Pinned UOGTO declaration counts differ from the lock")

    def inventory(self):
        return {
            "revision": self.revision,
            "lock_sha256": self.lock_sha256,
            "classes": {iri: sorted(paths) for iri, paths in sorted(self.classes.items())},
            "properties": {
                iri: {role: sorted(paths) for role, paths in sorted(roles.items())}
                for iri, roles in sorted(self.properties.items())
            },
        }

    def validate(self, graph):
        for subject, predicate, value in graph:
            if predicate == RDF.type and owned(value) and value not in self.classes:
                raise ValueError(f"Unknown UOGTO class {value} for {subject}")
            if owned(predicate):
                specific = OWL.DatatypeProperty if isinstance(value, Literal) else OWL.ObjectProperty
                accepted = {specific, OWL.AnnotationProperty, RDF.Property}
                if accepted.isdisjoint(self.properties.get(predicate, ())):
                    raise ValueError(f"Unknown UOGTO property or wrong value kind for {predicate} at {subject}")
            if isinstance(value, Literal) and not valid_literal(value):
                raise ValueError(f"Ill-typed literal {value.n3()} for {predicate} at {subject}")
        # Only the pinned bundle may supply schema axioms used for inference.
        data = Graph()
        data += (triple for triple in graph if triple[1] not in SCHEMA_PREDICATES)
        conforms, _, report = shacl_validate(
            data, shacl_graph=self.shapes, ont_graph=self.ontology,
            inference="rdfs", inplace=False, do_owl_imports=False,
        )
        if not conforms:
            raise ValueError(str(report))

    def roundtrip(self, source, format_name, base=None):
        asserted = set(parse_rdf(input=source, format=format_name, base_iri=base, without_named_graphs=True))
        rendered = serialize_rdf(asserted, format=RdfFormat.N_TRIPLES)
        output = b"".join(sorted(line + b"\n" for line in rendered.split(b"\n") if line))
        if set(parse_rdf(input=output, format=RdfFormat.N_TRIPLES)) != asserted:
            raise ValueError("RDF terms changed during N-Triples round-trip")
        restored = parse_graph(output, "nt")
        self.validate(restored)
        return output, restored


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    for name in ("stage", "inventory", "roundtrip"):
        command = commands.add_parser(name)
        command.add_argument("--ontology-dir", required=True, type=Path)
        if name == "roundtrip":
            command.add_argument("--input", required=True, type=Path)
            command.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    try:
        if args.command == "stage":
            stage(args.ontology_dir)
            return
        bundle = Bundle(args.ontology_dir)
        if args.command == "inventory":
            print(json.dumps(bundle.inventory(), ensure_ascii=False, indent=2))
            return
        format_name = FORMATS.get(args.input.suffix.lower())
        if not format_name:
            raise ValueError("Input must have a .ttl or .nt extension")
        output, _ = bundle.roundtrip(args.input.read_bytes(), format_name, args.input.resolve().as_uri())
        args.output.write_bytes(output)
    except (OSError, ValueError, SyntaxError, rdflib.exceptions.Error) as error:
        parser.exit(1, f"{error}\n")


if __name__ == "__main__":
    main()
