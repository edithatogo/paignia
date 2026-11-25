---
title: Release and reference
description: Find the signed release, Rust APIs, evidence, and contribution guides.
---

- [Signed Paignia 0.1.0 release](https://github.com/edithatogo/paignia/releases/tag/v0.1.0)
- [Archived software and DOI](https://doi.org/10.5281/zenodo.24515285)
- [Citation metadata](https://github.com/edithatogo/paignia/blob/main/CITATION.cff)
- [Application compatibility and downstream intake](https://github.com/edithatogo/paignia/blob/main/docs/integrations/downstream-game-intake.md)
- [Core comparator results](https://github.com/edithatogo/paignia/blob/main/docs/validation/receipts/t21-core/README.md)
- [Contribution and maintenance](https://github.com/edithatogo/paignia/blob/main/CONTRIBUTING.md)

Generate the native Rust API documentation from the selected revision:

```sh
cargo doc --workspace --no-deps --all-features --locked --open
```
