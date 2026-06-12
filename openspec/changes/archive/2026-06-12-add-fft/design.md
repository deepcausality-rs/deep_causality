## Context

The DEC-native periodic incompressible Navier-Stokes solver runs a Leray
projection per step
(`deep_causality_topology/src/types/manifold/differential/leray.rs`). Its
grade-0 Poisson solve `Δ₀ φ = δω` goes through `solve_laplacian` →
`deep_causality_sparse::cg_solve`, gauge-fixed by `subtract_mean_in_place`.
After the `1bce8a6d1` performance pass (boundary-matrix memoization,
allocation-free operator `_of` variants, opt-in Rayon `parallel` feature),
this CG solve dominates the 388 ms full step at 32³ f64. CG's outer loop is
inherently sequential and its iteration count grows with resolution.

On a fully periodic lattice the DFT diagonalizes `Δ₀` exactly, so a forward
FFT, a pointwise divide, and an inverse FFT replace the iteration entirely:
O(N log N), exact to rounding, no convergence-failure mode.

Constraints that shape the design:

- `[workspace.dependencies]` is intentionally empty — no `rustfft`, no
  `num-complex`. Everything is built in-house.
- Repo-wide `unsafe_code = "forbid"`; new crates opt in with
  `[lints] workspace = true`.
- The solver stack is generic over `deep_causality_num::RealField` (f32, f64,
  Float106 in tests); `deep_causality_num` already provides
  `Complex<T: RealField>` and the `ComplexField<R>` trait (`conjugate`,
  `norm_sqr`, `from_re_im`, …). The number tower deliberately separates the
  real type, the `RealField` algebra, and the adjacent complex field.
- Binding references: `openspec/notes/fft/add-fft.md` (decisions §7) and
  `openspec/notes/fft/fft_state_of_the_art.md` (§5 build plan).
- `LatticeComplex<const D, R>` exposes `shape()` and `periodic()`; the metric
  (Hodge star) carries the geometric scaling that `Δ₀` applies.

## Goals / Non-Goals

**Goals:**

- A dedicated crate `deep_causality_fft` with forward/inverse complex FFT and
  forward/inverse real FFT (rFFT/irFFT), generic over `RealField`, depending
  only on `deep_causality_num`.
- Wall-clock-fast by the state-of-the-art survey's standard: regular,
  cache-friendly mixed radix-2/4 power-of-two core with hardcoded small-N
  butterflies and a Bluestein fallback — not the flop-record split-radix.
- A plan-style API (build once per length, execute many, allocation-free hot
  path) so the solver's step loop pays setup cost once.
- D-dimensional transforms by row-column decomposition, parallelizable under
  the established opt-in Rayon `parallel` feature pattern.
- A spectral grade-0 Poisson path in `deep_causality_topology`, dispatched
  automatically inside `leray_project` (and the grade-0 Hodge branch) when the
  lattice is fully periodic.
- Benchmarks: stand-alone criterion benchmark in the FFT crate; updated
  `dec_solver_benchmark.rs` in physics.

**Non-Goals:**

- Johnson–Frigo modified split-radix (flop-optimal but irregular access; the
  survey recommends against it as the workhorse).
- Rader's and Good–Thomas algorithms (Bluestein already covers prime and
  awkward sizes at O(N log N); add specialized prime handling only if a real
  workload demands it).
- Explicit SIMD intrinsics (forbidden-`unsafe` policy; rely on
  auto-vectorization of the regular radix-4 kernels).
- Spectral solves on non-periodic or mixed-periodicity lattices (CG remains
  that path; a preconditioner stays the follow-up there).
- Replacing CG anywhere else (the β-step of the full Hodge decomposition is
  untouched).
- A standalone spectral *differentiation* toolkit (energy spectra, dealiasing
  etc. are natural future consumers, not part of this change).

## Decisions

### D1: New crate `deep_causality_fft`, consuming the existing number tower

The crate defines transform machinery only; `Complex<R>`/`ComplexField<R>`
come from `deep_causality_num`. Alternative considered: split re/im slice
representation inside the FFT crate — rejected because the workspace already
has a public, trait-backed complex type and an interleaved `&mut [Complex<R>]`
buffer keeps the API direct; `Complex<R>` is `Copy` with `{re, im}` layout, so
butterflies compile to the same scalar arithmetic.

### D2: Layered algorithm structure per the survey's §5 build plan

1. Naïve O(n²) DFT — never on the hot path, kept as the validation reference
   (the RustFFT pattern).
2. Iterative radix-2 DIT with bit-reversal permutation and precomputed
   twiddles — the correctness baseline.
3. Radix-4 / mixed radix-2-4 — the power-of-two workhorse (one radix-2 stage
   for odd log₂N, radix-4 stages otherwise). Chosen over split-radix because
   regular access auto-vectorizes; this is where production libraries get
   their wall-clock speed.
4. Hardcoded straight-line butterflies for small N (2, 4, 8, 16, 32) used as
   base cases.
5. Bluestein chirp-z fallback for arbitrary lengths, layered on the
   power-of-two core (pad to a power of two ≥ 2n−1).

A planner selects per length: hardcoded butterfly → power-of-two pipeline →
Bluestein. The solver grids (16³/32³/64³) always hit the power-of-two path;
Bluestein guarantees O(N log N) for any future lattice shape rather than a
silent O(n²) cliff.

### D3: Inverse by conjugation reuse; normalization fixed as forward-unnormalized, inverse-1/N

`ifft(x) = conj(fft(conj(x))) / N`. One forward kernel serves both directions,
which keeps the pair provably consistent — more valuable for a causality
framework than shaving the conjugation passes. The normalization convention is
part of the public contract and documented on every entry point; round-trip
`ifft(fft(x)) = x` to rounding is a spec scenario.

### D4: rFFT in Stage 1 via the half-spectrum layout

Real input of length N produces N/2+1 complex bins (Hermitian symmetry);
irFFT inverts it. Implemented by the standard N/2-complex-FFT packing trick,
so it reuses the complex core. The Poisson RHS is real, so the primary
consumer never pays for the redundant half-spectrum.

### D5: Plan API with caller-visible scratch

`FftPlan<R>` (and `RfftPlan<R>`) are built per transform length and hold
twiddles, the permutation, and the stage schedule; they are immutable after
construction (read-only and shareable, the PocketFFT property). Execution
borrows a caller-provided scratch buffer so repeated calls allocate nothing —
consistent with the allocation-elimination direction of the perf pass.

### D6: Multi-dimensional transforms by row-column decomposition

A `FftPlanNd<R>` holds one 1-D plan per axis length and applies batched
strided 1-D transforms axis by axis. Strided columns are gathered into
contiguous scratch, transformed, and scattered back (cache-friendlier than
strided butterflies, and it reuses the contiguous kernels unchanged). Under
the `parallel` feature the independent 1-D batches fan out via Rayon behind a
measured granularity threshold — small transforms stay serial (the 24³ lesson:
an unthresholded fan-out ran 2× slower from fork-join overhead). The
`MaybeParallel` bound pattern carries `Send + Sync` only under the feature.

### D7: Spectral Poisson dispatch is automatic on fully periodic lattices

`leray_project` (and the grade-0 branch only of `hodge_decompose`) checks
`periodic().iter().all(|&p| p)`; when true it uses the spectral solve,
otherwise CG, with no new options surface. Alternative considered: opt-in via
`HodgeDecomposeOptions` — rejected because the spectral result is strictly
better (exact vs. tolerance-converged) and a silent always-on improvement
matches how the boundary-matrix memoization shipped. The CG-facing options
(tolerance, iteration budget) are simply unused on the spectral path; CG error
semantics are unchanged everywhere CG still runs.

The spectral solve: rFFT of `δω` → divide bin `k` by the lattice Laplacian
eigenvalue `λ_k = Σ_d (2 − 2·cos(2π·k_d/N_d)) / h_d²` → zero the `k = 0` bin
(this *is* the mean-subtraction gauge fix, expressed spectrally) → irFFT.
The eigenvalues MUST match the discrete `Δ₀` that the CG path applies,
including the metric scaling the Hodge star encodes; the cross-check
"spectral and CG agree on the same periodic problem within the CG tolerance"
is the binding verification anchor, not the formula transcription.

### D8: Plan cache placed by measurement

The per-shape `FftPlanNd` plus eigenvalue table must be computed once and
reused across solver steps. Two candidate homes: beside `boundary_cache` in
the manifold (shared by anything that projects on that manifold, preserved
across `Clone` like the existing caches) or owned by `DecNsSolver` (already
holds per-run state). Decision rule per the preparation note: wherever
profiling shows the most gain; the benchmark comparison in the tasks settles
it. Default starting point is the manifold-side cache for symmetry with
`boundary_cache`, falling back to solver-owned if borrow or contention issues
surface.

### D9: HKT composability — the FFT is an `Iso`-shaped morphism, not a Functor

Investigated against `deep_causality_haft` and the uniform-math surface
(`website/docs/src/content/docs/concepts/hkt.md`, `uniform-math.md`). Three
candidate encodings, two rejected:

- **`Functor`/`Monad` instance on the FFT itself — rejected.** `fmap` and
  `bind` are pointwise/structure-preserving; the FFT is a global linear map
  across the whole container. Forcing it into `fmap` would violate the
  functor laws.
- **Tier 3 `NaturalIso<F, G>` between a position-space and a frequency-space
  witness — rejected.** `NaturalIso` must hold for *every* `T` and commute
  with `fmap` (`to_target(F::fmap(fa, h)) == G::fmap(to_target(fa), h)`).
  The FFT exists only at `T = Complex<R>`, and naturality fails for
  nonlinear `h` (the DFT commutes with linear maps only).
- **Tier-2-style value-level `Iso` pair + Functor in the frequency domain —
  adopted.** The transform pair *is* an isomorphism on a fixed container:
  the round-trip law `ifft(fft(x)) = x` (D3) is literally the Tier 2
  `Iso::to_target`/`to_source` round-trip law from `deep_causality_num`,
  carried by a stateful plan rather than a stateless `From` impl. Once in
  the frequency domain, the data is an ordinary `CausalTensor<Complex<R>>`,
  and `CausalTensorWitness` already gives it lawful `Functor`/`Monad`
  instances. The spectral Poisson solve then reads as a conjugation in the
  uniform vocabulary — `irfft ∘ fmap(divide-by-λ_k) ∘ rfft` — with the
  pointwise eigenvalue divide expressed through the existing tensor `fmap`,
  no new trait surface and no new witness.

Consequences: `deep_causality_fft` stays slice-level and depends only on
`deep_causality_num` (no `haft` dependency); the composition with the uniform
math stack happens in the consuming layer (topology), which already holds the
tensor witnesses. The FFT crate's tests assert the iso round-trip laws in the
same spirit as `haft`'s `assert_natural_iso_round_trip` test support. A
first-class `FourierIso` witness (e.g., adjunction-style position↔momentum
bridge alongside `BoundedAdjunction`) remains open as a future change if a
second spectral consumer materializes; nothing in this design blocks it.

### D10: Two-level benchmarking

- `deep_causality_fft/benches/`: criterion benchmark over 1-D and 3-D
  transforms at the solver-relevant sizes (16³, 32³, 64³) plus a Bluestein
  size, f32/f64.
- `deep_causality_physics/benches/dec_solver_benchmark.rs`: extended with a
  spectral-projection variant so the full-step effect is measured directly
  against the current 388 ms / 32³ baseline.

## Risks / Trade-offs

- [Eigenvalue/operator mismatch — the spectral solve silently diverges from
  the discrete `Δ₀` if the metric scaling is transcribed wrong] → The
  CG-vs-spectral agreement test on multiple shapes and anisotropic spacings is
  a hard gate; the Taylor-Green closed-form decay test in physics is the
  end-to-end backstop.
- [Tests pinning CG-converged values change at the rounding level once the
  periodic path is exact] → Audit periodic-lattice tolerance assertions in
  topology/physics; tolerances may only tighten, never loosen.
- [Generic `RealField` blocks the precomputed-constant tricks of f64-only
  FFTs; `Float106::sin_cos` is currently two independent full-range Taylor
  evaluations (~60 double-double iterations each)] → Two-sided mitigation.
  (a) Twiddle generation goes through the numerically careful
  recurrence-plus-exact-anchor scheme (the PocketFFT approach) using
  `FromPrimitive` conversions, so only O(√N) `sin_cos` calls happen per plan,
  once — accuracy is spec-tested against the naïve DFT per precision.
  (b) `deep_causality_num` gains a real fast `sin_cos` path for `Float106`
  (QD-library style): reduce `x = k·π/16 + s` with `|s| ≤ π/32` via a
  32-entry precomputed double-double table of `sin/cos(k·π/16)`, evaluate the
  short Taylor pair for `sin(s)`/`cos(s)` (≈10 terms at the 1e-33
  convergence cutoff instead of ~60), and combine by angle addition —
  computing both values in one pass over a shared reduction and shared `s²`
  powers. This is an internal performance change to an existing trait method
  (`sin_cos` is already in the `Float` surface), no API or requirement-level
  behavior change; it also benefits every other Float106 consumer.
- [Bit-reversal and strided gather/scatter are bounds-checked under
  forbidden-`unsafe`] → Accept the cost; structure inner loops over
  slices/chunks so the compiler elides checks. The win over CG is algorithmic
  (O(N log N) vs O(N·iters)); single-digit-percent kernel overhead does not
  threaten it.
- [Auto-dispatch changes results on existing periodic workloads without an
  opt-out] → Results only become more accurate; documented in the changelog.
  If a regression escape hatch proves necessary during implementation, a
  crate-internal fallback to CG is trivial since the CG path remains intact.
- [Bluestein doubles memory and roughly triples cost versus a native
  power-of-two transform] → Acceptable: it is a correctness fallback for
  non-power-of-two shapes, which no current workload uses.

## Migration Plan

1. Land `deep_causality_fft` as a new workspace member (Cargo + Bazel build
   files) with no consumers — pure addition, no risk.
2. Wire the spectral path into topology behind the periodicity check; all
   non-periodic behavior is untouched, so rollback is deleting the dispatch
   branch.
3. Update benchmarks and the physics README performance table.

No data migration; no public-API breakage. The release flows through the
existing release-plz workspace process.

## Open Questions

- None blocking. The plan-cache home (D8) is deliberately resolved by
  measurement during implementation rather than up front.
