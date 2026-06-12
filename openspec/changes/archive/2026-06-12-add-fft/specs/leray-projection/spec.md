## MODIFIED Requirements

### Requirement: Leray projection as a first-class half-decomposition entry point
`Manifold<K, R>` (for `K::Metric: HasHodgeStar<R>`) SHALL provide
`leray_project(field)` for 1-forms, computing `P(ω) = ω − d(Δ₀⁻¹ δω)` via a single
gauge-fixed grade-0 solve (mean subtraction, as in the existing decomposition),
without invoking the β-step. On fully periodic lattices the grade-0 solve SHALL
be the spectral (FFT) solve — exact to rounding, with no tolerance, iteration
budget, or convergence-failure mode; the CG tolerance and iteration options are
unused on this path. On all other manifolds the grade-0 solve SHALL remain the
gauge-fixed CG solve with unchanged semantics, and CG non-convergence SHALL
propagate as a typed error. Dispatch SHALL be automatic from the lattice's
periodicity; no new options surface is introduced. The grade-0 potential SHALL
be retrievable alongside the projected field (the later pressure-recovery
diagnostic consumes it; this change does not emit pressure).

#### Scenario: Projection annihilates exact gradients
- **WHEN** `leray_project` is applied to `d φ` for a sampled smooth potential `φ` on a torus
- **THEN** the result has norm at rounding level for the precision (spectral path)

#### Scenario: Projection exactness across precisions
- **WHEN** `leray_project` is applied to an arbitrary smooth 1-form sample
- **THEN** the discrete divergence of the result is at rounding level on fully periodic lattices, and at or below the CG tolerance on non-periodic lattices, at f32 and f64 (with the existing per-backend tolerance clamping where CG runs)

#### Scenario: Idempotency
- **WHEN** `leray_project` is applied twice in succession
- **THEN** the second application changes the field by at most rounding level on the spectral path (CG tolerance on the CG path)

#### Scenario: Harmonic mean flow is retained on the torus
- **WHEN** a constant (harmonic) 1-form component is present in the input on a periodic lattice
- **THEN** it appears unchanged in the projected output (the projector removes only the gradient part; the mean flow is divergence-free and conserved)

#### Scenario: Solver path is independent of the β-step
- **WHEN** `leray_project` runs on a periodic lattice where the full decomposition's β-step would be singular
- **THEN** it succeeds, demonstrating the half-decomposition's well-posedness on tori

#### Scenario: Spectral dispatch follows periodicity
- **WHEN** `leray_project` runs on a fully periodic lattice, a mixed-periodicity lattice, and an open lattice
- **THEN** the fully periodic case takes the spectral path while the mixed and open cases take the CG path with behavior identical to the pre-change implementation

#### Scenario: Spectral and CG projections agree
- **WHEN** the same 1-form on a fully periodic lattice is projected via the spectral path and via the (retained, internally invokable) CG path
- **THEN** the projected fields and grade-0 potentials agree within the CG tolerance
