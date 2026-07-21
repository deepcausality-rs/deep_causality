## ADDED Requirements

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
