## Why

High-order computations in the workspace — SURD joint distributions, CFD flowfields, and UQ response
surfaces — hit the curse of dimensionality: a dense rank-`d` tensor needs `nᵈ` storage and is
unrepresentable past ~15 binary variables. The `deep_causality_tensor` crate already ships the linear
algebra a tensor-train (matrix-product-state / -operator) layer needs (SVD, reshape, einsum, contraction),
but has no compressed-tensor format to exploit it. A one-dimensional tensor-network module gives all three
consumers a single, precision-generic, statically-dispatched primitive set for compress → operate →
recompress, with the design already locked in `openspec/notes/tensor-network/Tensor-Network-Spec.md`.

## What Changes

- Add a **tensor-network module** inside `deep_causality_tensor` (`src/types/causal_tensor_network/`,
  traits in `src/traits/`) — **not** a new crate.
- Introduce the **tensor-train (MPS) state** `CausalTensorTrain<T>` + trait `TensorTrain<T>`, and the
  **matrix-product-operator (MPO)** `CausalTensorTrainOperator<T>` + trait `TensorTrainOperator<T>`.
- Add the **Stage-0 numerical foundation**: a robust truncated thin-SVD and Householder QR (additions to
  the existing `ops/tensor_svd*`, not a rewrite of the public `svd`), and a `Truncation<T>` policy type.
- Provide **TT-cross** construction (build a TT from an oracle without going dense) and an **ALS/DMRG
  sweep engine** (linear solve, fit/completion, integrate, eigensolve).
- Compose operators through the existing **`EndoArrow<CausalTensorTrain<T>>`** algebra; expose a storage
  **HKT witness** (`Functor`/`Foldable`/`Pure` only) and the **`Module`/`AddGroup`/`Ring`** algebra
  impls, mirroring `CausalTensor`.
- Keep **precision a parameter** at every stage (tested at `f32`/`f64`/`Float106`) and make **complex
  (`Complex<T>`) and forward-mode-AD (`Dual<T>`) scalars** first-class via the crate-native `Normed` /
  `Scalar` bounds. Reverse-mode AD is out of scope.
- Add a **criterion benchmark suite** under `benches/benchmarks/causal_tensor_network_type/`.
- No breaking changes: this is purely additive. Existing `CausalTensor` behavior is untouched.

## Capabilities

### New Capabilities
- `tensor-network-numerics`: Stage-0 foundation — robust truncated thin-SVD, Householder QR,
  `Truncation<T>` policy, the precision invariant, and the layered scalar bounds (`Normed` for the
  norm/SVD layer, `Scalar` for the algebra/AD layer).
- `tensor-train`: the MPS/TT state — `CausalTensorTrain<T>` + `TensorTrain<T>` trait; TT-SVD and other
  constructors, canonical forms, rounding, norm/inner, `add`/`scale`/`add_scalar`/`hadamard`,
  `marginalize`, `eval`, `to_dense`, QTT reshape; the `Module`/`AddGroup`/`Ring` algebra impls and the
  storage HKT witness (`Functor`/`Foldable`/`Pure`).
- `tensor-train-operator`: the MPO — `CausalTensorTrainOperator<T>` + `TensorTrainOperator<T>` trait;
  `from_dense`, `apply` (MPO·MPS), `compose` (MPO·MPO), operator rounding, and the
  `EndoArrow<CausalTensorTrain<T>>` composition.
- `tensor-train-cross`: TT-cross / ACA construction of a TT from an oracle closure without forming the
  dense tensor (`CrossConfig<T>`), plus the `apply_nonlinear` elementwise re-approximation built on it.
- `tensor-train-solve`: the alternating-sweep (ALS / DMRG) engine — `linear` solve, `fit`/completion,
  `integrate`, DMRG `eigen`, and optional TDVP step (`SolveConfig<T>`).
- `tensor-train-scalars`: scalar generality — complex-aware SVD/QR and the `Complex<T>` instantiation,
  the `Dual<T>` forward-mode-AD instantiation via the relaxed bound, and the mixed `Dual<Complex<T>>`
  case, with the precision/AD test matrix.

### Modified Capabilities
<!-- None. The truncated SVD and QR are net-new additions to deep_causality_tensor; no existing spec's
     requirements change. This change is purely additive. -->

## Impact

- **Crate:** `deep_causality_tensor` only — new modules under `src/traits/`,
  `src/types/causal_tensor_network/`, `src/extensions/ext_hkt_tensor_train.rs`, tests mirroring the tree,
  and benches under `benches/benchmarks/causal_tensor_network_type/`. Bazel `BUILD.bazel` test/bench
  registration updated.
- **Dependencies:** uses existing internal deps only (`deep_causality_num` for `RealField`/`Real`/
  `Field`/`Complex`/`Dual`/`Normed`/`Scalar`, `deep_causality_haft` for the HKT/`Arrow` traits). No new
  external runtime crates; `criterion` (already a dev-dep) for benches.
- **Downstream (future, not in this change):** `deep_causality_algorithms` (SURD), `deep_causality_cfd`,
  and `deep_causality_uncertain` become able to consume the format; no changes to them here.
- **Error surface:** extends the existing `CausalTensorError` enum with new variants
  (`BondDimensionMismatch`, `NotCanonical`, `RankExceeded`, `SweepDidNotConverge`, `CrossSampleFailure`);
  no second error type introduced.
- **Policy compliance:** static dispatch only (no `dyn`), `unsafe_code = "forbid"`, no lib-code macros,
  no concrete float literals (precision-generic), 100 % test coverage of added code.
