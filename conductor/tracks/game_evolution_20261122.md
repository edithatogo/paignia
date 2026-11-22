# Game evolution and upstream reuse — 22 November 2026

## Specification

UOGTO is the ontology; Paignia is the reusable Rust game-theory library. Every
new or materially changed downstream game records both semantic and provider
coverage, including reuse decisions. Reusable gaps are implemented upstream;
domain assumptions, data and valuation stay downstream. Relevant existing
upstream libraries are reused through qualified interfaces without cycles.
The scope is the six applications named in the intake contract. Innovate is a
parallel innovation library with gradual generic-game migration. Voiage owns
VoP/VoH methods and `vop_poc_nz` is their intended demonstration; archives retain source and
provenance responsibilities. Tutorial adoption proceeds in versioned lessons.

This is a contribution-workflow increment. It does not implement or qualify
the candidate methods in the intake table or change T17/T18 acceptance.

## Plan and disposition

- [x] Reconcile current ownership, manifests, T17/T18 and downstream examples.
- [x] Publish the [intake and reuse contract](../../docs/integrations/downstream-game-intake.md)
  with shared UOGTO keys and concrete application leads.
- [x] Link the contract from agent/contributor guidance, README and PR template.
- [x] Check the focused documentation diff and referenced local paths.

The paired UOGTO policy belongs to its existing semantic-game-contract track.
Implementation requests retain separate owners and acceptance: NHRA bargaining
and QRE need assessment; GINSIM information/compliance models need classification;
existing finite-game applications reuse accepted provider contracts where they
fit. All open T17.10/T17.12 and T18.03 obligations remain open.

## Verification scope

Documentation changes only, based on `3573ec7af6239be845f04c4a5ec7831198e09e92`.
Focused verification covers whitespace, relative Markdown file links and the
explicit eight-document allowlist; results are recorded in the pull request.
Independent documentation review found a mismatch in semantic disposition names;
the paired contracts now use the same four explicit semantic states, retaining
pending review separately from unsupported capability. Native source,
manifests, locks, fixtures, CI gates and release receipts are unchanged; no
numerical or application qualification is inferred from these checks.

## Executed tutorial reuse — 22 November 2026

Recorded `tutorial_health_gt` edition 0.2 under `finite-perfect-information`,
with the same case identity and semantic disposition linked from UOGTO. The
consumer at `3851a2346c8e9cc457c7bfee67848be13fc8ca46` uses the existing pinned
core/solver and separately pinned declaration validator. Its local test checks
the known answer, a payoff contrast and an invalid graph; its locked run returns
the hand-derived `(3, 2)` utilities. Exact source/lock hashes and toolchain are
in the intake case note. No new method or runtime dependency was needed.

This upstream documentation records actual consumer evidence and preserves the
distinction between semantic declaration validation and native solving. Hosted
tutorial jobs have no executed steps and are not treated as passing. A public
upstream fixture and full RDF mapping remain open; T17/T18 receipts and acceptance
statuses are unchanged. Scoped link/hash and whitespace checks cover this note;
native source, dependencies and fixtures in Paignia are unchanged.
