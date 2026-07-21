## MODIFIED Requirements

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

**Axial means along the flight-velocity direction `v̂`**, resolved from the carried truth state — the
same `v̂` `RetroThrust` thrusts against, not a fixed coordinate axis. The stage MUST project the
force channel onto `v̂` to obtain the axial drag, scale that projection, and leave the components
orthogonal to `v̂` untouched. A fixed-axis reading is only correct while the flight path happens to
lie along that axis, and silently decrements the wrong component the moment it does not.

`PlumeObstruction` MUST compose **before** `RetroThrust`. Both write along `−v̂`, and the preserved
fraction is a decrement on the *aerodynamic drag alone*; applied after the thrust term is on the
channel, it would scale the thrust as well.

#### Scenario: The decrement follows the correlation at the commanded throttle

- **WHEN** `PlumeObstruction` runs at a positive commanded throttle with a forebody drag on the
  channel
- **THEN** the axial drag is scaled by `srp_preserved_drag_fraction_kernel(C_T)` for that world's
  `C_T`, and the lateral force components are unchanged

#### Scenario: Axial is resolved from the flight velocity, not a fixed axis

- **WHEN** the flight velocity is not aligned with any coordinate axis
- **THEN** the scaled component is the projection of the force channel onto `v̂`, and the components
  orthogonal to `v̂` are unchanged

#### Scenario: Zero throttle is strictly inert

- **WHEN** `PlumeObstruction` runs at a commanded throttle ≤ 0 (or with no throttle commanded)
- **THEN** the force channel, scalars, and provenance log are untouched
