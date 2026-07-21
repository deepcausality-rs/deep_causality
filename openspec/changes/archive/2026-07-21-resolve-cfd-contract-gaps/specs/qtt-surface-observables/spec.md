## MODIFIED Requirements

### Requirement: Neutral wall heat flux from a penalized passive scalar

The `solvers/qtt` module SHALL optionally advect–diffuse a passive scalar `T` on the same rollout, with
the body penalized to a wall temperature `T_w`, and SHALL expose the penalization heat integral
`Q = (1/η) ∫ χ_body ⊙ (T_w − T) dV` (the same contraction shape as drag). This SHALL be
**neutral** (no chemistry) — the seam the Gap-2 reacting energy equation replaces.

The observable's exposed name and its published series key SHALL describe the quantity actually
computed. The integral above is a temperature-weighted **volumetric rate** with units `[T]·[L]²/[t]`;
it carries no gradient, no conductivity and no surface normal, so it is not a heat flux — Fourier's law
is `q = −k·∂T/∂n`, and no scaling converts a volume integral of a temperature deficit into a flux.
Exposing it as `wall_heat_flux` invites exactly the absolute reading its own docstring disclaims, and
for a re-entry thermal-protection consumer wall heat flux is the safety-critical quantity.

The wall temperature `T_w` the quantity is defined against SHALL be configurable. It is currently
hardcoded to zero at the production call site and is not a field of the march configuration, so the
observable is reported against a wall temperature the caller cannot set or inspect.

#### Scenario: Wall heat flux responds to the thermal field
- **WHEN** the passive scalar is advected past a body held at a wall temperature different from the flow
- **THEN** a non-zero wall heat-flux observable is produced, computed as the mask–temperature-deficit
  contraction

#### Scenario: The name states the computed quantity
- **WHEN** a consumer reads the observable's name or its published series key
- **THEN** the name describes the penalization heat integral rather than a surface flux, so an absolute
  reading is not invited

#### Scenario: The wall temperature is set by the caller
- **WHEN** a case is configured
- **THEN** `T_w` is part of that configuration and appears in the run's record, rather than being
  hardcoded at the call site

#### Scenario: A consumer of the series moves with the rename
- **WHEN** the published series key changes
- **THEN** every in-repo consumer reads the new key, and no consumer silently observes an absent series
