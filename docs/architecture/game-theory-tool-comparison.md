# Game-theory software comparison

This comparison uses Paignia 0.1.0, the Gambit 16.7 documentation and the
current OpenSpiel documentation. It compares available capabilities on
25 November 2026. Timing and accuracy require matched-case measurements.

| Feature | Paignia | Gambit | OpenSpiel |
| --- | --- | --- | --- |
| Main research use | Embed bounded game methods in a Rust application and keep source identities and result evidence. | Construct and analyse finite strategic and extensive-form games with equilibrium solvers, Python, a command line and a GUI. | Run research on learning, search and planning in procedural game environments through C++ or Python. |
| Representation | Explicit finite acyclic decision/chance graphs, a separate perfect-recall information-set profile, and bounded two-player normal form. Reuse a continuation as a DAG node. | Strategic tables and extensive game trees with information sets and chance. Reduced strategic form can be computed from an extensive game. Rational probabilities and utilities are available. | Procedural states and action histories, explicit or supported sampled chance, simultaneous moves, observations and information states. Includes normal-form and matrix games. |
| Equilibrium and policy methods | Backward induction with chance, pure and bounded mixed Nash, CFR/CFR+, optional correlated equilibrium, binary commitment, paired-payoff segment bounds and replicator integration in documented domains. | Pure enumeration, support and vertex enumeration, complementarity, linear programming for constant-sum games, and numerical methods for harder Nash problems. Also logit QRE. Exact rational arithmetic is available for relevant methods. | CFR variants, best response and exploitability, search, multi-agent learning and evaluation tools. It includes game collections for experimentation. |
| Exchange and traceability | Optional versioned JSON requests/results with source binding. UOGTO RDF projections and locked witness examples supply a semantic exchange path. Native results include method, status and diagnostics. | Native model files such as NFG and EFG, Python model objects and import of supported materialized OpenSpiel games. | Game parameters, serialized games and states, bots and trajectories. Procedural games and game-specific types can require an adapter for external exchange. |
| Deployment and teaching | Rust crates, a locked example and reproducible RDF/JSON cases. The optional Kairos adapter executes a qualified finite path. | Desktop and command-line tools, Python package, tutorials, and examples. | C++ core and Python interfaces, extensive runnable games, algorithms and examples. |

Sources for the capability claims. [Paignia provider contracts](../validation/provider-review-20261120.md) ·
[Gambit documentation](https://gambitproject.readthedocs.io/en/stable/) ·
[OpenSpiel capabilities](https://openspiel.readthedocs.io/en/latest/intro.html) ·
[OpenSpiel algorithms](https://openspiel.readthedocs.io/en/latest/algorithms.html)

Paignia's strongest comparison is an embedded, source-bound Rust workflow for
an admitted game with repeatable model exchange. Gambit can provide an exact
oracle on suitably translated rational cases. OpenSpiel can provide reference
policies, values and exploitability for suitably translated finite games. A
fair comparison records precisely which representations are equivalent, how
ties are handled and which result has been computed.

## What the JSON and RDF profiles standardize

UOGTO provides a versioned ontology, including OWL vocabulary, SHACL shapes and
JSON-LD contexts. The W3C Recommendations define RDF, SHACL and JSON-LD. The
ontology supplies game-specific terms and constraints over those technologies.
Paignia's optional version 1 JSON request and response give compatible clients
an executable contract over specific model profiles. The public replay verifies
RDF-to-request, native solving, and result-to-RDF for bounded profiles. It also
checks the locked UOGTO vocabulary witnesses on their own terms.

The current combination is a project-defined interoperability profile. A
standards proposal would need a normative mapping, machine-readable validation
where suitable, explicit information-loss and compatibility rules, conformance
fixtures and independent implementations. The successful RDF witness census
establishes representational coverage for a pinned vocabulary. Numerical game
support is determined by the executable profile and solving method.

[UOGTO](https://github.com/edithatogo/UOGTO) ·
[JSON-LD 1.1](https://www.w3.org/TR/json-ld11/) ·
[Shapes Constraint Language](https://www.w3.org/TR/shacl/) ·
[Paignia wire profiles](../../crates/paignia-io/README.md) ·
[Normative profile work](https://github.com/edithatogo/paignia/issues/96)

## Game diagrams

Mermaid flowchart syntax can display a finite tree or DAG, with labels for
players, choices, chance probabilities and utilities. The prototype should
derive the diagram from the admitted model so changes to a payoff or action
also change its figure. The model keeps its executable semantics in Paignia
and UOGTO. Start with the public buyer–seller example before testing
information-set annotations or other game forms.

[Mermaid flowcharts](https://mermaid.js.org/syntax/flowchart.html) ·
[Diagram export proposal](https://github.com/edithatogo/paignia/issues/95)
