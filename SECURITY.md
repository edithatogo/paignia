# Paignia security policy

Paignia is a pre-release library. Security fixes target the current main branch.
Applications pinned to a Git revision should compare their pin with the fix and
rerun their qualified game and dependency checks before upgrading. The version
and optional-feature boundaries are described in the [package documentation](README.md#packages-and-current-implementation).

Report a security vulnerability through [GitHub private reporting](https://github.com/edithatogo/paignia/security/advisories/new).
The [account security policy](https://github.com/edithatogo/.github/blob/main/SECURITY.md)
governs disclosure and provides an alternative contact route.

Include the provider commit or package version, Rust toolchain, enabled features,
the affected API and a minimal synthetic reproduction. Report privately when
an untrusted game, dependency or integration can cross a trust boundary, exhaust
resources in a service, or disclose data. Use a [regular issue](https://github.com/edithatogo/paignia/issues/new/choose)
for ordinary solver discrepancies. Use synthetic payoffs and identifiers in
either report.
