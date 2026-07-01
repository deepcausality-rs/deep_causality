## Why

`CausalTensorTrainOperator` already supports operator composition (`compose`, the product) and a
multiplicative identity (`identity`), but has **no additive structure** — making it an incomplete
algebra. Assembling finite-difference / spectral MPO operators from primitives — a centered derivative
`(S₊ − S₋)/2Δx`, a Laplacian `(S₊ + S₋ − 2·I)/Δx²` from grid-shift operators — needs operator sum,
difference, and scalar multiple. This is the foundational primitive for the planned CFD ↔ tensor-network
(QTT) bridge.

## What Changes

- Add public `add`, `sub`, `neg`, and `scale` to `CausalTensorTrainOperator`, completing the operator
  algebra alongside `compose` / `identity`. Implemented by delegating to the existing combined-index
  train view (`as_combined_train` / `from_combined_train` + `CausalTensorTrain::add` / `scale`).
- `add` / `sub` require matching input/output dimensions; their bond dimensions add (caller rounds to
  recompress). `scale` / `neg` are rank-preserving.
- Purely additive; no breaking changes, defaults untouched.

## Capabilities

### Modified Capabilities
- `tensor-train-operator`: gains the additive operator algebra (`add` / `sub` / `neg` / `scale`),
  completing the ring/module structure already begun by `compose` and `identity`.
