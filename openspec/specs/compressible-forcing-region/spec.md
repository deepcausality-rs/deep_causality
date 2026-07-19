# compressible-forcing-region Specification

## Purpose
TBD - created by archiving change plasma-retropulsion-de-risk. Update Purpose after archive.
## Requirements
### Requirement: Masked forcing region on the compressible marcher

The compressible QTT march path SHALL accept an optional forcing region — a rank-bounded
smoothed mask over the grid, a target conserved state, and a penalization strength η — and
SHALL drive the state toward the target inside the masked region each step (fused Hadamard
product + round, the incompressible penalization pattern ported to the 4-component
`EulerStateTt2d`), while leaving the exterior to evolve freely so the outer flow forms its own
standoff-shock response. The mask MUST reuse the smoothed-mask codec (tanh volume-fraction
skirt) so its bond dimension stays bounded, and the forced state MUST remain bond-bounded under
the run's truncation policy.

#### Scenario: The interior is driven toward the target

- **WHEN** a compressible march runs with a forcing region over a static mask and target state
- **THEN** the state inside the masked region converges toward the target (to the penalization
  floor) while the exterior field evolves, and the marched state's bond dimension stays under
  the run's cap

#### Scenario: The mask is rank-bounded

- **WHEN** a plume-shaped smoothed mask is built and quantized
- **THEN** its bond dimension is bounded far below the dense element count, and dequantizing
  recovers the smoothed volume fraction within rounding tolerance

### Requirement: Plume-shaped mask from the analytic kernels

The forcing harness SHALL derive the mask geometry and target interior state from the existing
propulsion kernels: the plume boundary from `cordell_braun_plume_boundary` at the world's
thrust coefficient (via `srp_thrust_coefficient`) and momentum-flux ratio (via
`momentum_flux_ratio`), with the commanded throttle entering the world as the published
constant `"commanded_throttle"` — the same name the propulsion coupling contract pins. The
throttle is constant per run or branch, so the mask MAY be static per world; the binaries MUST
state this simplification in their headers.

#### Scenario: The imprint follows the commanded throttle

- **WHEN** two worlds publish different `"commanded_throttle"` values and each is imprinted
- **THEN** each world's mask geometry derives from its own C_T through the plume-boundary
  kernel, and the two imprinted layers differ

### Requirement: No forcing region changes nothing

With no forcing region configured, the compressible march path SHALL be bit-identical to the
pre-change marcher: the same step results, reports, and rebuild behavior on the same
configuration. The corridor example flies this marcher, so this guarantee is guarded by the
standing corridor-inheritance gate (prong A re-runs before this change archives).

#### Scenario: The unforced marcher is bit-identical

- **WHEN** the same compressible world is marched N steps before and after this change with no
  forcing region configured
- **THEN** the marched states and reports are bit-identical

#### Scenario: The corridor still reproduces its witnesses

- **WHEN** the corridor example is re-run after the forcing seam lands
- **THEN** its gate witnesses equal the committed `output.txt` values exactly

