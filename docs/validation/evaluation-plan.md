# Evaluation plan for Paignia and downstream models

Protocol 1 · 25 November 2026

## Questions and outcome measures

1. Does the native core preserve admitted game semantics, return valid policies
   and residuals, and reject out-of-profile inputs reproducibly?
2. On games Paignia, Gambit and OpenSpiel can represent equivalently, how do
   correctness, completion rates, time and peak memory scale? Report conversion
   and solve costs separately and together. Establish a method-specific error
   bound or target exploitability before timing.
3. Does substituting Paignia preserve each admitted application's decision,
   payoff and diagnostic meaning over its supported domain? Measure which
   models can be replaced, with the oracle and reason for each gap.

Pin the source, package digest, semantic profile, comparator version,
compiler, architecture, dataset and seed for every run. Record objective,
support and scale bounds, tolerances, timeout and error definitions before
running comparisons. Keep every seeded case and all observed errors. Build
release configurations once per tool, interleave repeated measurements on the
same host and report medians and dispersion with the cost of translation. Use
fixed resources, measured cold and warm runs, and repeat on a second qualified
architecture before making portability or performance claims.

## Core and method gates

| Family | Correctness evidence | Comparative test |
| --- | --- | --- |
| Game admission and exchange | Generated finite graphs, deep and shared paths, tied actions, missing or duplicate identifiers, invalid chance mass and bad RDF/JSON values. Require decision/utility identities across conversion. Reuse T19 wire, shape, and witness suites. | Stratify by tree size, depth, branch factor, chance fraction and sharing. Separate translation rejections from solver failures and reproduce the fixed reference hashes. |
| Backward induction | Exact rational hand oracles on small games and independent direct traversal. Include zero-reach contingencies, cancellation, zero chance weights, ties and multi-player utility order. | Compare selected policies, expected values and optimal action sets on overlap. Translate DAGs to explicit trees for tools that require trees, preserve state identities, and measure the unfolding cost. |
| Pure and bounded mixed Nash | Exhaustive exact small normal-form games. Verify probability mass, support consistency, deviation gain, and the expected scope when support is non-square, singular or degenerate. | Use relevant Gambit exact methods and small hand games. Evaluate matched solution sets where both scopes permit enumeration and report verified witnesses separately. |
| Perfect-recall CFR and CFR+ | Existing independent Kuhn and Leduc oracles, explicit information-set admission, utility and deviation checks, and malformed perfect-recall cases. | Measure OpenSpiel CFR on equivalent games with the same iteration accounting and averaging convention. Compare NashConv or exploitability by both time and iterations. |
| Correlated equilibrium, commitment and bounded dynamics | Check original-unit incentive residuals and primal/dual certificates, enumerate finite commitment cases and test tie rules. Check integration against analytical or high-precision trajectories and step refinement. Keep the existing paired-segment oracle and its degeneracies. | Use independent linear or exact enumeration methods where interfaces match. Report criterion- and scale-specific error, failure and cost. |

Use the existing [method contracts](../architecture/game-theory-tool-comparison.md)
to choose the threshold for each family. Keep the numerical thresholds
independent of the performance comparisons. A reproducible discrepancy gets a
minimal fixture and explicit disposition. A benchmark claim requires passing
the same accuracy criterion over declared, versioned workloads with retained
raw runs. Higher-dimensional Nash feasibility remains in its separate research
leaf.

## Application and model gates

Use one row **per model and production caller** in the linked repository
issues. Include a hand or independent oracle, the current implementation, the
candidate dependency, the semantic map, test cases, approval scope, and the
source and output hashes. Freeze a baseline and a stress case before changing
a caller. Preserve local calibration, data rights, payoff definitions and
validation obligations. Treat reference-only and teaching solvers as named
oracles when useful.

| Repository | First comparison and success criterion |
| --- | --- |
| New Drug Reimbursement Game | Reuse Chapter 8 finite-offer evidence. Check selected offers, firm rent and net health benefit against the chapter calculation. Add each remaining chapter model with its own assumptions and price, capacity and discount stress cases. |
| Innovate | Reuse the accepted sequential-pricing case. Compare the selected commitment, demand and payoffs using the application oracle. Inventory other pricing, diffusion and substitution callers by method. |
| NHRA Game | Enumerate normal-form, sequential and alternating-offer cases. Compare QRE to a matching logit reference; check discount, bargaining and commitment boundaries against derived results. |
| GINSIM | Replay the screening, proxy-information and compliance cases with distinct information structures. Check feasible policy, utilities and decision changes under stipulated assumptions. |
| GTPCNZ | Map demonstrative and production funding models separately. Check actions and calibrated payoffs against the funding model. Establish a deviation bound where a Nash result is claimed. |
| Health game tutorial | Reproduce the existing finite coordination example, then verify each runnable lesson against a hand result and pinned dependency. Keep the edition tied to its lesson. |
| Tobac | Continue in the existing application lane. Qualify the published/synthetic inspection oracle, commitment ties and signed lineage against the applicable method and empirical release scope. |

A migration closes after the matched caller passes its ordinary locked checks,
the selected archived release or revision is proven, duplicate generic
production logic is gone, and rollback remains a qualified version. Preserve
failed or unsupported cases with the owning issue. The comparison cohort and
all seven children are tracked in [Paignia #97](https://github.com/edithatogo/paignia/issues/97).

## Findings and acceptance

Publish a results table with accepted domains, input counts, reason-coded
failures, policy and utility errors, numerical residuals, elapsed time and peak
memory. Retain paired raw outputs and reproducible commands. Present the
benefit of identity-preserving interchange and verified reuse separately from
the timing measurements. Independent numerical and application review should
bind findings to the exact package and document any unresolved discrepancies.
Paper claims require the relevant run and review evidence.
