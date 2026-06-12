## Why

After the DEC solver performance pass (commit `1bce8a6d1`), the remaining
bottleneck in the periodic incompressible Navier-Stokes solver is the Leray
projection's grade-0 Poisson solve, which runs unpreconditioned conjugate
gradient once per solver step. CG cannot be parallelized across iterations and
its iteration count grows with grid resolution. On a periodic lattice the DFT
diagonalizes the discrete Laplacian exactly, so an FFT-based solve replaces
the entire CG iteration with an O(N log N) transform pair that is exact to
rounding. The workspace's zero-external-dependency policy rules out `rustfft`,
so the transform machinery must be implemented in-house as a dedicated crate.

Preparation notes: `openspec/notes/fft/add-fft.md` (motivation, math, resolved
decisions) and `openspec/notes/fft/fft_state_of_the_art.md` (algorithm survey;
its §5 build plan is binding for the implementation).

## What Changes

- New workspace crate `deep_causality_fft`: zero dependencies beyond
  `deep_causality_num`, `[lints] workspace = true` (no `unsafe`), generic over
  `RealField` using the existing `Complex<T: RealField>` /`ComplexField<R>`
  number tower.
- Forward and inverse complex FFT following the state-of-the-art build plan:
  iterative radix-2 correctness baseline validated against a naïve O(n²) DFT,
  radix-4 / mixed radix-2-4 power-of-two workhorse, hardcoded small-N
  butterflies, Bluestein fallback for arbitrary lengths. Inverse via
  conjugation reuse of the forward path (no separate inverse kernel).
- Real-to-complex (rFFT) and complex-to-real (irFFT) transforms in Stage 1,
  since the primary consumer's input (the Poisson right-hand side) is real.
- Plan-style API: precomputed twiddle/permutation state created once per
  transform length and reused across executions; allocation-free hot path.
- Multi-dimensional transforms by row-column decomposition over strided axes,
  with an opt-in Rayon `parallel` feature following the established
  topology/physics pattern (forwarded feature, granularity thresholds).
- `deep_causality_topology` gains a spectral Poisson path: on fully periodic
  lattices, `leray_project` solves `Δ₀ φ = δω` by FFT → pointwise divide by
  the lattice Laplacian eigenvalues (zero mode zeroed = gauge fix) → inverse
  FFT, replacing the CG solve on that path. CG remains the solver for
  non-periodic and mixed-periodicity lattices.
- Benchmarks: a dedicated criterion benchmark in the new crate, plus an update
  to `deep_causality_physics/benches/dec_solver_benchmark.rs` measuring the
  spectral projection's effect on the full solver step (currently 388 ms at
  32³ f64).

## Capabilities

### New Capabilities
- `fft-core`: forward/inverse complex FFT over `Complex<R: RealField>` —
  planner, power-of-two kernels (radix-2/radix-4), small-N butterflies,
  Bluestein fallback, normalization contract, naïve-DFT validation reference.
- `fft-real`: real-to-complex forward (rFFT) and complex-to-real inverse
  (irFFT) transforms with the half-spectrum (Hermitian-symmetry) layout.
- `fft-multidim`: D-dimensional transforms via batched strided 1-D transforms;
  opt-in `parallel` feature semantics and granularity thresholds.
- `spectral-poisson`: FFT-based solve of the gauge-fixed grade-0 Poisson
  problem on fully periodic lattices, consumed by the Leray projection.

### Modified Capabilities
- `leray-projection`: on fully periodic lattices the grade-0 solve SHALL use
  the spectral path (exact to rounding, no tolerance/iteration budget, no
  convergence-failure mode) instead of CG; CG behavior and error semantics
  are retained for all non-spectral paths. Existing scenarios phrased
  against "the CG tolerance" gain a spectral counterpart.

## Impact

- New crate `deep_causality_fft` added to the workspace (`Cargo.toml` members,
  BUILD.bazel/MODULE.bazel per repo build conventions).
- `deep_causality_num`: consumed for `Complex`, `ComplexField`, `RealField`,
  `FromPrimitive`; no API changes. One internal performance improvement:
  `Float106::sin_cos` gains a fast path (table-based octant reduction + short
  shared Taylor pair) replacing the current two independent full-range Taylor
  evaluations, so extended-precision twiddle generation is not pathologically
  slow.
- `deep_causality_topology`: new dependency on `deep_causality_fft`; spectral
  dispatch inside `leray_project`/the grade-0 Hodge branch; plan-cache state
  placed wherever profiling shows the most gain (manifold beside
  `boundary_cache`, or `DecNsSolver`).
- `deep_causality_physics`: no API change; inherits the faster projection;
  `dec_solver_benchmark.rs` extended. Tests pinning CG-converged values on
  periodic grids may need tolerance review since the spectral result is exact
  to rounding.
- No breaking public-API changes anticipated.
