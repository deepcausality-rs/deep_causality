# Note: Add an FFT crate (`deep_causality_fft`) for the spectral Poisson solve

Status: pre-spec note. Captures the motivation, the math, the design
constraints, and the open questions ahead of an OpenSpec proposal.

## 1. Where this comes from

The DEC-native periodic incompressible Navier-Stokes solver (CFD Stage 1,
`deep_causality_physics/src/theories/fluid_dynamics/dec/`) went through a
performance pass (commit `1bce8a6d1`): boundary-matrix memoization, removal of
per-evaluation allocation and manifold cloning, and an opt-in Rayon `parallel`
feature. The 32³ f64 full solver step went from 850 ms to 388 ms.

Profiling after that pass shows the remaining bottleneck is the pressure
projection. Every solver step runs a Leray projection
(`deep_causality_topology/src/types/manifold/differential/leray.rs`), which
solves the grade-0 Poisson problem `Δ₀ φ = δω` with unpreconditioned conjugate
gradient (`deep_causality_sparse::cg_solve`), gauge-fixed by mean subtraction.

CG resists further parallelization by construction: each iteration depends on
the result of the previous one, so only the per-iteration matvec fans out, and
the iteration *count* grows with grid resolution (the lattice Laplacian's
condition number scales as O(N²) in the per-axis cell count). The commit
message names a preconditioner as the next follow-up, but on a periodic
lattice there is a stronger move available: eliminate the iteration entirely.

## 2. Why FFT removes the bottleneck

On a periodic lattice (flat torus), the discrete Laplacian `Δ₀` is a
convolution, so the DFT diagonalizes it exactly. The eigenvalue for wave
vector `k = (k₁, …, k_D)` on an `N₁ × … × N_D` grid with spacing `h_d` is

```
λ_k = Σ_d (2 − 2·cos(2π·k_d / N_d)) / h_d²
```

The Poisson solve becomes three steps:

1. Forward FFT of the right-hand side `δω`.
2. Pointwise divide: `φ̂_k = (δω)̂_k / λ_k`, with `φ̂_0 = 0` for the zero
   mode — this *is* the gauge fix that `subtract_mean_in_place` currently
   performs, expressed spectrally.
3. Inverse FFT to recover `φ`.

So both a forward and an inverse transform are required per projection.

Properties relative to CG:

- **Cost**: O(N log N) total, fixed and known in advance, versus
  O(N · iterations) with a resolution-dependent iteration count.
- **Exactness**: the solve is exact to rounding — no tolerance, no iteration
  budget, no `HodgeDecompositionFailed` convergence failure on this path.
- **Parallelism**: a D-dimensional FFT is a batch of independent 1-D
  transforms along each axis; the batches fan out cleanly under the existing
  `parallel` feature, unlike CG's sequential outer loop.

This supersedes the planned CG preconditioner *on the periodic path only*.
CG (eventually preconditioned) remains the solver for non-periodic and
variable-grid geometries (`openspec/notes/cfd/variable-grid-geometry.md`),
where the Laplacian is not a convolution.

## 3. Why a dedicated crate

The workspace has a zero-external-dependency policy
(`[workspace.dependencies]` is intentionally empty); pulling in `rustfft` is
not an option. The FFT belongs in its own crate, `deep_causality_fft`,
following the existing pattern of small focused crates
(`deep_causality_sparse`, `deep_causality_rand`, `deep_causality_num`):

- The topology crate should not own transform machinery; it should consume it
  the same way it consumes `cg_solve` from `deep_causality_sparse`.
- Other plausible consumers exist beyond the Poisson solve: energy-spectrum
  diagnostics for the Taylor-Green / CFD-challenge evaluation, and spectral
  kernels in the physics theory modules (waves, photonics, condensed matter).

## 4. Design constraints

- **`unsafe_code = "forbid"`**: the crate opts in with `[lints] workspace =
  true`. An FFT needs no `unsafe`; it is index arithmetic over slices. Do not
  join the documented exemption list (rand/multivector/topology).
- **Generic over `RealField`**: the solver stack is generic over
  `deep_causality_num::RealField` (+ `FromPrimitive`), and the FFT must be
  too. This requires a complex arithmetic representation — either a `Complex<R>`
  type (check whether `deep_causality_num` already provides one; if not, it
  belongs there, not in the FFT crate) or split real/imaginary slices.
- **Real input**: the Poisson RHS is real-valued. A real-to-complex transform
  (rFFT) halves the work and memory versus a complex FFT of real data. Worth
  speccing from the start since the primary consumer is purely real.
- **Plan/twiddle caching**: twiddle factors and bit-reversal permutations
  should be computed once per grid shape and reused across solver steps,
  mirroring the memoization pattern just established for boundary matrices
  (`boundary_cache`). A `Plan`-style API (create once, execute many) fits the
  solver's step loop and keeps the hot path allocation-free, consistent with
  the perf pass.
- **Normalization**: pin the convention explicitly (suggest: unnormalized
  forward, 1/N on inverse) and document it; the Poisson scaling must
  round-trip exactly.
- **`parallel` feature**: same opt-in Rayon pattern as topology/physics
  (forwarded feature, `MaybeParallel` bound, measured granularity thresholds —
  small transforms stay serial, the 24³ lesson from the perf pass applies).

## 5. Algorithm scope

- **Stage 1 (sufficient for the solver today)**: iterative radix-2 (or
  radix-4) Cooley-Tukey for power-of-two lengths. The benchmark and example
  grids are 16³/32³/64³, all power-of-two per axis.
- **Multi-dimensional**: row-column decomposition — batched 1-D transforms
  along each axis with strided access; this is also where the `parallel`
  feature engages.
- **Later, only if needed**: mixed-radix or Bluestein for arbitrary lengths
  (would unblock non-power-of-two periodic grids). Do not spec this into
  Stage 1.

## 6. Integration sketch

- New crate `deep_causality_fft` (zero deps beyond `deep_causality_num`).
- `deep_causality_topology` gains a spectral Poisson path: when the complex is
  a periodic `LatticeComplex`, `leray_project` (and the grade-0 branch of the
  Hodge decomposition) can dispatch to the FFT solve instead of CG. Dispatch
  policy — automatic when periodic, or caller-selected via
  `HodgeDecomposeOptions` — is a spec decision.
- The CG path stays as-is for everything non-periodic; the existing tolerance
  and iteration-budget options simply do not apply to the spectral path.
- Verification anchor: the spectral and CG solves must agree on the same
  periodic problem to within the CG tolerance; the Taylor-Green vortex has a
  closed-form decay solution already used by the physics tests.

## 7. Decisions (questions resolved 2026-06-12)

1. **Complex type**: `deep_causality_num` already provides it.
   `Complex<T: RealField>` (`deep_causality_num/src/complex/complex_number/`)
   with the `ComplexField<R>` trait (`src/algebra/field_complex.rs`) supplying
   `conjugate`, `norm_sqr`, `from_re_im`, etc. The number tower deliberately
   separates the real type, the `RealField` algebra trait, and the adjacent
   complex field. The FFT crate consumes these; it defines no numeric types of
   its own.
2. **rFFT is in Stage 1** — the primary consumer (Poisson RHS) is real-valued.
3. **Algorithm**: follow `fft_state_of_the_art.md` §5 — layered build:
   iterative radix-2 correctness baseline (validated against a naïve O(n²)
   DFT), radix-4 / mixed radix-2-4 power-of-two workhorse, hardcoded
   straight-line butterflies for small N, Bluestein fallback for arbitrary
   lengths. Do not chase the Johnson–Frigo flop record: regular,
   cache-friendly access beats flop-minimal split-radix on real hardware.
   Inverse FFT by conjugation reuse of the forward path (conjugate → forward
   → conjugate → scale 1/N), never a separate kernel.
4. **Plan cache placement**: wherever profiling shows the most gain — a
   performance decision for the implementation, not a design constraint.
   Candidates: beside `boundary_cache` in the manifold, or owned by
   `DecNsSolver`.
5. **Benchmarks**: a dedicated stand-alone criterion benchmark in the new FFT
   crate (transform sizes spanning the solver grids), *and* an update to
   `deep_causality_physics/benches/dec_solver_benchmark.rs` so the spectral
   projection's effect on the full solver step is measured directly
   (currently 388 ms at 32³).
6. **Dispatch policy** (CG vs. spectral inside `leray_project`) remains a spec
   decision for the proposal.
