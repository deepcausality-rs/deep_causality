## ADDED Requirements

### Requirement: Spectral grade-0 Poisson solve on fully periodic lattices
`deep_causality_topology` SHALL provide a spectral solve of the gauge-fixed
grade-0 Poisson problem `Δ₀ φ = rhs` on fully periodic lattices
(`LatticeComplex` with `periodic()` all true): forward real FFT of the
right-hand side, pointwise division of bin `k` by the lattice Laplacian
eigenvalue `λ_k = Σ_d (2 − 2·cos(2π·k_d/N_d)) / h_d²`, zeroing of the `k = 0`
bin (the spectral expression of the mean-subtraction gauge fix), and inverse
real FFT. The eigenvalues SHALL correspond exactly to the discrete `Δ₀`
operator the CG path applies on the same manifold, including the metric
scaling encoded by the Hodge star. The solve SHALL have no tolerance,
iteration budget, or convergence-failure mode.

#### Scenario: Spectral and CG solves agree
- **WHEN** the same grade-0 Poisson problem on a fully periodic lattice is solved spectrally and by the existing CG path, across multiple shapes including anisotropic spacings
- **THEN** the two solutions agree within the CG tolerance

#### Scenario: Residual is exact to rounding
- **WHEN** `Δ₀` is applied to the spectral solution `φ`
- **THEN** the residual against the (mean-free) right-hand side is at rounding level for the precision, not merely at a CG tolerance

#### Scenario: Gauge fix via the zero mode
- **WHEN** the spectral solve completes
- **THEN** the returned potential has zero mean, matching the existing `subtract_mean_in_place` gauge convention

### Requirement: Per-shape spectral plan reuse
The FFT plans and eigenvalue table for a given lattice shape SHALL be
computed once and reused across repeated solves on that shape (cached in the
manifold beside the existing operator caches, or owned by the solver — the
placement is settled by benchmark measurement during implementation). Repeated
solves SHALL NOT rebuild plans or eigenvalue tables, and SHALL NOT allocate
per solve beyond the cached state.

#### Scenario: Plans survive repeated solves
- **WHEN** the spectral Poisson solve runs many times on the same lattice shape (as the NS solver's step loop does)
- **THEN** plan and eigenvalue construction occurs only once and subsequent solves perform no per-solve heap allocation

#### Scenario: Cache survives Clone where manifold-cached
- **WHEN** the cache lives in the manifold and the manifold is cloned
- **THEN** the spectral plan state is preserved across the clone, consistent with the existing lattice cache behavior

### Requirement: Solver-level benchmark coverage
`deep_causality_physics/benches/dec_solver_benchmark.rs` SHALL be extended to
measure the Leray projection and full solver step with the spectral path
active, against the same grids as the existing CG measurements (16³/32³).

#### Scenario: Benchmark demonstrates the spectral speedup
- **WHEN** the updated solver benchmark runs at 32³ f64
- **THEN** it reports spectral-projection and full-step timings comparable against the recorded CG baseline (full step 388 ms)
