## ADDED Requirements

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
alter the correlation as the force-channel drag closure. `PlumeObstruction` MAY, behind an explicit
opt-in, imprint the analytic plume on the marched compressible layer through the landed
`ForcingRegion` seam (a `plume_mask_2d` mask from `cordell_braun_plume_boundary_kernel` at the
world's `C_T`), but the drag decrement applied to the channel is the A0 preserved-drag fraction
whether or not the imprint is active; a world that does not opt in marches with no plume imprint and
the same force-channel decrement.

#### Scenario: The imprint does not change the force-channel decrement

- **WHEN** two otherwise-identical worlds run `PlumeObstruction` at the same throttle, one with the
  marched imprint opted in and one without
- **THEN** the aero-force-channel drag decrement is identical between them (the correlation is the
  authority); only the marched field differs

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
