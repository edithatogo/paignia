# Game evolution and upstream reuse — 22 November 2026

## Specification

UOGTO is the ontology; Paignia is the reusable Rust game-theory library. Every
new or materially changed downstream game records both semantic and provider
coverage, including reuse decisions. Reusable gaps are implemented upstream;
domain assumptions, data and valuation stay downstream. Relevant existing
upstream libraries are reused through qualified interfaces without cycles.

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
`git diff --check` passed. All 41 relative Markdown file links across the seven
changed documents resolved, and the change-scope check found no other files.
Independent documentation review found a mismatch in semantic disposition names;
the paired contracts now use the same four explicit semantic states, retaining
pending review separately from unsupported capability. Native source,
manifests, locks, fixtures, CI gates and release receipts are unchanged; no
numerical or application qualification is inferred from these checks.
