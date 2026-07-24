# dec-ns-rate

## Purpose

The DEC right-hand side of incompressible Navier–Stokes in rotational
(Lamb) form under Leray projection: the marched rate is
`P(−i_u(du♭) − ν Δ_dR u♭ + g♭)` (the projector inside the rate, per the
governing equation of `cfd-gap.md` §2), assembled from the Stage 0
operators on a metric-bearing periodic `Manifold<LatticeComplex<D, R>, R>`.
The unprojected assembly is exposed separately for cross-validation and
the pressure diagnostic.

## Requirements
### Requirement: Rotational-form rate assembly

The crate SHALL provide a rate evaluator in
`deep_causality_physics::theories::fluid_dynamics::dec` with two surfaces:
an **unprojected assembly** returning `−i_u(du♭) − ν Δ_dR u♭ + g♭` and the
**projected rate** returning its Leray projection (one gauge-fixed grade-0
solve — direct-spectral on uniform periodic/walled lattices, CG elsewhere;
fallible on the CG path). The convective and viscous terms SHALL realize
the `exterior_derivative`/`interior_product` composition and `laplacian(1)`
with the Stage 0 sign pin (`Δ_dR = −∇²`, hence the minus sign realizes
`+ν∇²u`); on cubical lattices the evaluation MAY run through the compiled
stencil pipeline and the fused workspace, whose equivalence to the generic
composition is test-gated (the `dec-stencil-operators` capability). On
fully periodic lattices the viscous term MAY be evaluated spectrally when
opted in (the `spectral-diffusion` capability). The projected rate SHALL
return a divergence-free 1-form (at the solve's exactness) and surface
solve failure as an error where the solve is fallible. The evaluator SHALL
be generic over `R: RealField` with no concrete float named in its
signature.

#### Scenario: Convective term matches the Stage 0 cross-validation oracle

- **WHEN** the unprojected assembly is evaluated on the sampled 2D
  Taylor–Green field over the refinement ladder `[8, 16, 32]`
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

- **WHEN** the same state is evaluated through the unprojected assembly
  with and without a body-force 1-form
- **THEN** the difference of the two results equals the body-force
  coefficients to machine rounding

#### Scenario: The projected rate is divergence-free and surfaces CG failure

- **WHEN** the projected rate is evaluated on a per-edge-metric lattice
  (the CG path) with a healthy budget, and again with a one-iteration
  budget
- **THEN** the first returns a 1-form whose divergence residual is at CG
  tolerance, and the second returns the wrapped projection error

#### Scenario: Stencil and generic assemblies agree

- **WHEN** the unprojected assembly runs through the compiled stencil
  pipeline and through the generic operator composition on the same state
- **THEN** the results agree within 100·ε of the scalar at f64 and Float106

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
