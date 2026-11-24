"""Reject vocabulary witnesses that conflict with declared source roles."""

import os
from pathlib import Path
import unittest

from rdflib import Namespace

from coverage import property_expectations
from interchange import Bundle, parse_graph

EXT = Namespace("https://w3id.org/uogto/extensions#")


class TermEvidenceTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.bundle = Bundle(Path(os.environ["UOGTO_BUNDLE"]))

    def test_auction_witness_needs_intended_subject_and_literal_type(self):
        graph = parse_graph(b"""
            @prefix g: <https://w3id.org/uogto/core#> .
            @prefix x: <https://w3id.org/uogto/extensions#> .
            @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
            <urn:outcome> a g:Outcome ; x:hasWinningBid "13"^^xsd:double .
            <urn:decimal> a x:Auction ; x:hasWinningBid 13.0 .
            <urn:auction> a x:Auction ; x:hasWinningBid "13"^^xsd:double .
        """, "turtle")
        accepted = [str(triple[0]) for triple in graph.triples((None, EXT.hasWinningBid, None))
                    if property_expectations(self.bundle, graph, triple) is not None]
        self.assertEqual(accepted, ["urn:auction"])

    def test_transaction_cost_uses_canonical_instance_role(self):
        graph = parse_graph(b"""
            @prefix g: <https://w3id.org/uogto/core#> .
            @prefix x: <https://w3id.org/uogto/extensions#> .
            <urn:trade> a g:GameInstance ; x:hasTransactionCost <urn:cost> .
            <urn:other> a g:GameSpecification ; x:hasTransactionCost <urn:cost> .
            <urn:cost> a x:TransactionCost .
        """, "turtle")
        accepted = [str(triple[0]) for triple in graph.triples((None, EXT.hasTransactionCost, None))
                    if property_expectations(self.bundle, graph, triple) is not None]
        self.assertEqual(accepted, ["urn:trade"])


    def test_prose_only_profile_link_needs_equilibrium_subject(self):
        graph = parse_graph(b"""
            @prefix g: <https://w3id.org/uogto/core#> .
            <urn:game> a g:DynamicGame ; g:hasStrategyProfile <urn:profile> .
            <urn:equilibrium> a g:Equilibrium ; g:hasStrategyProfile <urn:profile> .
            <urn:profile> a g:StrategyProfile .
        """, "turtle")
        core = Namespace("https://w3id.org/uogto/core#")
        accepted = [str(triple[0]) for triple in graph.triples((None, core.hasStrategyProfile, None))
                    if property_expectations(self.bundle, graph, triple) is not None]
        self.assertEqual(accepted, ["urn:equilibrium"])
