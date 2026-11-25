---
title: Replay a model across RDF and JSON
description: Preserve the source model, ordered legal choices and numerical results.
---

Begin with the buyer–seller Turtle graph and the frozen solver request.
Turtle and JSON-LD encode an RDF graph. The Paignia version 1.0.0 request is a
typed JSON projection of that graph.

```sh
uv venv --python 3.15.2 target/dev-python
uv pip install --python target/dev-python/bin/python --require-hashes \
  -r verification/uogto_interchange/requirements.txt
target/dev-python/bin/python -I verification/uogto_interchange/interchange.py \
  stage --ontology-dir target/uogto-bundle
target/dev-python/bin/python -I verification/uogto_interchange/verify.py \
  --ontology-dir target/uogto-bundle/ontologies/core --output target/uogto-interchange
target/dev-python/bin/python -I verification/uogto_interchange/verify_profiles.py \
  --ontology-dir target/uogto-bundle --output target/native-profiles
```

Keep the graph bytes and the receipt. Check the result's `source` against
the input binding. Replaying the other profiles returns the Kuhn policies,
the Prisoner's Dilemma pure equilibrium and the matching-pennies mixed
witness.

[Normative mapping](https://github.com/edithatogo/paignia/blob/main/docs/architecture/interchange-profile-v1.md) ·
[UOGTO external catalogue](https://github.com/edithatogo/UOGTO/blob/main/docs/contracts/external-game-catalogues-v1.md)
