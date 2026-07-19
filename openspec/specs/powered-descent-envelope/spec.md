# powered-descent-envelope Specification

## Purpose
TBD - created by archiving change plasma-retropulsion-cfd-contracts. Update Purpose after archive.
## Requirements
### Requirement: Optional burn axes on the safety envelope

`SafetyEnvelope` SHALL gain optional powered-descent axes carried as `burn: Option<BurnEnvelope>`,
where `BurnEnvelope` holds: a throttle floor and ceiling, a maximum thrust coefficient `max_ct`
(the dynamic throttle cap), an ignition dynamic-pressure window `[q_min, q_max]`, a propellant
floor, and a maximum descent rate. The existing three-limit constructor
`SafetyEnvelope::new(max_heat_flux, max_g_load, max_bank_rad)` MUST keep its signature and yield
`burn: None`, so every existing call site compiles and behaves unchanged; the burn axes attach
through an explicit builder step.

#### Scenario: The three-limit constructor is unchanged

- **WHEN** the corridor constructs its envelope with the existing three-argument `new`
- **THEN** the envelope carries `burn: None` and the corridor code compiles without edits

#### Scenario: Burn axes attach explicitly

- **WHEN** an envelope is built with the burn-axis builder step supplying all six axes
- **THEN** the envelope reports each axis with the supplied value

### Requirement: The gate enforces the burn axes with the same pattern

`CyberneticCorrect` SHALL enforce active burn axes through its existing sense → clamp → `Err`
cybernetic pattern, sensing through configurable scalar-field names (the `heat_flux_field`
precedent): the commanded throttle read from the throttle channel is clamped into
`[floor, min(ceiling, ct_ceiling)]`, where `ct_ceiling` is the throttle at which the thrust
coefficient reaches `max_ct` given the sensed dynamic pressure and the configured thrust
reference and reference area — a **dynamic** ceiling, since `C_T = T/(q∞·S_ref)` moves with
`q∞`. A clamp that changes the command MUST be logged to the `EffectLog`. An unrecoverable
breach — propellant at or below the floor while thrust is commanded, or descent rate above the
bound — MUST log the breach and return the same `Err(PhysicalInvariantBroken)` short-circuit
the bank gate returns today, never an out-of-envelope command.

#### Scenario: The dynamic thrust-coefficient cap binds before the static ceiling

- **WHEN** the sensed dynamic pressure is low enough that the `max_ct` ceiling falls below the
  static throttle ceiling, and a larger throttle is commanded
- **THEN** the gate writes the C_T-derived ceiling into the throttle channel and logs the
  bounded correction

#### Scenario: Propellant floor breach refuses, not clamps

- **WHEN** the sensed propellant is at or below the floor while a positive throttle is commanded
- **THEN** the gate logs the breach and returns `Err`, short-circuiting the coupling step, and
  no throttle command is emitted

#### Scenario: Descent-rate bound is enforced

- **WHEN** the sensed descent rate exceeds the bound with no admissible corrective command
- **THEN** the gate logs the breach and returns `Err` rather than emitting an unsafe action

### Requirement: Inactive axes change nothing

`CyberneticCorrect` SHALL behave bit-identically to the pre-change gate whenever `burn` is
`None`, or the burn axes are present and **no throttle is commanded on either seam**: the same bank
clamp, the same log entries, the same breach semantics, and no new log traffic. The original wording
— "no throttle channel was written" — was ambiguous against the crate's own definition of a
commanded throttle, which is the throttle channel **or**, absent one, the world-published
`"commanded_throttle"` scalar. Under that definition a scalar-driven world *does* carry a throttle
command, so invisibility MUST key on both seams being quiet rather than on the channel alone. A
world that attaches burn axes and commands a positive throttle only through the published scalar is
therefore **outside** this requirement's protection, and is governed by the blind-gate requirement
below.

#### Scenario: The corridor gate is bit-identical

- **WHEN** a corridor-class coupled march runs with the extended gate, `burn: None`, and no
  throttle writes
- **THEN** its report, final field, and provenance log are bit-identical to the pre-extension
  gate's output on the same world

#### Scenario: Burn axes with neither seam driven stay silent

- **WHEN** an envelope carries burn axes, the throttle channel is absent, and no positive
  `"commanded_throttle"` scalar rides the field
- **THEN** the gate emits no new log traffic and no new error

### Requirement: The ignition dynamic-pressure window is an enforced axis

`CyberneticCorrect` SHALL enforce the envelope's `[q_min, q_max]` ignition window against the sensed
dynamic pressure, which today is stored on `BurnEnvelope` with no read site anywhere in the crate.
A commanded throttle rising from zero while the sensed dynamic pressure lies outside the window MUST
be refused — logged and returned as the same `Err(PhysicalInvariantBroken)` short-circuit the other
unrecoverable breaches use — rather than clamped, because the window bounds *when a burn may start*
rather than *how hard it may push*. Once a burn is under way the window MUST NOT be re-applied as a
running constraint; the propellant floor, the descent-rate bound, and the thrust-coefficient cap are
the axes that bound a burn in progress.

#### Scenario: Ignition outside the window is refused

- **WHEN** a throttle rises from zero while the sensed dynamic pressure is below `q_min` or above
  `q_max`
- **THEN** the gate logs the breach and returns `Err`, and no thrust is commanded

#### Scenario: The window does not bound a burn already under way

- **WHEN** a committed burn's dynamic pressure leaves the window as the vehicle decelerates
- **THEN** the gate does not refuse on the window axis, and the running axes continue to enforce

### Requirement: A crossed clamp window admits no throttle and is a breach

`CyberneticCorrect` SHALL treat a clamp interval whose upper bound has fallen below its lower bound
— the dynamic thrust-coefficient ceiling below the throttle floor — as an unrecoverable breach,
logging it and returning `Err`. The interval is `[throttle_floor, min(throttle_ceiling, ct_ceiling)]`
and the clamp helper carries no ordering precondition, so when the two bounds cross the gate today
emits one of **two** out-of-envelope values, chosen by the command rather than by any rule, with
only the ordinary bounding log: because the helper tests the lower bound first, a command between
the two bounds is clamped **up to the floor**, exceeding the very `max_ct` cap the ceiling encodes,
while a command at or above the floor is clamped **down to `ct_ceiling`**, below the floor that
encodes the central-nozzle stability constraint. Neither branch is admissible, so the gate MUST NOT
be repaired by reordering the helper or by holding at either bound — a requirement phrased as
"never below the floor" would be satisfied by an implementation that silently violates the cap
instead. There is no admissible throttle at a crossed window, and reporting that is the correct
action.

#### Scenario: A crossed window refuses rather than choosing a bound

- **WHEN** the sensed dynamic pressure is low enough that the `max_ct` ceiling falls below the
  throttle floor, and any throttle is commanded
- **THEN** the gate logs an infeasible-window breach and returns `Err`, rather than emitting either
  bound

#### Scenario: Neither out-of-envelope branch survives

- **WHEN** a command lies between the crossed bounds, and separately when it lies at or above the
  floor
- **THEN** both cases refuse, rather than one clamping up past the cap and the other down past the
  floor

### Requirement: Every axis is sensed and decided before any refusal

`CyberneticCorrect` SHALL sense all of its axes, collect and log every breach it finds, and only
then return. Today the thermal and g-load refusal returns before the burn block is reached, so a
heat or g breach **masks** a simultaneous propellant-floor or descent-rate breach: neither is
logged, and the step that most needs a complete diagnosis reports only one of its causes. This is a
loss of diagnostic information rather than a safety hole — the refusal short-circuits every stage
after the gate, so no throttle consumer runs and no out-of-envelope command is ever realized — and
the requirement exists so that provenance names every reason a step failed, not merely the first.
The returned error MUST name the first breach in a fixed axis order so the error string stays
deterministic.

#### Scenario: Simultaneous breaches are all logged

- **WHEN** a heat-flux breach and a propellant-floor breach occur on the same step
- **THEN** both are logged, and the returned error names the first in the fixed axis order

#### Scenario: The error string is deterministic

- **WHEN** the same combination of breaches occurs on two runs
- **THEN** the returned error is identical

### Requirement: A gate that cannot see the commanded throttle refuses

`CyberneticCorrect` SHALL refuse when burn axes are attached, a positive `"commanded_throttle"`
scalar rides the field, and the throttle channel is absent — the configuration in which thrust,
propellant depletion, the drag decrement, and the plume re-imprint all proceed while the gate
enforces nothing, because the propulsion stages read the channel **or** the scalar while the gate
reads the channel alone. This configuration is reachable today: a world can publish the scalar as a
counterfactual intervention and drive the full propulsion path with it. The gate MUST NOT widen its
own sensing to the scalar, because its clamp writes the bounded value back into the channel, and a
channel written from a scalar source would then take precedence over that world's published
constant on every later step — freezing a counterfactual branch at its first clamped value and
silently destroying the published-constant intervention seam. Refusing names the misconfiguration
where it happens instead of flying unenforced thrust.

#### Scenario: Scalar-driven thrust with burn axes attached is refused

- **WHEN** a world attaches burn axes and commands a positive throttle only through the published
  scalar
- **THEN** the gate logs that it cannot enforce a throttle it cannot see and returns `Err`

#### Scenario: The counterfactual seam is not hijacked

- **WHEN** the gate handles a scalar-sourced command
- **THEN** it does not write the throttle channel, so a branch world's published constant keeps
  deciding that branch's throttle on every step

### Requirement: An unusable burn-sensing configuration is reported at construction

`CyberneticCorrect` SHALL reject or report a configuration in which burn axes are attached while the
thrust reference is not positive. The dynamic thrust-coefficient cap is computed from the thrust
reference and reference area, and the default constructor leaves both at zero, so attaching burn
axes without the burn-sensing builder step disables the `max_ct` axis **silently** — no log, no
error, no diagnostic — while every other burn axis continues to enforce. Exactly one axis degrades,
and the specification MUST say so precisely rather than describing burn sensing as simply off.

#### Scenario: Burn axes without a positive thrust reference are reported

- **WHEN** an envelope carries burn axes and the gate's thrust reference is zero
- **THEN** the misconfiguration is surfaced rather than the `max_ct` axis silently never binding

### Requirement: The reduction applied to each sensed axis is specified

`CyberneticCorrect` SHALL document, per sensed **scalar**, the reduction it applies across cells and
the safety direction that reduction implies. The five sensed scalars — heat flux, g load, dynamic
pressure, propellant, and descent rate — are each reduced by taking the maximum over cells, folded
from zero, which is **conservative** for heat flux, g load, and descent rate, where the worst cell
decides, and **permissive** for dynamic pressure, where the largest sensed value yields the loosest
thrust-coefficient ceiling, and for propellant, where the fullest cell is compared against the
floor. An absent producer therefore reads as zero, which is inside the envelope for the descent-rate
bound and at or below the floor for propellant. The bank and throttle **channels** are not scalars
and are read as options, not reductions.

#### Scenario: The permissive axes are named as such

- **WHEN** the envelope's sensing behavior is read from the specification
- **THEN** the dynamic-pressure and propellant reductions are identified as permissive, and the
  thermal, g-load, and descent-rate reductions as conservative

