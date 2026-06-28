## Context

`add-tensor-network` (archived) delivered the full MPS/MPO stack across `f32`/`f64`/`Float106`/`Dual`/
`Complex` behind one `ConjugateScalar` bound, with deterministic, accuracy-first kernels (one-sided
Jacobi SVD, Householder QR, maxvol/LU TT-cross) and no external dependencies. The benchmark suite makes
the cost structure observable; this change applies published, dependency-free-compatible accelerations
to the dominant cost centers **without changing defaults or weakening accuracy guarantees**.

Hard constraints carried over (AGENTS / repo policy):
- `unsafe_code = "forbid"`; no `dyn`; no lib-code macros; no concrete float literals in lib code.
- No new external runtime crates. Randomness uses the existing self-contained splitmix64 seed already
  used by `random_seeded` / `init_random` / `cross` — deterministic and reproducible.
- Precision/scalar generality preserved: every new path is bound on `ConjugateScalar`, with magnitudes
  in `T::Real`. The `CausalTensor::matmul`/`sum_axes` `Default` bound is still avoided (so `Dual` works)
  — the TT-layer `linalg` is used.
- **Defaults unchanged.** Deterministic Jacobi-SVD rounding, maxvol cross, and exact `hadamard` remain
  the default. Fast paths are opt-in through the existing `Truncation` / `CrossConfig` types.

## Goals / Non-Goals

Goals: cut the constant factors of the SVD/rounding-bound operations (`round`, `from_dense`,
`hadamard_rounded`, solver enrichment), the cross pivot step, and the dense contraction kernels —
verified by (a) correctness-to-tolerance against the deterministic path and (b) the benchmark suite.

Non-Goals: multi-threading/parallel TT-SVD (TSQR) — deferred (the repo is single-threaded by default;
`rayon` is an optional feature elsewhere). General-geometry tensor-network contraction-order
optimization (cotengra-style) — not applicable to the linear TT chain (its order is fixed); revisit
only if the network topology generalizes beyond MPS/MPO. Reverse-mode AD — out of scope as before.

## Decisions

### 1. Adaptive randomized TT-rounding (primary)

**Algorithm (randomize-then-orthogonalize with Khatri-Rao sketches; Al Daas–Ballard 2023, adaptive
variant arXiv:2511.03598).** To round a TT `X` (cores `X_k`, bonds `r_k`) to target sketch size `ℓ`:

1. **Randomize (right-to-left partial contraction).** Draw independent Gaussian matrices `Ω_k`
   (`n_k × ℓ`) per site (`from_real` of splitmix64-derived normals; Box–Muller). Form the structured
   sketch contractions without materializing the `n₁···n_d × ℓ` matrix, via the recurrence
   `W_k = H(X_k) · (W_{k+1} ⊙ Ω_k)` (`⊙` = column-wise Khatri–Rao), `W_d = H(X_d)·Ω_d`. Cost
   `O(d·n·r²·ℓ)`.
2. **Orthogonalize (left-to-right QR sweep).** For `k = 1..d-1`: form `S_k = V(Y_k)·W_{k+1}`,
   QR-factor `S_k = Q_k R_k`, set core `k ← Q_k`, push `Qᵀ`·(old core) into core `k+1`.
3. **Adaptive rank.** Start `ℓ = ⌈max_k r_k · f_init⌉` (e.g. `f_init = 0.1`). Estimate the relative
   residual with a few extra sketch columns, `‖(I − QQᴴ)·X·Ω‖_F / (√s · ‖X‖)`; if it exceeds the
   `Truncation` tolerance, grow `ℓ` and re-sweep; else stop. Typically 2–3 iterations.

Complexity `O(d·n·r²·ℓ)` with `ℓ` ≈ retained rank + small oversample, vs deterministic `O(d·n·r³)` —
the published win is up to ~50× when compressing a high-rank train.

**Hermitian/complex correctness.** All inner products and QR are the conjugate-aware kernels already in
the crate; the sketch is real-Gaussian for real `T` and `from_real`-injected for complex (sketching the
column space is conjugation-agnostic). The error estimator uses `T::Real` magnitudes.

**Surfacing.** Add a `Truncation` rounding-method selector (deterministic | randomized) — the policy
object already threads everywhere `round` is called, so no method-signature churn. Deterministic stays
the default; randomized requires the caller to opt in (and, being randomized, is documented as
tolerance-accurate, not bit-reproducible beyond the fixed seed).

### 2. Greedy-pivot TT-cross (secondary)

Replace the cost of the maxvol/LU rank-revealing pivot with **residual-greedy pivoting**
(arXiv:2407.11290): pick the row/column of largest current residual `|A − Ã|`, which is quasi-optimal
(volume within a bounded factor of maxvol) at `O(N·R²)` instead of maxvol's iterative row swaps, and
yields the **nestedness** property (left/right index sets stay nested across sweeps, so partial
contractions are reused). Selected via a `CrossConfig` strategy flag; maxvol/LU stays the default until
the greedy path is validated to recover the same low-rank oracles.

### 3. Fused Hadamard-then-truncate (secondary)

`hadamard` is exact and squares the bond (`r²`); `hadamard_rounded` currently builds the full `r²` train
then rounds. The fused version rounds **as it builds**: maintain a running left-canonical form and, after
forming each squared-bond core, immediately truncate it against the carried `R` before moving on, so the
peak bond is `~r·r_keep` rather than `r²`. Result identical to `hadamard(...).round(...)` within
tolerance; only the transient memory/flops differ. (Composes with the randomized rounding of §1.)

### 4. Cache-blocked, allocation-reusing dense kernels (tertiary)

- **`linalg::matmul`**: transpose `B` once and tile the `i`/`j`/`p` loops in cache-friendly blocks
  (e.g. 32–64) so the hot loop is unit-stride on both operands. BLAS-free, `Default`-free.
- **`inner`/`norm` transfer-matrix contraction**: reuse two ping-pong scratch `Vec`s across sites
  instead of allocating `m`/`nl` per site, removing the per-site allocation that dominates small trains.

## Risks / Trade-offs

- **Randomized ≠ bit-reproducible.** Mitigated by the fixed seed (reproducible per run) and by keeping
  deterministic the default; documented. Accuracy is controlled to the tolerance via the residual
  estimator, with oversampling guarding the tail.
- **Greedy pivots can stall on adversarial oracles** (as maxvol can). Mitigated by keeping maxvol the
  default, bounding sweeps, and reporting the residual.
- **Block sizes are machine-dependent.** Use a conservative compile-time block constant (no concrete
  float literals; integer block size is fine), validated by the bench suite; no autotuning.
- All four are independently shippable and independently revertible; each carries a correctness gate
  against the deterministic path before it can become non-default.

## Migration

Purely additive and opt-in. No public type or default-behaviour change: `round`/`cross`/`hadamard`
keep their signatures and defaults; the new fast paths are reached through new `Truncation` /
`CrossConfig` options. Existing tests and downstream crates are unaffected.
