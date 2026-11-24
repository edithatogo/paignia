"""Check the public RDF boundary against its pinned validation bundle."""

import os
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest
from unittest.mock import patch

from rdflib import Literal, Namespace, RDF, URIRef, XSD

from interchange import Bundle, FORMATS, HERE, parse_graph, main, valid_literal, verified_bytes

GT = Namespace("https://w3id.org/uogto/core#")


class InterchangeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.directory = Path(os.environ["UOGTO_BUNDLE"])

    def test_public_roundtrip_preserves_assertions_offline(self):
        with TemporaryDirectory() as temp:
            source = Path(temp) / "graph.ttl"
            output = Path(temp) / "graph.nt"
            source.write_text(
                '<urn:buyer> <urn:note> [ <urn:value> "+001"^^<http://www.w3.org/2001/XMLSchema#integer> ; '
                '<urn:caption> "café\u2028au lait"@fr ; <urn:choice> <urn:one:choice>, <urn:two:choice> ] .\n'
                '<urn:buyer> <urn:number> +001, 1, 01.20 ; '
                '<urn:whitespace> "  a  b "^^<http://www.w3.org/2001/XMLSchema#token> .',
                encoding="utf-8",
            )
            args = ["interchange.py", "roundtrip", "--ontology-dir", str(self.directory),
                    "--input", str(source), "--output", str(output)]
            with patch("sys.argv", args), patch("interchange.urlopen", side_effect=AssertionError):
                main()
            rendered = output.read_text(encoding="utf-8")
            with patch("sys.argv", args):
                main()
            self.assertEqual(rendered, output.read_text(encoding="utf-8"))
            self.assertIn('"+001"^^', rendered)
            self.assertIn('"café\u2028au lait"@fr', rendered)
            self.assertIn('"01.20"^^', rendered)
            self.assertIn('"  a  b "^^', rendered)
            self.assertEqual(rendered.count("<urn:number>"), 3)
            result = parse_graph(output.read_bytes(), "nt")
            self.assertEqual(len(result), 9)
            self.assertEqual(set(result.objects(None, URIRef("urn:choice"))),
                             {URIRef("urn:one:choice"), URIRef("urn:two:choice")})

    def test_shacl_uses_declared_float_value_space(self):
        source = b"""
            @prefix g: <https://w3id.org/uogto/core#> .
            @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
            <urn:edge> g:probability "1.00000001"^^xsd:float ; <urn:limit> "+INF"^^xsd:double .
        """
        bundle = Bundle(self.directory)
        output, _ = bundle.roundtrip(source, FORMATS[".ttl"])
        self.assertIn(b'"1.00000001"^^', output)
        self.assertIn(b'"+INF"^^', output)
        above_one = source.replace(b"1.00000001", b"1.0000000596046447753906250000000000001")
        with self.assertRaisesRegex(ValueError, "Probability values"):
            bundle.roundtrip(above_one, FORMATS[".ttl"])

    def test_rejects_missing_required_relationship(self):
        graph = parse_graph((HERE / "game.ttl").read_bytes(), "turtle")
        player = next(graph.subjects(RDF.type, GT.Player))
        graph.remove((player, GT.identifier, None))
        with self.assertRaisesRegex(ValueError, "MinCountConstraintComponent"):
            Bundle(self.directory).validate(graph)

    def test_input_schema_cannot_supply_a_missing_shacl_assertion(self):
        graph = b"""
            @prefix g: <https://w3id.org/uogto/core#> .
            @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
            <urn:game> a g:GameSpecification ; <urn:unqualifiedPlayer> <urn:buyer> .
            <urn:unqualifiedPlayer> rdfs:subPropertyOf g:hasPlayer .
        """
        with self.assertRaisesRegex(ValueError, "MinCountConstraintComponent"):
            Bundle(self.directory).roundtrip(graph, FORMATS[".ttl"])

    def test_rejects_wrong_value_kind_or_unknown_class(self):
        bundle = Bundle(self.directory)
        for statement, expected in [
            ((URIRef("urn:example"), GT.hasPlayer, Literal("player")), "wrong value kind"),
            ((URIRef("urn:example"), RDF.type, GT.Missing), "Unknown UOGTO class"),
        ]:
            graph = parse_graph(b"", "turtle")
            graph.add(statement)
            with self.assertRaisesRegex(ValueError, expected):
                bundle.validate(graph)

    def test_rejects_malformed_literal_and_changed_bundle_bytes(self):
        for lexical, datatype in [("1_0", XSD.integer), ("1e2", XSD.decimal), ("inf", XSD.double)]:
            with self.subTest(lexical=lexical):
                self.assertFalse(valid_literal(Literal(lexical, datatype=datatype, normalize=False)))
        with self.assertRaisesRegex(ValueError, "hash mismatch"):
            verified_bytes("ontologies/core/uogto-core.ttl", "0" * 64, b"changed")


if __name__ == "__main__":
    unittest.main()
