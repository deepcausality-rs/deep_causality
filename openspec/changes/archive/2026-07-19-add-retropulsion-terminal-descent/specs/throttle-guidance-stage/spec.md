## ADDED Requirements

### Requirement: ThrottleGuidance commands the throttle from the Tier-A closed form

`ThrottleGuidance` SHALL be a `PhysicsStage` in `deep_causality_cfd` that forms the commanded
deceleration `a_cmd = v²/2h + g` through `suicide_burn_deceleration_kernel`, reading the flight speed
and altitude the compressible carrier publishes and a configured gravitational acceleration, and
maps it to a throttle through the crate's existing linear thrust convention `T = θ · T_full` —
`θ = m·a_cmd / T_full`, using the carried `"mass"` scalar — saturated to `[0, 1]` before any envelope
sees it. It MUST write the result with `CoupledField::set_throttle_action`. The stage MUST NOT
introduce a thrust-from-throttle kernel: the physics crate deliberately leaves throttle mapping to
the CFD-side stages, and `RetroThrust` already realizes the same relation, so the two stages share
one convention. Apollo polynomial guidance and convex powered-descent guidance are the named
upgrade path, not this stage's scope.

#### Scenario: The commanded throttle follows the guidance law

- **WHEN** the stage runs with a positive flight speed, a positive altitude, and a carried mass
- **THEN** the throttle channel carries `m·(v²/2h + g)/T_full`, saturated into `[0, 1]`

#### Scenario: The guidance law and the thrust stage share one convention

- **WHEN** `ThrottleGuidance` commands a throttle and `RetroThrust` converts it back to a force
- **THEN** the realized thrust is `θ · T_full` for the same `T_full`, with no second mapping

#### Scenario: Ground contact is an error, not a division

- **WHEN** the sensed altitude is at or below zero
- **THEN** the kernel's singularity error propagates out of the stage rather than producing a
  throttle from a non-positive altitude

### Requirement: Guidance commands zero from the first step so the envelope is live

`ThrottleGuidance` SHALL write the throttle channel on **every** step, commanding zero before the
ignition corridor commits, and MUST NOT defer its first write to the commit. The powered-descent
gate reaches its burn axes only when the throttle channel is present, so a stage that wrote the
channel only at ignition would leave the propellant floor, the descent-rate bound, and the throttle
clamp unenforced on every pre-ignition step. Commanding zero is safe because the thrust and plume
stages are strictly inert at a throttle of zero or below, so ignition remains an event on the
commanded **value** rather than on the channel's existence.

#### Scenario: Pre-ignition steps are gated

- **WHEN** a burn world runs before its ignition corridor commits
- **THEN** the throttle channel carries zero, the gate's burn axes are evaluated on every step, and
  the thrust and plume stages make no force write, scalar mutation, or log entry

#### Scenario: Ignition is a change of value, not of wiring

- **WHEN** the corridor commits
- **THEN** the same stage begins commanding a positive throttle through the same channel, with no
  change to the coupling stack

### Requirement: Guidance composes after navigation and before the gate, with a one-step lag

`ThrottleGuidance` SHALL compose after the navigation stage — so it reads the current step's
navigation quality — and before the cybernetic gate, so the gate clamps the command it wrote. The
thrust and plume stages compose earlier, per the force-channel ordering contract, so they read the
throttle the gate clamped on the **previous** step. This one-step actuation lag is inherited from
the bank channel, which behaves identically, and MUST be specified rather than corrected: closing it
would require either sensing before the force channel is written or clamping before the command
exists.

#### Scenario: The gate clamps what guidance wrote

- **WHEN** guidance commands a throttle outside the envelope
- **THEN** the gate bounds it and writes the bounded value back to the same channel within the same
  step

#### Scenario: The thrust stages fly the previous step's clamped command

- **WHEN** the gate clamps a command at step `k`
- **THEN** the thrust and plume stages realize that clamped value at step `k+1`, not at step `k`
