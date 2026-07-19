## ADDED Requirements

### Requirement: RetroThrust composes thrust onto the force channel

`RetroThrust` SHALL be a `PhysicsStage` that, at a commanded throttle > 0, forms the retro-thrust
acceleration `−(T/m)·v̂` — `T = commanded_throttle · thrust_ref`, `m` the carried `"mass"` scalar,
`v̂` the flight-velocity direction (the corridor's motion axis) — and composes it onto the aero-force
channel through `CoupledField::add_aero_force` (never the overwriting `set_aero_force`), so thrust
adds to the lift stage's vector instead of clobbering it. The stage MUST read the throttle from the
throttle channel or, absent one, the world-published `"commanded_throttle"` scalar (the
`PropulsionStub` seam), and MUST error if an active throttle is commanded with a non-positive carried
mass. At a commanded throttle ≤ 0 (or absent) the stage MUST be strictly inert: no force write, no
scalar mutation, no log entry.

#### Scenario: Thrust adds to the lift vector under active throttle

- **WHEN** a lift stage has written the aero force and `RetroThrust` runs at a positive commanded
  throttle over a positive carried mass
- **THEN** the aero-force axial component gains `−T/m` and the lateral lift components are unchanged

#### Scenario: Zero throttle is strictly inert

- **WHEN** `RetroThrust` runs at a commanded throttle ≤ 0 (or with no throttle commanded)
- **THEN** the coupled field's force channel, scalars, and provenance log are untouched

#### Scenario: Active thrust without carried mass is an error

- **WHEN** a positive throttle is commanded but no positive `"mass"` scalar rides the field
- **THEN** the stage returns `Err(PhysicalInvariantBroken)` rather than dividing by a missing mass

### Requirement: RetroThrust depletes propellant and sets ignition

`RetroThrust` SHALL, on the active path, deplete the `"propellant"` and `"mass"` scalars by `ṁ·Δt`
where `ṁ = T/(Isp·g₀)` from `propellant_mass_flow_kernel`, and set the `"ignited"` scalar. Depletion
uses the mass in hand at the start of the step to form the thrust acceleration, so the normalization
is consistent within the step.

#### Scenario: Propellant and mass fall by the kernel's mass flow

- **WHEN** `RetroThrust` runs one active step with `Isp` and a positive thrust
- **THEN** both `"propellant"` and `"mass"` decrease by `ṁ·Δt` consistent with
  `propellant_mass_flow_kernel`, and `"ignited"` is set

### Requirement: The IMU senses the burn through the existing specific-force seam

`RetroThrust` SHALL require no new navigation code to be felt by the onboard IMU: because the stage
composes thrust into the same aero-force channel the ESKF's specific-force term already reads
("gravity + thrust"), the navigation prediction sees the burn automatically once the summed force is
on the channel. The stage MUST compose before the force consumers (the loads stage, the truth
propagator, the navigation kick), the M2 order contract.

#### Scenario: Navigation reads the summed force including thrust

- **WHEN** a coupled step composes lift, `RetroThrust`, and then the navigation stage in the pinned
  order
- **THEN** the navigation prediction reads the identical summed force vector, thrust included, with
  no navigation-side code change
