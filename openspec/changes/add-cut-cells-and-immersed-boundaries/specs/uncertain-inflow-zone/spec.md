## ADDED Requirements

### Requirement: Sensor-fed uncertain inflow zone
The solver SHALL support an `UncertainInflowZone` that tags a set of inflow boundary cells
fed by a `MaybeUncertain<R>` stream, where `R` is the solver's precision (from the
`generalize-uncertain-over-realfield` prerequisite). Each step the zone SHALL gate the
stream with `lift_to_uncertain` and, on success, take a presence-confirmed `R` inflow value
for assembly. The selective uncertain typing SHALL be confined to the tagged inflow patch;
the solver rate and step SHALL remain `R: RealField` with no `MaybeUncertain` in their
signatures.

#### Scenario: Present sensor value drives the inflow
- **WHEN** the sensor stream reports a present value above the presence threshold for an inflow cell
- **THEN** the lifted `R` value is applied as the Dirichlet inflow for that cell and the march proceeds with an `R`-typed field

#### Scenario: Uncertain typing does not leak into the march
- **WHEN** the solver rate and step are evaluated
- **THEN** no `MaybeUncertain` value enters them; the uncertain types exist only at the tagged inflow patch and the extra memory cost is confined to that patch

### Requirement: Dropout triggers a logged corrective intervention
The zone SHALL detect a sensor dropout (the presence gate `lift_to_uncertain` returning a
presence error) and fire a BC-fallback corrective intervention that substitutes the
last-good or a configured-default inflow via `.intervene`, recording the dropout and the
fallback in the `EffectLog`. A run with no dropouts SHALL reproduce the deterministic-inflow
control run to rounding.

#### Scenario: Dropout falls back and is logged
- **WHEN** the sensor stream drops below the presence threshold for an inflow cell
- **THEN** the BC-fallback value is applied via `.intervene`, the dropout and fallback are recorded in the `EffectLog`, and the march continues without error

#### Scenario: No-dropout run matches the deterministic control
- **WHEN** a stream with no dropouts drives the inflow zone and is compared against a deterministic-inflow run of the same case
- **THEN** the marched fields agree to rounding
