## 1. Stage 0 — Numerical foundation (`tensor-network-numerics`)

- [x] 1.1 Add the `Truncation<T>` type under `src/types/causal_tensor_network/truncation/mod.rs` with `by_bond`/`by_tol`/`new` constructors and `InvalidParameter` rejection of bad policies
- [x] 1.2 Implement robust truncated thin-SVD (`svd_truncated`) as an addition under `ops/tensor_svd_truncated/`, returning rank-truncated orthonormal `U`/`Vt`; leave the existing public `svd` unchanged. (Implemented via **one-sided Jacobi**, chosen over Golub–Kahan for higher relative accuracy with a simpler, branch-stable kernel — documented at the call site.)
- [x] 1.3 Implement Householder `qr` returning orthonormal `Q` and upper-triangular `R`
- [x] 1.4 Apply the layered scalar-bound discipline: SVD/QR kernels bound on **`Scalar`** (`Real + Div + FromPrimitive`, admits `Dual` for Stage-4 AD with no body change), `Truncation` on **`Real`**; all magnitude comparisons via `.abs()` on the real scalar; every division guarded by a checked-nonzero pivot; no concrete float literal in lib code. (The complex `Normed`+`ComplexField` Hermitian kernel is Stage 4; the real kernel deliberately carries the minimal `Scalar` bound so that path is instantiation-only.)
- [x] 1.5 Reference numerics validated against embedded known values (diagonal singular values, golden-ratio 2×2) plus orthogonality residuals `‖UᵀU − I‖ ≤ √ε·16` and `Q R = A` / `U S Vt = A` reconstruction, all at `f32`, `f64`, `Float106`
- [x] 1.6 Register new test files in their `mod.rs` chain and the `types` `rust_test_suite` glob in `tests/BUILD.bazel`; `cargo test -p deep_causality_tensor` green (36 integration + 381 lib + 25 doctests), clippy clean, formatted

## 2. Stage 1 — Tensor-train state (`tensor-train`)

- [ ] 2.1 Add `CanonicalForm` enum (`canonical_form/mod.rs`) and the `TensorTrain<T>` trait in `src/traits/tensor_train.rs`
- [ ] 2.2 Add `CausalTensorTrain<T>` struct + private fields (`causal_tensor_train/mod.rs`) and getters (`cores`/`order`/`phys_dims`/`bond_dims`/`max_bond`/`canonical_form`)
- [ ] 2.3 Extend `CausalTensorError` with `BondDimensionMismatch`, `NotCanonical`, `RankExceeded` (+ `Display` arms)
- [ ] 2.4 Implement `from_dense` (TT-SVD left-to-right sweep), and `from_fn`/`to_dense` with the `RankExceeded` element-count guard; add `zeros`/`ones`/`random_seeded`
- [ ] 2.5 Implement QR-based `left_canonicalize`/`right_canonicalize`/`canonicalize_at` updating the tracked `CanonicalForm` (in-place via `get_mut`)
- [ ] 2.6 Implement `round(&Truncation)` (SVD recompression, idempotent)
- [ ] 2.7 Implement `norm`/`inner` via the mixed-canonical center
- [ ] 2.8 Implement exact `add`/`scale`/`add_scalar`/`hadamard` and their `*_rounded(&Truncation)` variants
- [ ] 2.9 Implement `marginalize` and `eval`
- [ ] 2.10 Implement QTT `quantize`/`dequantize` reshape helpers (`ops/qtt.rs`)
- [ ] 2.11 Implement the algebra-trait impls — `Module<T>` (exact `scale`), `AbelianGroup`/`AddGroup` (via `add`), `Ring` (via `hadamard`) — via marker traits + num blanket impls; do NOT implement `Field`/`InvMonoid`; document the lax + shape-dependent-`Zero`/`One` caveats
- [ ] 2.12 Add `CausalTensorTrainWitness` (`extensions/ext_hkt_tensor_train.rs`) implementing `Functor`/`Foldable`/`Pure` over core storage only
- [ ] 2.13 Wire `lib.rs` re-exports (`TensorTrain`, `CausalTensorTrain`, `Truncation`, `CanonicalForm`, `CausalTensorTrainWitness`)
- [ ] 2.14 Write mirror tests (round-trip, compression accuracy, algebra laws vs dense, marginalize vs `sum_axes`, eval, functor laws, error paths) at `f32`/`f64`/`Float106`; register in `mod.rs` + `tests/BUILD.bazel`

## 3. Stage 2a — Matrix-product operator (`tensor-train-operator`)

- [ ] 3.1 Add the `TensorTrainOperator<T>` trait (`src/traits/tensor_train_operator.rs`) and `CausalTensorTrainOperator<T>` type with rank-4 cores + getters
- [ ] 3.2 Implement inherent constructors `identity(dims)` and `from_dense` (operator TT-SVD)
- [ ] 3.3 Implement `apply` (MPO·MPS), `compose` (MPO·MPO), `adjoint`, and operator `round`
- [ ] 3.4 Implement `EndoArrow<CausalTensorTrain<T>>` (`compose` = `>>>`, `identity` = `Id`, `apply` = action)
- [ ] 3.5 Re-export operator types from `lib.rs`
- [ ] 3.6 Tests: `apply` vs dense matrix–vector, `compose` vs dense matmul, identity action, EndoArrow associativity/identity/action laws to tolerance; register in `mod.rs` + Bazel

## 4. Stage 2b — TT-cross and nonlinear maps (`tensor-train-cross`)

- [ ] 4.1 Add `CrossConfig<T>` (`cross_config/mod.rs`: `max_sweeps`, `rank_cap`, tolerance)
- [ ] 4.2 Extend `CausalTensorError` with `CrossSampleFailure`
- [ ] 4.3 Implement `CausalTensorTrain::cross` (ACA/maxvol pivoting) building a train from an oracle without going dense, returning a residual estimate, bounded by the config, and failing on non-finite samples
- [ ] 4.4 Implement `apply_nonlinear(f, &CrossConfig) -> (Self, residual)` on the `TensorTrain` trait; confirm no `map_elementwise` is exposed
- [ ] 4.5 Tests: recover a known low-rank oracle + residual, no dense allocation, budget respected, `CrossSampleFailure`, `apply_nonlinear` residual; register in `mod.rs` + Bazel

## 5. Stage 2c/3 — Solve engine (`tensor-train-solve`)

- [ ] 5.1 Add `SolveConfig<T>` (`solve_config/mod.rs`) and extend `CausalTensorError` with `SweepDidNotConverge`
- [ ] 5.2 Implement the shared alternating one-/two-site sweep driver (`solve/mod.rs`)
- [ ] 5.3 Implement `solve::linear` (ALS `A x = b`) and `solve::fit` (TT regression/completion)
- [ ] 5.4 Implement `integrate` on the `TensorTrain` trait (per-site weight contraction)
- [ ] 5.5 Implement `solve::eigen` (two-site DMRG ground state) on the shared driver
- [ ] 5.6 (Optional, if a consumer exists) implement `solve::tdvp_step`
- [ ] 5.7 Re-export `solve::{linear,fit,eigen}` and `SolveConfig` from `lib.rs`
- [ ] 5.8 Tests: linear recovers known `x`, fit recovers known train, integrate vs dense, eigen recovers known eigenpair, `SweepDidNotConverge`; register in `mod.rs` + Bazel

## 6. Stage 4 — Scalar generality (`tensor-train-scalars`)

- [ ] 6.1 Generalize the SVD/QR/inner kernels to the complex case (Hermitian transpose, real singular values, `⟨a|b⟩ = Σ aᵢ* bᵢ`) via `ComplexField` conjugation
- [ ] 6.2 Instantiate and test the whole stack at `Complex<f64>` (unitarity, conjugated inner, round-trips)
- [ ] 6.3 Confirm the algebra/AD path is bound on `Scalar` (not `Field`); instantiate and test at `Dual<f64>` for forward-mode AD (gradient vs central finite differences to `√ε`; truncation decisions on the real part)
- [ ] 6.4 Test the mixed `Dual<Complex<f64>>` case (value channel vs plain complex; finite derivative channel)
- [ ] 6.5 Ensure the generic test bodies are instantiated across the full precision/scalar matrix (`f32`/`f64`/`Float106` + `Complex<f64>`/`Dual<f64>`)

## 7. Benchmarks and finalization

- [ ] 7.1 Add `bench_svd_qr.rs` (Stage 0) and `bench_tensor_train_core.rs` (Stage 1) under `benches/benchmarks/causal_tensor_network_type/`, registered in its `mod.rs` and the `criterion_main!` set; verify the `r³` scaling rows
- [ ] 7.2 Add `bench_tensor_train_operator.rs`, `bench_tensor_train_cross.rs`, `bench_tensor_train_solve.rs` as their stages land
- [ ] 7.3 Confirm each §7 complexity target has a matching benchmark; ensure the full suite runs in seconds
- [ ] 7.4 Run `make format && make fix`; confirm `unsafe_code = "forbid"`, no `dyn`, no lib-code macros, no concrete float literals
- [ ] 7.5 Run `cargo test -p deep_causality_tensor` and confirm 100 % coverage of added code; update `BUILD.bazel` files
- [ ] 7.6 Run `openspec validate add-tensor-network` and reconcile any spec drift before `/opsx:apply` completion
