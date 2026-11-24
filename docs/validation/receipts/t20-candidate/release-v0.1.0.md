# Paignia 0.1.0 distribution verification

Verified 25 November 2026. The signed annotated tag `v0.1.0` points to
`a04a19da06cc50e310a172574ed66f62000b891a` and tree
`056db7c18014b5f760d289d672d171a7b4b14e56`. GitHub independently reports
the tag signature as valid for tag object
`838ca7d04d7807c10a23f9c6fbd6cd7e8a9d825f`. The tag and signed checksum
manifest use OpenPGP fingerprint `BB03C82343A653EE44BD5CDA9DF6B142F065199E`.
[GitHub release](https://github.com/edithatogo/paignia/releases/tag/v0.1.0)

The DOI resolves to the public Zenodo record for version 0.1.0 and DataCite
reports it as findable. All eight files were downloaded from Zenodo and
GitHub and compared byte by byte. The detached signature and the six file
checks covered by its signed manifest verified in both copies. The two
Cargo packages and qualification JSON also verified against the archived
Sigstore attestations from the GitHub-hosted Native CI workflow on the exact
main commit. [Version DOI](https://doi.org/10.5281/zenodo.24515285) ·
[DataCite registration](https://api.datacite.org/dois/10.5281/zenodo.24515285) ·
[Candidate CI](https://github.com/edithatogo/paignia/actions/runs/45438493484) ·
[Candidate Offline AI Security](https://github.com/edithatogo/paignia/actions/runs/45438494033)

| Qualified artifact | SHA-256 |
| --- | --- |
| Source archive | `28daca5363a8e8c09788c4d0d6d68b20183f21be4bea1d6818342af30b3af21c` |
| `paignia-core-0.1.0.crate` | `e917b650bc7d008d4d0968dc6b0a4e04b1d9f6e0ae1c79818563fba111fac973` |
| `paignia-solve-0.1.0.crate` | `638fb6c971e11750cee3b2c68cbc4f9ffa7a7d710b149dbbc58c8bde627fa7fc` |
| Cargo package qualification | `c47e1d1f7c0ce8f69daf3544301c22bff97fd4109c1aab499a8d64bd8a86cdb0` |

The [candidate tests and application receipts](README.md) establish
installability for those same crate bytes and the bounded finite
perfect-information profile. Broader methods retain their separate
contracts. Distribution uses the signed Git tag and archived files. Cargo
registry publication remains a separate project decision.
