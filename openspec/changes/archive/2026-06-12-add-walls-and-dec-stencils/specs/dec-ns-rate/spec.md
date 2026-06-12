## MODIFIED Requirements

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
