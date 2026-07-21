# plume-obstruction-stage Specification

## Purpose
TBD - created by archiving change add-retropulsion-coupled-stages. Update Purpose after archive.
## Requirements
### Requirement: The A0 correlation is the in-flight drag authority

`PlumeObstruction` SHALL be a `PhysicsStage` that, at a commanded throttle > 0, contracts the plume's
preserved-drag fraction from the cited Jarvinen–Adams correlation and applies it to the forebody drag
on the aero-force channel: derive the thrust and `C_T = srp_thrust_coefficient_kernel(T, q∞, S_ref)`,
read `srp_preserved_drag_fraction_kernel(C_T)`, and scale the axial forebody drag on the channel by
the preserved fraction. Per the measured de-risk verdict
(`openspec/notes/cfd-plasma-retropulsion/derisk-verdict.md`, AMBER; the 2026-07-17 addendum measured
both coupling models and pinned the missing collapse to the harness, not the model class), this
correlation is the **committed drag authority** in flight — not a field-contracted decrement. At a
commanded throttle ≤ 0 (or absent) the stage MUST be strictly inert.

**`q∞` is sensed, not supplied.** The stage MUST read the freestream dynamic pressure from the field
each step, from the same scalar the safety gate's burn sensing reads, so the drag closure and the
envelope's dynamic thrust-coefficient cap describe one physical state. A `q∞` fixed at construction
let the two normalize the same coefficient against pressures 5.4× apart in the same step — the closure
evaluating the correlation deep in its shallow range while the gate simultaneously judged the vehicle
to be at its stability limit. An absent or non-positive sensed value MUST be an error rather than a
fallback, because a silent fallback is how a second normalization survives unnoticed.

**Inertness clears, it does not merely skip.** A stage that stops applying a decrement — at zero
throttle or outside its applicability band — MUST remove any previously published preserved-drag
fraction. A stale fraction left on the field reads to every consumer as a live measurement, so a
vehicle that shut its engine down would otherwise carry its last decrement forward indefinitely.
Removing a scalar that was never published is a no-op, so the strict-inertness contract holds for a
world that never burned.

**Axial means along the flight-velocity direction `v̂`**, resolved from the carried truth state — the
same `v̂` `RetroThrust` thrusts against, not a fixed coordinate axis. The stage MUST project the
force channel onto `v̂` to obtain the axial drag, scale that projection, and leave the components
orthogonal to `v̂` untouched.

`PlumeObstruction` MUST compose **before** `RetroThrust`. Both write along `−v̂`, and the preserved
fraction is a decrement on the *aerodynamic drag alone*; applied after the thrust term is on the
channel, it would scale the thrust as well.

#### Scenario: The decrement follows the correlation at the sensed dynamic pressure

- **WHEN** `PlumeObstruction` runs at a positive commanded throttle with a forebody drag on the
  channel
- **THEN** the axial drag is scaled by `srp_preserved_drag_fraction_kernel(C_T)` for the `C_T` formed
  from the **sensed** `q∞`, and the lateral force components are unchanged

#### Scenario: Closure and envelope agree on C_T within a step

- **WHEN** the plume stage and the safety gate both evaluate the thrust coefficient on the same step
- **THEN** both use the same sensed dynamic pressure and produce the same `C_T` for the same thrust

#### Scenario: An absent dynamic-pressure sensor fails the step

- **WHEN** the stage runs under an active throttle on a field carrying no dynamic-pressure scalar, or
  a non-positive one
- **THEN** the stage returns an error rather than substituting a default

#### Scenario: A shut-down engine carries no decrement forward

- **WHEN** a world that was burning commands zero throttle on a later step
- **THEN** the published preserved-drag fraction is removed, so no consumer reads it as live

#### Scenario: Zero throttle is strictly inert

- **WHEN** `PlumeObstruction` runs at a commanded throttle ≤ 0 on a world that never published a
  preserved-drag fraction
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

### Requirement: The A0 correlation applies only inside its validated Mach band

`PlumeObstruction` SHALL stand down outside a configurable flight-Mach applicability band: it applies
no drag decrement, publishes no plume geometry, and records one provenance entry per crossing. The
Jarvinen–Adams correlation's mechanism is **bow-shock displacement** — the plume pushes the standoff
shock away and the high post-shock pressure decelerating the forebody is replaced by low-pressure
recirculating plume gas. Below the dataset's Mach floor there is no bow shock to displace, so the
correlation describes nothing there. Carried down unbounded it deleted roughly 78% of a subsonic
vehicle's aerodynamic drag on the strength of a supersonic interaction.

The band MUST be opt-in, so a world that configures none behaves as before.

#### Scenario: A subsonic leg carries no A0 decrement

- **WHEN** a burn runs at a positive throttle below the band's Mach floor
- **THEN** the aerodynamic drag on the force channel is unmodified by the plume stage, and provenance
  records that the closure stood down

#### Scenario: Re-entering the band resumes the closure

- **WHEN** the sensed flight Mach crosses back inside the band under an active throttle
- **THEN** the decrement resumes and a second provenance entry records the crossing

#### Scenario: An unbounded stage applies at every Mach

- **WHEN** a world configures no applicability band
- **THEN** the decrement applies wherever the throttle is positive

### Requirement: The plume geometry is built from the sensed freestream

The Cordell–Braun plume boundary SHALL take its freestream Mach number and static pressure from the
flown state rather than from constants fixed at construction, so the kernel's own validity envelope
tests the flight. The geometry publication MUST additionally carry its **own** applicability band,
declared separately from the drag correlation's.

The two bands are separate because the two models are: the drag correlation is measured over Mach
0.4–2.0 and the plume-boundary model is validated over Mach 2–4. They meet at a single point, so a
descent flying the correlation's band is outside the geometry model's for essentially all of it.
Declaring both at the call site makes that disjointness legible rather than leaving it to be
discovered when the kernel refuses mid-flight — or hidden by handing the kernel a constant that sits
inside its envelope while the vehicle does not.

Freestream static pressure requires a producer: the compressible carrier MUST publish the freestream
temperature it already holds, and the flight-sensor stage MUST derive `p∞ = n∞·k_B·T∞` beside the
`q∞` it produces.

#### Scenario: Geometry tracks the descent

- **WHEN** the vehicle descends through two altitudes with materially different static pressure at one
  throttle
- **THEN** the published plume radius and penetration differ between the two steps

#### Scenario: The kernel's envelope refusal reaches the caller

- **WHEN** the flown freestream Mach leaves the kernel's documented envelope inside the declared
  geometry band
- **THEN** the kernel's rejection surfaces as a step error rather than being masked by a constant

#### Scenario: Outside its own band the geometry stands down

- **WHEN** the sensed flight Mach lies outside the geometry model's declared band
- **THEN** no plume geometry is published, any previously published geometry is removed, and the
  crossing is recorded once
