# plume-obstruction-stage Specification

## Purpose
TBD - created by archiving change add-retropulsion-coupled-stages. Update Purpose after archive.
## Requirements
### Requirement: The A0 correlation is the in-flight drag authority

`PlumeObstruction` SHALL be a `PhysicsStage` that, at a commanded throttle > 0, contracts the plume's
preserved-drag fraction from the cited Jarvinen–Adams correlation and applies it to the forebody drag
on the aero-force channel: derive the thrust and `C_T = srp_thrust_coefficient_kernel(T, q∞, S_ref)`
(with `momentum_flux_ratio_kernel` sizing the plume input from the world's throttle), read
`srp_preserved_drag_fraction_kernel(C_T)`, and scale the axial forebody drag on the channel by the
preserved fraction. Per the measured de-risk verdict
(`openspec/notes/cfd-plasma-retropulsion/derisk-verdict.md`, AMBER; the 2026-07-17 addendum measured
both coupling models and pinned the missing collapse to the harness, not the model class), this
correlation is the **committed drag authority** in flight — not a field-contracted decrement. At a
commanded throttle ≤ 0 (or absent) the stage MUST be strictly inert.

#### Scenario: The decrement follows the correlation at the commanded throttle

- **WHEN** `PlumeObstruction` runs at a positive commanded throttle with a forebody drag on the
  channel
- **THEN** the axial drag is scaled by `srp_preserved_drag_fraction_kernel(C_T)` for that world's
  `C_T`, and the lateral force components are unchanged

#### Scenario: Zero throttle is strictly inert

- **WHEN** `PlumeObstruction` runs at a commanded throttle ≤ 0 (or with no throttle commanded)
- **THEN** the force channel, scalars, and provenance log are untouched

### Requirement: The optional marched imprint is state realism, never the drag closure

The optional marched-layer plume imprint SHALL be **state realism only** and MUST NOT replace or
alter the correlation as the force-channel drag closure: the drag decrement applied to the channel is
the A0 preserved-drag fraction whether or not the imprint is active, and a world that does not opt in
marches with no plume imprint and the same force-channel decrement.

Because a `PhysicsStage` mutates only the `CoupledField` while the `ForcingRegion` lives on the
carrier's march path, the imprint SHALL be driven through the carrier's **existing** field-reading
reconfiguration channel — the same `pre_step` path that already reads a stage-written scalar
(`"truth_state"`) to set the inflow strip and rebuild the marcher — and not through a new
stage→marcher coupling. `PlumeObstruction` SHALL publish the plume geometry it derived from that
world's `C_T` (via `cordell_braun_plume_boundary_kernel`) as named scalars on the field; the carrier,
**behind an explicit opt-in**, SHALL read those scalars in `pre_step` and refresh its forcing region
so the imprint follows a varying throttle.

#### Scenario: The imprint does not change the force-channel decrement

- **WHEN** two otherwise-identical worlds run `PlumeObstruction` at the same throttle, one with the
  marched imprint opted in and one without
- **THEN** the aero-force-channel drag decrement is identical between them (the correlation is the
  authority); only the marched field differs

#### Scenario: The stage publishes the plume geometry for the carrier to read

- **WHEN** `PlumeObstruction` runs at an active throttle
- **THEN** the field carries the plume's max radius and penetration length derived from
  `cordell_braun_plume_boundary_kernel` at that world's `C_T`, as named scalars

### Requirement: The imprint refreshes only on a throttle drift, logged and capped

The carrier's plume re-imprint SHALL reuse the solver-rebuild discipline it already applies to the
acoustic envelope: with the imprint opted in, the forcing region is refreshed only when the commanded
throttle drifts past a configured tolerance, each refresh MUST be logged to the provenance log, and
the refresh count MUST be bounded by a configured cap so a noisy throttle cannot rebuild the mask
every step. With **no** imprint opt-in the carrier's forcing region MUST be left exactly as configured
at world build, so the unforced march path stays bit-identical.

#### Scenario: A small throttle change does not rebuild the mask

- **WHEN** the commanded throttle moves by less than the configured re-imprint tolerance between
  steps
- **THEN** the forcing region is not refreshed and no re-imprint entry is logged

#### Scenario: A throttle drift past tolerance refreshes and logs

- **WHEN** the commanded throttle drifts past the tolerance
- **THEN** the carrier refreshes the forcing region from the published plume geometry and records a
  re-imprint entry in the provenance log

#### Scenario: Without the opt-in the march path is untouched

- **WHEN** a world runs `PlumeObstruction` with no imprint opt-in configured
- **THEN** the carrier's forcing region is exactly the world-build value and the marched state is
  bit-identical to the same world without the stage's geometry scalars

### Requirement: The in-flight decrement is cross-checked against M1's measured band

`PlumeObstruction`'s in-flight contracted decrement SHALL be verifiable against the M1 measured band:
the applied preserved-drag fraction at a swept `C_T` MUST equal `srp_preserved_drag_fraction_kernel`
within the correlation's digitization tolerance, so the flight-time authority is the same cited curve
M1 gated against, not a re-derived or field-contracted number.

#### Scenario: The applied fraction matches the correlation within tolerance

- **WHEN** a burn-leg harness sweeps `C_T` and reads the fraction `PlumeObstruction` applied at each
  point
- **THEN** each applied fraction equals `srp_preserved_drag_fraction_kernel(C_T)` within the
  digitization tolerance

