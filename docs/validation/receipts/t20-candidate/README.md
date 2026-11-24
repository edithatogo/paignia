# Paignia 0.1.0 candidate qualification

The candidate is commit `a04a19da06cc50e310a172574ed66f62000b891a`. Native CI
qualified the Rust 1.75 package archives with the provider checkout unavailable.
GitHub CLI verified the SLSA provenance of both archives and the package
receipt against the exact main commit, the Native CI workflow and a GitHub-hosted
runner. [Native CI and archive artifact](https://github.com/edithatogo/paignia/actions/runs/45438493484)
· [Attestation verification record](provenance.json)
· [Offline AI Security](https://github.com/edithatogo/paignia/actions/runs/45438494033)

## Application results

Both tests used the same attested `paignia-core` and `paignia-solve` archives
with default features disabled. They ran on Linux x64 using Apple Container
and Rosetta with Rust 1.100.0. The containers had read-only inputs, no network
route, no provider source checkout and no Cargo Git cache. The applications'
original dependency lock entries were preserved apart from the two provider
source bindings. Resolved metadata identifies both package paths inside each
container.

| Application case | Qualified revision | Result |
| --- | --- | --- |
| Reimbursement Chapter 8 finite offers | [`8e7b067`](https://github.com/edithatogo/new-drug-reimbursement-game/commit/8e7b0674b234521e97abd7e65b655892fb646636) | Six passed. The independent Chapter 8 oracle selects the 16,000 offer in the finite menu with rent 160,000 and net economic benefit in health of 2/3. [Receipt](reimbursement.json) |
| Innovate sequential pricing | [`7999fdd`](https://github.com/edithatogo/innovate/commit/7999fddec28fea30e93c67c4e15857c42c884616) | Five passed. The independent demand calculation selects the premium commitment with net payoffs 27.5 and 29.5. This is the frozen case used for T17. [Receipt](innovate.json) |

Run the provider's evidence guard with the repository-local Python to validate
the archive and application receipt hashes and their common static profile.

```sh
target/dev-python/bin/python -I scripts/validate_game_consumer_evidence.py \
  --manifest docs/validation/receipts/t20-candidate/manifest.json
```

## Citable release

The package version and draft citation metadata have been verified for
0.1.0. The public release still needs a verified signed `v0.1.0` tag for
the candidate commit and a versioned DOI that resolves to a deposit carrying
the verified archive bytes. Version and identifier checks must use the
published record and the downloaded files. [Zenodo DOI workflow](https://help.zenodo.org/docs/deposit/describe-records/reserve-doi/)
