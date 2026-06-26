<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Tensor-Network implementation status (working note)

Living status for the `add-tensor-network` OpenSpec change. Spec: `Tensor-Network-Spec.md` (same
folder). Change dir: `openspec/changes/add-tensor-network/`. All code lives in **`deep_causality_tensor`**
(module, not a separate crate).

## Git state (as of this note)

Committed on `main` (one commit per stage):
- `d9240b802` Stage 0 — truncated SVD (one-sided Jacobi), Householder QR, `Truncation`.
- `adbd02e7d` Stage 1 — `CausalTensorTrain` (MPS) state.
- `8ece81e82` Stage 2a — `CausalTensorTrainOperator` (MPO).
- `e4c5a1947` Stage 2b — TT-cross + `apply_nonlinear`.
- `b0b875b1d` Stage 2c — ALS solve engine (`SolveConfig`, `integrate`, `solve::fit`, `solve::linear`).

**Uncommitted:** `solve/local.rs` — `solve::linear` rewritten from normal-equations ALS to **AMEn**
(rank-adaptive residual enrichment; direct solve for square `A`, normal-equations fallback for
rectangular). All 9 solve tests pass (f64/Float106). **Not yet committed; needs its own commit.**

Test counts: ~470 in the TT integration suite + lib + 25 doctests, green at f32/f64/Float106 (where the
op admits it). Workspace builds. Commit discipline: NEVER commit (AGENTS golden rule) — prepare message,
user commits. After each stage: draft commit message, user reviews/commits.

## What exists (per stage)

- **Stage 0** `src/types/causal_tensor/ops/`: `svd_truncated` (one-sided Jacobi — robust, orthonormal
  factors), `qr` (Householder), `tensor_qtt` (quantize/merge reshape). `Truncation<T>` in
  `causal_tensor_network/truncation/`.
- **Stage 1** `causal_tensor_network/causal_tensor_train/`: type `CausalTensorTrain<T>` + trait
  `TensorTrain<T>` (`src/traits/tensor_train.rs`). from_dense (TT-SVD), from_fn, to_dense (guarded),
  zeros/ones/random_seeded, canonical (left/right/at via QR), round, norm/inner, add/scale/add_scalar/
  hadamard (+rounded), marginalize, eval, integrate. Algebra tower `Zero`/`One`/`Module`/`AddGroup`/`Ring`
  with a **total shape-polymorphic identity** (order-0 absorbing 0/1). HKT witness
  `CausalTensorTrainWitness` (Functor/Foldable/Pure storage-functor only). `CanonicalForm` enum.
- **Stage 2a** `causal_tensor_train_operator/`: type `CausalTensorTrainOperator<T>` + trait
  `TensorTrainOperator<T>`. identity/from_dense/from_cores, apply (MPO·MPS), compose (MPO·MPO),
  transpose, round, to_dense. `Arrow` impl → blanket `EndoArrow<CausalTensorTrain<T>>`.
- **Stage 2b** `causal_tensor_train/construct/cross.rs`: `cross` (maxvol/LU-pivot TT-cross, rank-adaptive,
  no dense), `CrossConfig`, `apply_nonlinear` (on the trait). Error `CrossSampleFailure`.
- **Stage 2c** `causal_tensor_network/solve/`: `SolveConfig`, `solve::fit` (ALS TT-completion,
  block-diagonal local LLS), `solve::linear` (AMEn, uncommitted), environment helpers. Error
  `SweepDidNotConverge`. Re-exported as `deep_causality_tensor::solve`.

Scalar bounds: norm/SVD layer → `Normed`; algebra/AD layer → `Scalar` (= `Real + Div + FromPrimitive`,
admits `Dual`); never gratuitous `Field`. Precision is a parameter everywhere; tested at f32/f64/Float106.

## Hard-won learnings (do not re-learn)

1. **`CausalTensor::permute_axes` is a STRIDED VIEW** — `as_slice()` returns data in original (un-permuted)
   physical order. Any transpose/reorder that then reads `as_slice()` MUST physically rebuild the buffer
   (see MPO `transpose`). This is a silent-corruption trap.
2. **`CausalTensor::matmul`/`sum_axes` require `T: Default`** — excludes `Dual`. The TT layer uses its own
   `Scalar`-bound `linalg::matmul` / `solve_dense` / `invert_square` instead.
3. **Rank-revealing pivots need a scale-relative threshold**, not exact-zero. Float elimination residuals
   are ~`max|·|·ε`; an exact `>0` test inflates every bond to the rank cap (cross bug: `[4,6,4]` vs
   `[2,2,2]`). Threshold used: `max_abs · ε · 64`.
4. **Ridge floors achievable accuracy** in ALS/fit; must be precision-scaled (`T::epsilon()`) for the
   tighter Float106 target, else `SweepDidNotConverge`.
5. **num `Zero`/`One` are parameterless** → a shape-carrying TT needs a *total absorbing identity*
   (order-0 empty train), mirroring `CausalTensor`'s broadcasting shape-`[]` scalar. `to_dense`/`eval`/
   `scale` guard the order-0 case. `scale` is **inherent** (not on the trait) to avoid colliding with
   `Module::scale`.
6. **AMEn-Part-I = normal equations (SPD), squares cond(A)**; the "avoid squaring" win is solving **square
   A directly** (Galerkin local systems on A, not AᵀA) — done in the new `linear`.
7. Clippy is enforced (fix, don't suppress). `RealField: Real` provides `AddAssign`/`MulAssign` so compound
   ops are available even in generic code.

## SOTA decision (user-approved, option A)

Adopt AMEn / DMRG3S and add SOTA where wise. Key papers (also to be cited in source + README):
- **AMEn** linear solver — Dolgov & Savostyanov 2014, *SIAM J. Sci. Comput.* 36(5), arXiv:1301.6068
  (Part I SPD; Part II nonsymmetric). → `solve::linear` (done, uncommitted).
- **DMRG3S** single-site DMRG + subspace expansion — Hubig, McCulloch, Schollwöck, Wolf 2015,
  *Phys. Rev. B* 91, 155115, arXiv:1501.05504. → Stage 3 eigensolver (shares AMEn enrichment engine).
- **CBE** controlled bond expansion — Gleis/Li/von Delft (DMRG arXiv:2207.14712; TDVP PRL 133, 026401).
  → optional frontier refinement.
- **TDVP** — Haegeman et al.; Paeckel et al. 2019 review "Time-evolution methods for MPS," *Ann. Phys.*
  → Stage 3 TDVP = two-site (or CBE-TDVP), not one-site (one-site can't grow rank).
- **Randomized TT rounding** — Daas/Ballard 2021 (arXiv:2110.04393), Khatri-Rao 2024. → optional perf
  upgrade to `round`/`add`; defer unless a bottleneck.

## TODO (resume here)

1. ~~**Re-verify** after the AMEn rewrite~~ — **DONE.** `cargo fmt` clean, `clippy --all-targets`
   exit 0, full `cargo test -p deep_causality_tensor` exit 0, workspace `cargo build` exit 0.
2. ~~**Citations in source files**~~ — **DONE.** `# Reference(s)` doc-comment sections added to:
   `tensor_svd_truncated/mod.rs` (Demmel & Veselić 1992), `tensor_qr/mod.rs` (Golub & Van Loan §5.2),
   `construct/mod.rs` `from_dense` (Oseledets 2011), `construct/cross.rs` (Oseledets–Tyrtyshnikov 2010
   + Goreinov et al. 2010 maxvol), `solve/local.rs` `fit` (Holtz–Rohwedder–Schneider 2012 + Grasedyck
   et al. 2015). `solve/local.rs` `linear` already carried the AMEn citation (Dolgov–Savostyanov 2014).
3. ~~**README References section**~~ — **DONE.** `## References` added to `deep_causality_tensor/README.md`
   (decomposition/numerics, cross, solvers, + a Stage-3 roadmap block: DMRG3S, CBE, TDVP review).
4. **Re-spec Stage 3** in `Tensor-Network-Spec.md` §5 and `add-tensor-network/tasks.md` group 5:
   eigensolver → **DMRG3S** (single-site + subspace expansion, reuse AMEn enrichment); TDVP → **two-site /
   CBE-TDVP**. *(Not yet started.)*
5. ~~**Draft the AMEn commit message**~~ — **DONE** (see below; awaiting user commit). Groups the AMEn
   `local.rs` rewrite together with the algorithm citations + README References as one follow-up.
6. Then continue **Stage 3** (DMRG3S eigensolver + two-site TDVP) and **Stage 4** (complex `Complex<T>` +
   dual `Dual<T>` scalar instantiation; complex needs the Hermitian SVD/QR kernel).

### Prepared commit message (item 5 — uncommitted working tree, user commits)

```
refactor(deep_causality_tensor): tensor-network — AMEn linear solve + algorithm citations

Rewrite solve::linear from normal-equations ALS to AMEn (Alternating Minimal
Energy): one-site ALS with residual subspace enrichment, so the bond dimension
is rank-adaptive (seeded small, grown toward the residual, capped at max_rank).
For a square operator the local Galerkin systems use A and b directly (cond A);
the rectangular path keeps the SPD normal equations (cond A²). Verified at
f64/Float106; clippy clean, workspace builds, full suite green.

Add paper citations to the doc comment of every algorithm in the layer
(one-sided Jacobi SVD, Householder QR, TT-SVD, TT-cross + maxvol, ALS/fit,
AMEn) and a ## References section to the crate README, including a Stage-3
roadmap block (DMRG3S, CBE, TDVP review).

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
```

## Open honest caveats

- `solve::linear` rectangular path still uses normal equations (cond²); the square path (the common
  implicit-step case) is direct. AMEn Part II (nonsymmetric direct) is the future fix for ill-conditioned
  rectangular/f32.
- TT-cross is maxvol/Oseledets-Tyrtyshnikov (standard); AMEn-cross / rect-maxvol are refinements, not done.
- `round`/`add` are deterministic QR+SVD; randomized rounding deferred.
