# dec-ns-rate

The DEC right-hand side of incompressible Navier–Stokes in rotational
(Lamb) form: `−i_u(du♭) − ν Δ_dR u♭ + g♭`, assembled from the Stage 0
operators on a metric-bearing periodic `Manifold<LatticeComplex<D, R>, R>`.

## ADDED Requirements

### Requirement: Rotational-form rate assembly

The crate SHALL provide a rate evaluator in
`deep_causality_physics::theories::fluid_dynamics::dec` that, given a
velocity edge 1-form on a metric-bearing periodic lattice manifold, a
kinematic viscosity `ν ≥ 0`, and an optional body-force 1-form, returns the
1-form `−i_u(du♭) − ν Δ_dR u♭ + g♭`. The convective term SHALL be composed
from `exterior_derivative` and `interior_product`; the viscous term SHALL
use `laplacian(1)` with the Stage 0 sign pin (`Δ_dR = −∇²`, hence the minus
sign realizes `+ν∇²u`). The evaluator SHALL be generic over
`R: RealField` with no concrete float named in its signature.

#### Scenario: Convective term matches the Stage 0 cross-validation oracle

- **WHEN** the rate is evaluated on the sampled 2D Taylor–Green field with
  `ν = 0` and no body force, over the refinement ladder `[8, 16, 32]`
- **THEN** the result agrees with the pointwise oracle
  (`incompressible_ns_rhs_kernel` fed by tangent-functor derivatives, via
  the Lamb identity) at second observed order, reusing the Stage 0
  capstone's comparison machinery

#### Scenario: Viscous sign produces decay, not growth

- **WHEN** the rate is evaluated with `ν > 0` on a sinusoidal eigenfield of
  the Laplacian and the field is stepped forward by one explicit step
- **THEN** the kinetic energy decreases — the anti-diffusion sign error the
  gap note warns about is structurally excluded by a test

#### Scenario: Body force enters additively

- **WHEN** the same state is evaluated with and without a body-force 1-form
- **THEN** the difference of the two results equals the body-force
  coefficients exactly

### Requirement: Construction-time validation makes the rate infallible

The rate evaluator SHALL validate its preconditions at construction —
metric present on the manifold, velocity length equal to `num_cells(1)`,
body-force length equal when supplied, `ν` finite and non-negative, `dt`
finite and positive where carried — and SHALL return
`PhysicsError` on violation. After successful construction the per-step
evaluation SHALL be infallible (`Fn(&S) -> S`), so it composes directly
with `Rk4`; internal operator `Result`s are unwrapped against the
construction-time invariants, with each unwrap documented as a coverage
exemption in the Stage 0 tradition.

#### Scenario: Mismatched body force is rejected at construction

- **WHEN** a rate is constructed with a body-force tensor whose length is
  not the manifold's edge count
- **THEN** construction returns `PhysicsError::DimensionMismatch` naming
  the expected and actual lengths

#### Scenario: Non-finite viscosity is rejected at construction

- **WHEN** a rate is constructed with `ν` equal to NaN, `+∞`, or a negative
  value
- **THEN** construction returns a `PhysicsError` and no evaluator is
  produced

#### Scenario: Metric-free manifold is rejected at construction

- **WHEN** a rate is constructed over a manifold without a Hodge-star
  metric
- **THEN** construction returns the underlying error instead of deferring
  the failure into the marching loop
