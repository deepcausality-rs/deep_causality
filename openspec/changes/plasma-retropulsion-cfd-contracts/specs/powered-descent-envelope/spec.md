## ADDED Requirements

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
`None`, or the burn axes are present but no throttle channel was written: the same bank clamp,
the same log entries, the same breach semantics, and no new log traffic. The powered-descent
extension MUST be invisible until both the axes and a throttle command exist.

#### Scenario: The corridor gate is bit-identical

- **WHEN** a corridor-class coupled march runs with the extended gate, `burn: None`, and no
  throttle writes
- **THEN** its report, final field, and provenance log are bit-identical to the pre-extension
  gate's output on the same world
