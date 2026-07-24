## ADDED Requirements

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
