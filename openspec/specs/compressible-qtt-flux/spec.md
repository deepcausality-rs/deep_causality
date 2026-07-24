# compressible-qtt-flux Specification

## Purpose
TBD - created by archiving change add-cfd-compressible-qtt-marcher. Update Purpose after archive.
## Requirements
### Requirement: Conservative compressible state and flux

`deep_causality_cfd` SHALL represent the compressible state in **conservative variables** `(ρ, ρu, ρv, ρw,
ρE, {ρY_s})` as tensor trains, and SHALL compute the conservative flux divergence `∇·F(U)` in tensor-train
form using the `qtt-codec-3d` / body-fitted operators. The scheme SHALL be **conservative** (the discrete flux
divergence telescopes), so captured/fitted jumps satisfy the Rankine–Hugoniot conditions.

#### Scenario: Conservation of total mass / momentum / energy
- **WHEN** the compressible update is marched on a periodic domain with no sources
- **THEN** the integrated `∫ρ`, `∫ρu`, `∫ρE` are conserved to the rounding floor over the run

#### Scenario: Free-stream preservation
- **WHEN** a uniform free-stream state is marched
- **THEN** it is preserved exactly (the flux of a constant state is zero), including in the body-fitted
  coordinate (metric identities hold discretely)

### Requirement: Approximate Riemann flux

The flux SHALL use an approximate Riemann solver (Rusanov/local Lax–Friedrichs as the baseline, HLLC as the
sharper option) expressed through the existing tensor-train operations, with a wave-speed estimate from the
state. On a smooth field the flux SHALL reduce to a centred flux to the scheme's order.

#### Scenario: Sod shock tube matches the exact Riemann solution
- **WHEN** the 1-D compressible flux is marched on the Sod shock-tube initial data
- **THEN** the density / velocity / pressure profiles match the exact Riemann solution within a recorded
  tolerance, with the shock, contact, and expansion at the correct speeds

### Requirement: EOS pressure closure via TT-cross

The marcher SHALL evaluate the pressure and temperature closure (`p, T` from density, internal energy, and
species fractions — ideal-gas baseline; the Tier-A two-temperature mixture EOS as the reacting option)
pointwise on the tensor-train state via TT-cross (`apply_nonlinear` / `cross`) at a controlled rank on smooth
fields.

#### Scenario: EOS recovers pressure on a smooth field
- **WHEN** the EOS closure is evaluated on a smooth density/energy train
- **THEN** the decoded pressure matches the pointwise analytic EOS within tolerance, at bounded rank

### Requirement: Non-positive pressure is rejected, not floored into the flux

Every compressible marcher SHALL reject a non-positive or non-finite pressure the same way it already
rejects a non-positive density: with `PhysicsError::PhysicalInvariantBroken` naming the offending
quantity. A floored value SHALL NOT be substituted into the flux.

All four marchers — `euler_1d`, `marcher_2d`, `marcher_3d`, `marcher_3d_fitted` — currently enforce
density positivity and then push the **unfloored** pressure into the flux components
(`f[1] = mx·vx + p`, `f[3] = (e + p)·vx`) while using the floored `p_floor` only for the acoustic wave
speed `c = √(γ·p_floor/ρ)`. A state with `E < ½ρ|u|²` therefore produces a negative pressure that
enters the momentum and energy fluxes unchallenged, while the wave-speed estimate is quietly repaired.
The ideal-gas EOS is not hyperbolic there, so the scheme's premise does not hold and the result is not
a solution of the equations being solved.

Where a floor is genuinely wanted for robustness rather than a rejection, the same floored value
SHALL be used consistently in the flux and the wave speed, and the fact that the floor engaged SHALL
be reported, so a run whose pressure was clamped is not read as a clean solve.

#### Scenario: A non-hyperbolic state is refused

- **WHEN** a cell's conservative state yields `p ≤ 0` under the EOS
- **THEN** the marcher returns an error naming the pressure and the offending cell, rather than
  computing a flux from it

#### Scenario: The rejection is uniform across the marcher family

- **WHEN** the same non-hyperbolic state is presented to the 1-D, 2-D, 3-D and fitted-3-D marchers
- **THEN** all four reject it, with the same error type and comparable diagnostics

#### Scenario: A floor, if used, is applied consistently and reported

- **WHEN** a robustness floor is applied to the pressure instead of rejecting the state
- **THEN** the floored value is used for both the flux and the wave speed, and the report records that
  the floor engaged

#### Scenario: Valid states are unaffected

- **WHEN** a state with `p > 0` everywhere is marched
- **THEN** the flux and wave speed are unchanged from before this requirement, and results are
  bit-identical
