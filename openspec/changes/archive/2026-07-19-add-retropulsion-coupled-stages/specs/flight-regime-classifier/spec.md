## ADDED Requirements

### Requirement: The classifier gains opt-in Mach, thrust, and touchdown axes

`RegimeClass` and `RegimeClassify` SHALL gain three flight-phase axes read from the carrier's
published scalars: a **Mach regime** (supersonic / transonic / subsonic, banded from `"flight_mach"`
with configurable thresholds), a **thrust state** (coast / burn from the `"ignited"` flag), and a
**touchdown** flag (from `"flight_altitude"` against a configured altitude floor). These axes MUST be
additive on `RegimeClass` and MUST be **opt-in** on `RegimeClassify`: absent the opt-in they stay
neutral *even when the scalars are published*, because the compressible carrier publishes
`"flight_mach"` on every step — so neutrality cannot be conditioned on the scalar being missing. A
corridor world (which never opts in) therefore classifies, keys, and logs exactly as before.

#### Scenario: Each axis reads its published scalar once opted in

- **WHEN** a classifier built with the flight-axis opt-in runs on a field carrying `"flight_mach"`,
  `"ignited"`, and `"flight_altitude"`
- **THEN** the recorded `RegimeClass` reports the Mach band, the thrust state, and the touchdown flag
  consistent with those scalars

#### Scenario: Without the opt-in the corridor classification is unchanged

- **WHEN** a classifier built **without** the flight-axis opt-in runs on a field that *does* publish
  `"flight_mach"`, `"ignited"`, and `"flight_altitude"` (as the compressible carrier always does)
- **THEN** all three axes are neutral, and the selected model, comms-denial, regime key, and logged
  message text are identical to the pre-change classifier's

### Requirement: A band, thrust, or touchdown transition is a logged regime change

The regime `key()` SHALL fold in the Mach regime, the thrust state, and the touchdown flag alongside
the existing governing-model and comms-denial pair, so a Mach-band crossing, a burn↔coast transition,
or a touchdown is detected as a regime change and logged once to the provenance log, while the
continuous `flight_mach` / altitude values stay out of the key (an unchanged regime is not re-logged).

#### Scenario: A Mach crossing under thrust logs a transition

- **WHEN** consecutive classifier steps cross a Mach band (e.g. supersonic → transonic) while the
  burn is lit
- **THEN** exactly one provenance entry records the transition, and a following step in the same band
  logs nothing new

#### Scenario: A burn↔coast transition logs

- **WHEN** the `"ignited"` flag flips between consecutive steps at an unchanged Mach band
- **THEN** the thrust-state change is logged as a regime transition

### Requirement: A burn leg emits the ordered regime cascade

A burn-leg harness SHALL show the ordered flight-regime cascade in the provenance log — aero →
thrust-dominated, the Mach crossings under thrust, and burn↔coast — each transition flow- or
trajectory-resolved, alongside the existing rarefaction / comms transitions, and with step integrity
held (no step captured an error, the coupled field stayed finite) across the leg.

#### Scenario: The cascade and integrity hold across a commanded burn

- **WHEN** a corridor-class world is marched through a commanded burn leg
- **THEN** the ordered regime transitions appear in provenance and no step recorded an error
