# dec-scalar-transport Specification

## Purpose
Passive scalar advection–diffusion on the DEC manifold — the temperature field a genuine Fourier-law
wall heat flux differentiates. The scalar reuses the momentum path's operators at a different grade
rather than carrying a parallel discretisation. Created by archiving change
`add-dec-scalar-transport-wall-heat-flux`.

## Requirements
### Requirement: Scalar advection–diffusion on the DEC manifold

The DEC solver SHALL march a scalar field `T` as a **0-cochain** on the same manifold the velocity
marches, under `∂T/∂t = −i_u(dT) − κ·Δ_dR T`, where `i_u` is the interior product of the velocity
1-cochain with the 1-form `dT`, and `Δ_dR` is the Hodge–de Rham Laplacian on 0-forms. The diffusive sign
SHALL follow the crate's Stage-0 pin — on a flat torus `Δ_dR = −∇²`, so physical diffusion `+κ∇²T`
enters as `−κ·Δ_dR T`, matching the viscous term of the momentum rate.

The scalar SHALL reuse the manifold's existing operators rather than introducing a parallel
discretisation, so that a scalar and a velocity component are differentiated by the same code.

#### Scenario: Pure diffusion decays a Fourier mode at the analytic rate
- **WHEN** a single Fourier mode `T = cos(kx)` is diffused with zero velocity
- **THEN** its amplitude decays as `exp(−κk²t)` to the scheme's order

#### Scenario: Pure advection transports a scalar without diffusing it
- **WHEN** a scalar is advected by a uniform divergence-free velocity with `κ = 0`
- **THEN** the scalar is translated, and its total integral is conserved to round-off

#### Scenario: A constant field is stationary
- **WHEN** a spatially constant `T` is marched under any velocity and any `κ`
- **THEN** it is unchanged, since both `dT` and `Δ_dR T` vanish identically

### Requirement: A Dirichlet wall temperature on the immersed body

The scalar march SHALL support pinning `T` to a wall temperature `T_w` on the immersed body, so the
field carries a boundary condition against which a wall gradient is defined. Absent such a condition a
wall heat flux is not merely inaccurate but undefined, since there is no wall value to difference
against.

The pinned set SHALL be derived from the same cut-cell geometry the momentum no-slip uses, so the
thermal and mechanical boundaries describe the same body.

#### Scenario: The wall holds its temperature
- **WHEN** a body is pinned to `T_w` and the scalar is marched
- **THEN** the constrained degrees of freedom remain at `T_w` for every step

#### Scenario: A body hotter than the fluid heats it
- **WHEN** a body is pinned above the surrounding fluid temperature and the scalar is marched
- **THEN** the fluid temperature near the body rises toward `T_w`, and the far field lags it
