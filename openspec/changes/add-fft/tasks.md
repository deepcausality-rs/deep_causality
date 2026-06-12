## 1. Crate scaffolding

- [ ] 1.1 Create `deep_causality_fft` workspace member: `Cargo.toml` (dep: `deep_causality_num` only; `[lints] workspace = true`; optional `parallel` feature wiring), `README.md`, `src/lib.rs` skeleton with crate docs referencing the normalization contract
- [ ] 1.2 Add Bazel build files (`BUILD.bazel`, MODULE registration) following the pattern of the most recently added crate
- [ ] 1.3 Verify `make build` and `cargo tree -p deep_causality_fft` show the crate compiling with only `deep_causality_num` as dependency

## 2. Fast `sin_cos` for Float106 (deep_causality_num)

- [ ] 2.1 Implement the fast `Float106::sin_cos` path: 32-entry precomputed double-double table for `sin/cos(k·π/16)`, reduction `x = k·π/16 + s` with `|s| ≤ π/32`, short shared Taylor pair for `sin(s)`/`cos(s)`, angle-addition combine; rewrite `sin`/`cos` to delegate to it
- [ ] 2.2 Accuracy tests: agreement with the existing Taylor implementation to ≤ 1e-32 relative across [0, 2π) sample points and the exact anchors (0, π/2, π, 3π/2); regression-test the existing `Float106` trig test suite
- [ ] 2.3 Micro-benchmark old vs new `sin_cos` and record the speedup in the change notes

## 3. Complex FFT core (fft-core)

- [ ] 3.1 Implement the naïve O(n²) DFT validation reference (test-only path, never planner-selected)
- [ ] 3.2 Implement numerically careful twiddle generation over `RealField + FromPrimitive` (exact anchors + bounded recurrence, PocketFFT-style) with per-precision accuracy tests against the naïve DFT
- [ ] 3.3 Implement iterative radix-2 DIT with bit-reversal permutation — the correctness baseline, tested against the naïve DFT at f32/f64
- [ ] 3.4 Implement the mixed radix-2/radix-4 power-of-two pipeline (radix-4 stages, one radix-2 stage for odd log₂N); test against radix-2 and the naïve DFT
- [ ] 3.5 Implement hardcoded straight-line butterflies for N = 2, 4, 8, 16, 32 and use them as planner base cases
- [ ] 3.6 Implement `FftPlan<R>`: immutable per-length plan (twiddles, permutation, stage schedule), execute over `&mut [Complex<R>]` with caller-provided scratch; test that repeated execution performs no per-execution allocation
- [ ] 3.7 Implement the inverse transform by conjugation reuse (`conj → forward → conj → 1/N`) and the round-trip law test `ifft(fft(x)) = x` to rounding; assert the iso round-trip in the spirit of haft's `assert_natural_iso_round_trip` (D9)
- [ ] 3.8 Implement the Bluestein chirp-z fallback on the power-of-two core; planner dispatch (butterfly → power-of-two → Bluestein); tests at prime and other non-power-of-two lengths

## 4. Real transforms (fft-real)

- [ ] 4.1 Implement rFFT via the N/2-complex packing trick producing the `N/2 + 1` half-spectrum; test against the complex FFT of the same data and the bin-0/bin-N/2 realness property
- [ ] 4.2 Implement irFFT (half-spectrum → N real samples, 1/N scaling); round-trip test `irfft(rfft(x)) = x` to rounding

## 5. Multi-dimensional transforms (fft-multidim)

- [ ] 5.1 Implement `FftPlanNd<R>` (and the real-entry-axis variant): row-column decomposition, one shared 1-D plan per distinct axis length, gather/transform/scatter for strided axes
- [ ] 5.2 Tests: 3-D vs naïve per-axis DFT on a small grid; round-trip at 16³ and 32³; anisotropic shape (16×32×8) with plan sharing
- [ ] 5.3 Implement the `parallel` feature: Rayon fan-out of independent 1-D batches behind a measured granularity threshold (small transforms stay serial); test output identity across both feature configurations
- [ ] 5.4 Add the stand-alone criterion benchmark (1-D and 3-D at 16³/32³/64³ sizes plus one Bluestein size, f32/f64) and record the granularity threshold from measurement

## 6. Spectral Poisson path in topology (spectral-poisson)

- [ ] 6.1 Add `deep_causality_fft` as a dependency of `deep_causality_topology` (Cargo + Bazel; forward the `parallel` feature)
- [ ] 6.2 Implement the eigenvalue table for the periodic lattice Laplacian `Δ₀` (per-shape `λ_k`, including the metric/spacing scaling the Hodge star encodes) and the spectral solve: rFFT → divide (zero the k=0 bin) → irFFT, composed over the tensor surface per D9 (`irfft ∘ fmap(/λ_k) ∘ rfft`)
- [ ] 6.3 Cross-check test: spectral vs CG agreement within CG tolerance on multiple fully periodic shapes including anisotropic spacings; residual-at-rounding and zero-mean gauge tests
- [ ] 6.4 Implement plan/eigenvalue caching — start manifold-side beside `boundary_cache` (preserved across `Clone`), benchmark against a `DecNsSolver`-owned variant, keep the faster placement (D8); test that repeated solves build plans once and do not allocate per solve

## 7. Leray dispatch (leray-projection delta)

- [ ] 7.1 Wire automatic dispatch in `leray_project` and the grade-0 branch of `hodge_decompose`: spectral when `periodic()` is all-true, CG otherwise; no new options surface
- [ ] 7.2 Dispatch tests: fully periodic takes the spectral path; mixed-periodicity and open lattices behave identically to pre-change CG (regression guard); spectral and CG projections agree on the same input
- [ ] 7.3 Audit existing periodic-lattice tolerance assertions in topology and physics tests; tighten where the spectral path makes results exact (never loosen)

## 8. Solver benchmark and docs

- [ ] 8.1 Extend `deep_causality_physics/benches/dec_solver_benchmark.rs` with spectral-projection and spectral full-step measurements at 16³/32³ against the recorded CG baseline (full step 388 ms at 32³ f64)
- [ ] 8.2 Update the physics example README performance table and the `deep_causality_fft` README (algorithm layering, normalization contract, `parallel` feature)
- [ ] 8.3 Full verification: `make build`, `make test`, clippy clean in both feature configurations, workspace-wide
