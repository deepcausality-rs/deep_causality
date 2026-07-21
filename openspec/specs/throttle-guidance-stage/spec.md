# throttle-guidance-stage Specification

## Purpose
TBD - created by archiving change add-retropulsion-terminal-descent. Update Purpose after archive.
## Requirements
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

### Requirement: A committed burn may be flown as a stopping burn, coasting to the ignition altitude

`ThrottleGuidance` SHALL support flying a committed burn as a **stopping burn**: command zero
throttle until the altitude falls to `ignition_altitude_kernel(speed, T/m, g, margin)`, then burn.
Opting in is per-stage; a guidance without it burns continuously from commit, which stays correct
for a deceleration burn that is not aimed at a landing site.

The closed form `a_cmd = v²/2h + g` degenerates to `a_cmd ≈ g` for large `h` — thrust balancing
weight. A guidance that commits high therefore nulls its descent rate at altitude and **hovers**,
spending propellant to hold station rather than to land. Coast-then-burn is not a workaround for
that but the optimal structure: Meditch (1964) showed the fuel-optimal control for the soft-landing
problem is bang-bang, null thrust then maximum thrust, so any sustained intermediate throttle is
wasteful by construction. It is also what a real lander's minimum throttle forces, since an engine
whose floor thrust exceeds the landed weight cannot hover at all.

The decision MUST be latched once started, on a carried field scalar, and MUST NOT be re-evaluated
as a live predicate. Thrust acceleration `a_T = T/m` rises as propellant burns off, so the ignition
altitude **falls**; a live predicate would find the vehicle above the new ignition altitude on the
step after lighting and shut the engine down.

Thrust-to-weight at or below one admits no coast — no ignition altitude exists — so the stage MUST
burn rather than refusing, leaving the envelope's descent-rate axis to judge the outcome.

#### Scenario: A committed guidance above its ignition altitude coasts

- **WHEN** a stopping-burn guidance is committed and the altitude is above the ignition altitude
- **THEN** it commands zero throttle, and the burn has not started

#### Scenario: The burn lights once and stays lit

- **WHEN** the altitude first reaches the ignition altitude
- **THEN** the stage commands a positive throttle, latches the start on a carried scalar, and logs
  the altitude, speed, and thrust-to-weight it lit at
- **AND WHEN** a later step would compute a lower ignition altitude than the current altitude
- **THEN** the burn stays lit, because the decision was made once

#### Scenario: A vehicle that cannot stop burns anyway

- **WHEN** thrust-to-weight has fallen to one or below
- **THEN** the stage burns rather than refusing, and the descent-rate axis judges the outcome

### Requirement: The stopping law targets a commanded plane at a commanded contact speed

`ThrottleGuidance` SHALL support nulling the descent velocity at a caller-supplied **target
altitude** rather than at zero altitude, and at a caller-supplied **contact speed** rather than at
rest. Both default to zero, which is the geometric surface at rest.

A lander arrives stopped at its **gear contact plane**, and a run that declares touchdown at a
positive altitude floor samples the descent rate there. Aiming at zero while sampling at the floor
reports the speed the vehicle still had one floor-height of braking short of its target: with net
deceleration `a`, the ideal profile still carries `sqrt(2·a·h_floor)` at the floor. That is a
formulation mismatch, not a property of the vehicle.

A lander also arrives **moving**. Falcon 9 touches down near 2 m/s, and that is a design point
rather than a residual: its landing engine at minimum throttle out-thrusts the nearly empty stage,
so it cannot hover and must be flown into the deck. A deep-throttling vehicle that *can* hover still
commands a positive contact speed, to make firm contact and to stop station-keeping over a landing
site it is not standing on.

The contact speed MUST be realized through the same closed form rather than beside it: the burn
removes kinetic energy down to the contact speed rather than to rest, so the kernel is asked to stop
an effective speed `sqrt(v² − v_c²)`, which is zero exactly at the commanded contact condition.

Below the target plane but above the surface the burn has arrived, and the stage MUST command
weight-balancing thrust and let the vehicle settle. Ground contact is a **different situation** and
MUST remain the kernel's singularity: a guidance asked to command a burn from at or below the
surface has been asked for something inadmissible and MUST say so rather than inventing a throttle.

#### Scenario: The law measures height above the target plane

- **WHEN** a guidance with a positive target altitude commands a burn
- **THEN** the deceleration follows the closed form over the height above that plane, not above the
  geometric surface

#### Scenario: The burn sheds down to the contact speed

- **WHEN** a guidance with a positive contact speed commands a burn
- **THEN** the commanded deceleration is the closed form over the effective speed `sqrt(v² − v_c²)`

#### Scenario: Arriving settles, ground contact errors

- **WHEN** the vehicle is at or below the target plane but above the surface
- **THEN** the stage commands weight-balancing thrust and the vehicle settles
- **AND WHEN** the altitude is at or below zero
- **THEN** the stage propagates the kernel's singularity rather than commanding a throttle

