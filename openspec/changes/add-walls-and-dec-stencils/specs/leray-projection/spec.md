## MODIFIED Requirements

### Requirement: Leray projection as a first-class half-decomposition entry point
`Manifold<K, R>` (for `K::Metric: HasHodgeStar<R>`) SHALL provide
`leray_project(field)` for 1-forms, computing `P(ω) = ω − d(Δ₀⁻¹ δω)` via a single
gauge-fixed grade-0 solve (mean subtraction, as in the existing decomposition),
without invoking the β-step. The grade-0 solve dispatches by domain:

* **Fully periodic uniform Euclidean lattices**: the spectral (DFT) solve —
  exact to rounding, no tolerance, iteration budget, or convergence-failure
  mode.
* **Uniform Euclidean lattices whose axes are each periodic or walled**:
  the direct Neumann solve (DFT/DCT per axis) with no-flux wall semantics —
  same exactness properties.
* **All other manifolds** (per-edge metrics, non-uniform geometry,
  simplicial): the gauge-fixed CG solve with unchanged semantics —
  Jacobi-preconditioned where the boundary-corrected diagonal is available
  — and CG non-convergence SHALL propagate as a typed error.

Dispatch SHALL be automatic from the lattice's periodicity and metric; no
new options surface is introduced. The grade-0 potential SHALL be
retrievable alongside the projected field (the pressure-recovery diagnostic
consumes it).

#### Scenario: Projection annihilates exact gradients
- **WHEN** `leray_project` is applied to `d φ` for a sampled smooth potential `φ` on a torus
- **THEN** the result has norm at rounding level for the precision (spectral path)

#### Scenario: Projection exactness across precisions
- **WHEN** `leray_project` is applied to an arbitrary smooth 1-form sample
- **THEN** the discrete divergence of the result is at rounding level on direct-solve domains (periodic and uniform walled), and at or below the CG tolerance elsewhere, at f32 and f64 (with the existing per-backend tolerance clamping where CG runs)

#### Scenario: Idempotency
- **WHEN** `leray_project` is applied twice in succession
- **THEN** the second application changes the field by at most rounding level on direct-solve domains (CG tolerance on the CG path)

#### Scenario: Harmonic mean flow is retained on the torus
- **WHEN** a constant (harmonic) 1-form component is present in the input on a periodic lattice
- **THEN** it appears unchanged in the projected output (the projector removes only the gradient part; the mean flow is divergence-free and conserved)

#### Scenario: Solver path is independent of the β-step
- **WHEN** `leray_project` runs on a periodic lattice where the full decomposition's β-step would be singular
- **THEN** it succeeds, demonstrating the half-decomposition's well-posedness on tori

#### Scenario: Dispatch follows domain class
- **WHEN** `leray_project` runs on a fully periodic lattice, a uniform walled/mixed lattice with the corrected star, and a per-edge-metric lattice
- **THEN** the first takes the DFT path, the second the DFT/DCT Neumann path, and the third the (preconditioned) CG path with unchanged error semantics

#### Scenario: Direct solves agree with CG
- **WHEN** the same 1-form is projected via a direct path and via the retained, internally invokable CG path on the same manifold (periodic and walled cases)
- **THEN** the projected fields and grade-0 potentials agree within the CG tolerance

#### Scenario: No flux through walls
- **WHEN** `leray_project` runs on a walled lattice
- **THEN** the removed gradient component has vanishing wall-normal boundary trace to rounding
