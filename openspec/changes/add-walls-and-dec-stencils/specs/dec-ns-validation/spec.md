## ADDED Requirements

### Requirement: Poiseuille channel rung (CI)
The validation ladder SHALL include body-force-driven laminar Poiseuille
flow (periodic-x, wall-y) marched to steady state, compared against the
exact parabolic profile over a refinement ladder — the analytic-first
gate for the wall substrate (corrected star, Neumann projection, no-slip
rows) before any reference-data comparison.

#### Scenario: Profile converges at second observed order
- **WHEN** the steady-state centerplane profile error is measured over the refinement ladder at f64
- **THEN** the observed convergence order is ≥ 1.9

#### Scenario: Steady state is wall-consistent
- **WHEN** the Poiseuille march reaches its steady-state criterion
- **THEN** wall-tangential edges are exactly zero and the divergence residual is at the solve's exactness

### Requirement: Lid-driven cavity rung (coarse CI + example)
The ladder SHALL include the Re-1000 lid-driven cavity: a coarse rung
(≤ 64²) in CI compared against the Ghia et al. (1982) centerline tables
with a pinned RMSE gate and an asserted refinement trend, and a
full-resolution example program that additionally emits the detected
vortex-center table (primary and corner eddies) against Ghia's values.

#### Scenario: Coarse cavity gates in CI
- **WHEN** the coarse cavity run completes in CI
- **THEN** centerline RMSE against the Ghia tables is within the pinned gate and decreases under refinement

#### Scenario: Example emits centerlines and the vortex table
- **WHEN** the cavity example runs at full resolution
- **THEN** it writes centerline CSVs and the detected vortex centers alongside the Ghia reference values

### Requirement: Stencil-path coverage in the existing ladder
Every existing CI rung (2D Taylor–Green table, 2D-in-3D, inviscid
invariants, double shear layer) SHALL pass through the compiled stencil
pipeline with results matching the generic path at tolerance, so the
ladder gates both evaluation strategies permanently.

#### Scenario: Ladder is strategy-agnostic
- **WHEN** the CI ladder runs with the stencil pipeline enabled
- **THEN** every rung passes with observed orders and conservation gates equal to the generic path's
