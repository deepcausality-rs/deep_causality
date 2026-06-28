## Why

The tensor-network layer (`add-tensor-network`, now archived) is functionally complete and fully
tested, but the benchmark suite (group 7) exposed where the constant factors live. Profiling the
Criterion rows against the textbook TT complexity classes shows **no asymptotic outliers** — every
operation tracks its expected `O(d·n·rᵏ)` cost — but three cost centers dominate, all traceable to
deliberate first-pass design choices:

1. **SVD / rounding is the bottleneck class.** `svd_truncated` (~1.35 ms at 48×48) uses one-sided
   Jacobi — chosen for high *relative* accuracy and zero external dependencies, but intrinsically
   ~5–10× slower than a bidiagonalization SVD. It propagates into `from_dense` (~55 µs), `round`
   (~73 µs), `hadamard_rounded`, and every solver's truncation.
2. **`hadamard` materializes the squared bond.** It builds cores at bond `r²` (256 at the middle of a
   bond-16 train) *before* any truncation, an `O(d·n·r⁴)` peak even when a round follows.
3. **The in-house `matmul` is an unblocked triple loop**, and the transfer-matrix contractions
   (`inner`/`norm`) allocate a fresh buffer per site.

A literature scan of the preprint servers shows this is a well-studied problem with a mature, modern,
*dependency-free-friendly* answer for each cost center — and the highest-impact one (randomized TT
rounding) the original design explicitly deferred "unless a bottleneck." The benchmarks now show it is
the bottleneck. This change applies the state of the art **without abandoning the precision/scalar
generality (`ConjugateScalar`), the no-BLAS/no-`unsafe` constraints, or the high-accuracy defaults** —
every fast path is opt-in behind the existing `Truncation` policy, with the deterministic kernels kept
as the default.

State of the art reviewed (arXiv / SIAM):
- **Adaptive randomized TT-rounding via Khatri-Rao sketches** — Al Daas, Ballard et al.,
  arXiv:2511.03598 (2025); foundational randomized rounding, SIAM J. Sci. Comput. 2023
  ([doi:10.1137/21M1451191](https://epubs.siam.org/doi/10.1137/21M1451191), arXiv:2110.04393).
  Randomize-then-orthogonalize with structured sketches; `O(d·n·r²·ℓ)` vs deterministic `O(d·n·r³)`;
  **up to ~50× over deterministic rounding**, adaptive (tolerance-based) with a built-in error
  estimator that matches our `Truncation` tol gates.
- **Greedy-pivot TT-cross with nestedness** — Shi, Hayes, Qiu, arXiv:2407.11290 (2024/2025); greedy
  (largest-residual) pivoting is quasi-optimal yet much cheaper than maxvol (`O(N·R²)`).
- **Randomized / block-Krylov & UTV SVD** — arXiv:2308.01480, arXiv:2504.04989; faster-than-Jacobi
  matrix SVD when the tolerance permits.

## What Changes

- **Adaptive randomized TT-rounding** as an opt-in policy on `round` (and the `*_rounded` and AMEn
  enrichment recompressions): randomize-then-orthogonalize with Khatri-Rao-structured Gaussian sketches
  applied core-by-core (never forming the full sketch), adaptive rank from the residual estimator.
  Deterministic SVD rounding stays the default; the randomized path is selected through the
  `Truncation` policy.
- **Greedy-pivot option for TT-cross**: residual-greedy pivot selection (with the nestedness property)
  as a cheaper alternative to the current maxvol/LU pivot, selectable via `CrossConfig`.
- **Fused Hadamard-then-truncate**: a `hadamard_rounded` that compresses bond-by-bond as it builds, so
  the `r²` blow-up is never materialized.
- **Cache-blocked, allocation-reusing dense kernels**: a B-transposed, cache-blocked `matmul` and
  scratch-buffer reuse in the `inner`/`norm` transfer-matrix contractions (still BLAS-free, still
  `Default`-free so `Dual` keeps working).
- A **benchmark-backed acceptance gate**: each fast path must match the deterministic result to the
  configured tolerance, and the bench suite must show the intended speedup.

No breaking changes: every addition is opt-in; the default behaviour, the public deterministic methods,
and the `ConjugateScalar` scalar generality (real / `Dual` / `Complex`) are preserved.

## Capabilities

### Modified Capabilities
- `tensor-train`: gains an **adaptive randomized rounding** policy on `round` (and the rounded
  variants), a **fused Hadamard-then-truncate**, and **allocation-reusing** `inner`/`norm`
  contractions — all behaviour-preserving (same result to tolerance) with reduced cost.
- `tensor-train-cross`: gains a **greedy-pivot** selection strategy as a `CrossConfig` option, cheaper
  than and quasi-equivalent to the maxvol/LU default.
- `tensor-network-numerics`: gains a **cache-blocked dense `matmul`** kernel and (optional) a
  **randomized range-finder truncated SVD** variant for the cases where the tolerance permits trading
  Jacobi's relative accuracy for speed.
