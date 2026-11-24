# Native static solver result emission

`paignia_solve::backward_induction_result` is an additive, opt-in entry point.
It receives a `GameSpecification`, exact semantic-profile string, explicit
`ResultIdentityRecord` and `SolverConfig`, then returns a computed
`BackwardInductionResult` or a separate typed `ResultSolveError`.

The implementation validates the input and supplied role-specific identities,
compares the specification ID, executes the existing Rust solver, constructs
the [native result envelope](game-result-contract-v1.md), and validates the
generated record against the source graph. It never calls another engine.
The existing solver API signatures and typed error boundaries are preserved.

The method identifies itself as `paignia/backward-induction`, version `1.0.1`.
Each chance node accumulates probability-weighted payoffs with an addition
correction per player. Products and returned values still use binary64.
Its selection rule is the seed's input-order pairwise tolerance followed by a
lexical tie comparison. Near-tie chains can depend on edge order. This is a
tolerance-aware contingent policy, not a certificate of exact subgame-perfect
equilibrium or all equilibria. A different selection rule needs a new version.

The result keeps the hypothetical solver traversal in `diagnostic_trace`,
separate from the envelope's policy/payoffs and from visit/tie counters in
`solver_diagnostics`. No causal execution, session, live policy, residual,
exploitability or convergence measurement is invented. Missing caller context
stays absent. Static instance/run identities remain caller-assigned references.

Input, identity, identity-mismatch, solver and output-contract failures remain
distinct. Budget exhaustion is an error, not a zero payoff or a claim that no
equilibrium exists. Non-finite generated payoffs reject even when the legacy
input profile admits all finite values. A regression uses an admitted
slightly-over-unit chance weight multiplied by `f64::MAX` to distinguish input
admission from valid computed output.

This API supplies computed native records; it defines no serialization format,
general equilibrium catalogue, conversion receipt or application acceptance.
The public regression target is `cargo test --locked -p paignia-solve --test
result_contract`. Its expectations independently fix the schema/selection scope,
typed failure boundaries and agreement with the current native solver.
