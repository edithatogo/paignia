# T22 discounted recurring-funding qualification

The readout is the expected discounted payoff in a two-state, two-player
synthetic game. The exact policy values are 32/13 and -16/13 and the
initial value is 20/13. The same policy is a saddle for both states.
The [contract](../../../architecture/discounted-game-contract-v1.md)
defines the equations, limits and interpretation of each reported bound.

- The reviewed Rust implementation includes seven controls for ordered
  mass, policy and minimax evaluation, an exact subnormal reward, nonzero
  unilateral gains, bounded iteration and input rejection. The existing
  Rust compiler matrix, package isolation and offline security gate
  exercised the code on [Paignia #105](https://github.com/edithatogo/paignia/pull/105).
- UOGTO installed its semantic projector from a wheel in an isolated
  Python 3.15.2 environment. Forty focused controls, validation and the
  broader test suite passed. Its merged revision is
  `b3f1de0119ee1c1e49913d9c1f9284e053e43433`. The assertion and projection
  exercise the extension's two classes and eleven properties. [UOGTO #145](https://github.com/edithatogo/UOGTO/pull/145)
- The receiving NHRA checkout replayed both methods with an independently
  installed `paignia-io` executable from provider source
  `9f23f8a09d4657e33d641c2b13e0826939acafa9`. Both requests and both
  responses matched the native replay byte for byte on Darwin arm64.
  The maintained receipt records their hashes. Native CI repeats the
  source-bound replay in an isolated Linux environment. [NHRA #40](https://github.com/edithatogo/nhra_game/issues/40)

The [wire receipt](receipt.json) binds the UOGTO revision, RDF digest,
covered vocabulary and both request and response digests. The receiving
exercise is synthetic. Actual NHRA renegotiation retains its own method
contract and rollout criteria.
