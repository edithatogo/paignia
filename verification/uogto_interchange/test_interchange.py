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
                '<urn:caption> "café\u2028au lait"@fr ; <urn:choice> <urn:one:choice>, <urn:two:choice> ] .',
                encoding="utf-8",
            )
            args = ["interchange.py", "roundtrip", "--ontology-dir", str(self.directory),
                    "--input", str(source), "--output", str(output)]
            with patch("sys.argv", args), patch("interchange.urlopen", side_effect=AssertionError):
                main()
            rendered = output.read_text(encoding="utf-8")
            self.assertIn('"+001"^^', rendered)
            self.assertIn('"café\u2028au lait"@fr', rendered)
            result = parse_graph(output.read_bytes(), FORMATS[".nt"])
            self.assertEqual(len(result), 5)
            self.assertEqual(set(result.objects(None, URIRef("urn:choice"))),
                             {URIRef("urn:one:choice"), URIRef("urn:two:choice")})

    def test_rejects_missing_required_relationship(self):
        graph = parse_graph((HERE / "game.ttl").read_bytes(), "turtle")
        player = next(graph.subjects(RDF.type, GT.Player))
        graph.remove((player, GT.identifier, None))
        with self.assertRaisesRegex(ValueError, "MinCountConstraintComponent"):
            Bundle(self.directory).roundtrip(graph)

    def test_rejects_wrong_value_kind_or_unknown_class(self):
        bundle = Bundle(self.directory)
        for statement, expected in [
            ((URIRef("urn:example"), GT.hasPlayer, Literal("player")), "wrong value kind"),
            ((URIRef("urn:example"), RDF.type, GT.Missing), "Unknown UOGTO class"),
        ]:
            graph = parse_graph(b"", "turtle")
            graph.add(statement)
            with self.assertRaisesRegex(ValueError, expected):
                bundle.roundtrip(graph)

    def test_rejects_malformed_literal_and_changed_bundle_bytes(self):
        for lexical, datatype in [("1_0", XSD.integer), ("1e2", XSD.decimal), ("inf", XSD.double)]:
            with self.subTest(lexical=lexical):
                self.assertFalse(valid_literal(Literal(lexical, datatype=datatype, normalize=False)))
        with self.assertRaisesRegex(ValueError, "hash mismatch"):
            verified_bytes("ontologies/core/uogto-core.ttl", "0" * 64, b"changed")


if __name__ == "__main__":
    unittest.main()
