# Software paper and independent reproduction

Use the [JOSS review checklist](https://joss.readthedocs.io/en/latest/review_checklist.html)
and [criteria](https://joss.readthedocs.io/en/latest/review_criteria.html) against a
named draft and release. Record the source revision and hashes of the manuscript,
bibliography, figures and cited computational artifacts. Carry the authorship,
contribution, conflict, funding, licence, methods, limitations and submission
dispositions in that edition's manuscript contract.

| JOSS review area | Current evidence and remaining work |
| --- | --- |
| Source, licence and support | The repository, Apache-2.0 text and contribution and security routes are public. Confirm the intended release and author contributions in the draft. |
| Scope, research need and development history | The [provider intake](../integrations/downstream-game-intake.md) names users and bounded adopters. A paper needs a grounded comparison and research impact evidence. Record public development and participation for editorial assessment. |
| Installation and examples | [Isolated package validation](game-consumer-handoff.md) and the [public UOGTO replay](../../verification/uogto_interchange/README.md) provide executable evidence. Replay both with the release edition. |
| Functionality and performance | Link each advertised claim to its exact method tests, oracle and scope. The [adaptive benchmark](credal-nash.md) includes the case where adaptive subdivision took longer. |
| Citation and archive | FAIR4RS preflight validates repository metadata. The selected release and DOI must resolve to the archived bytes cited in the draft. |
| Manuscript and editorial review | Draft selection, author approval and external review remain open. Require a statement of need, state of the field, software design, research impact and a specific AI use disclosure. Record the author and conflict checks and confer with the editors about sustained public history and engagement. |

## Inara draft check

The tracked repository starts with a reproducibility packet. Supply the actual
draft to this check when it is available. Inara 1.3.1 at
[`88ee5d6`](https://github.com/openjournals/inara/tree/88ee5d656f147d801448ce77ad8375619d383df6)
was exercised with Pandoc 3.14 on its upstream example. The run produced HTML
and well-formed JATS with five references.

In the trusted Inara checkout, using the qualified Pandoc, run:

```sh
make jats html ARTICLE=/absolute/path/to/paper.md TARGET_FOLDER=/absolute/path/to/draft-check
```

Inspect the diagnostics, title, authors, affiliations, references, figures and
headings in the generated output. On the publication environment also generate
the draft PDF with `make pdf` and inspect the rendered pages. Retain tool
versions and input and output hashes. Inara rendering is technical evidence.
The JOSS authors and reviewers assess scientific claims and release readiness.

## CODECHECK execution packet

The [public UOGTO case](../../verification/uogto_interchange/README.md) supplies
the question, exact command, immutable inputs, expected result, failure controls
and output digest receipt. A manuscript can add the segment benchmark and its
qualified package receipts when citing those results. Freeze the paper edition
and all cited code and archive identities together. A checker can download a
clean checkout, follow the replay commands and report observed outputs,
deviations and tool versions against those identities. Keep the independent
checker and any certificate linked from [T20](../../conductor/tracks/t20_release_maturity_20261124/plan.md).
Internal CI and maintainer reproduction receipts are available. A
[CODECHECK](https://codecheck.org.uk/) certificate is an independent review
deliverable.

## Maintained wrappers

For a maintained R wrapper, run `pkgcheck::pkgcheck()` and the package and
provider conformance tests in its repository. For a maintained Python wrapper,
apply the pyOpenSci review guidance and qualify the installed wheel and its
provider pin. Record the wrapper source revision, install and behavior checks
with the consumer receipts. The Paignia tree currently provides Rust packages
and verification tooling. [rOpenSci pkgcheck](https://docs.ropensci.org/pkgcheck/)
· [pyOpenSci package guide](https://www.pyopensci.org/python-package-guide/)
