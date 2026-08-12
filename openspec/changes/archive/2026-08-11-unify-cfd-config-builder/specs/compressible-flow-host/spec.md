## MODIFIED Requirements

### Requirement: The compressible two-temperature marcher runs in the CfdFlow coupled loop

`deep_causality_cfd` SHALL provide a compressible march host in the `CfdFlow` DSL: an owned config
plus builder started by `CfdConfigBuilder::compressible_march::<R>(name)` (the same config→run
split), driving the existing compressible two-temperature marcher
over its `EulerStateTt` state, with `run_coupled` hosting the between-step `PhysicsStage` stack
and the blackout sampler. Precision SHALL remain a parameter, and the marcher's numerics SHALL be
consumed as-is (no solver changes).

The builder's constructor and its `Default` implementation SHALL be crate-private, and the case name
SHALL be taken at the entry rather than through a builder method. The fixed dimensional anchors of
the physical projections SHALL be set through a builder method taking the temperature, number-density
and speed references, rather than through a public-field struct.

#### Scenario: A coupled compressible march produces the blackout report
- **WHEN** a compressible config is run with `run_coupled`, a coupling stack, and a blackout
  trigger
- **THEN** the run marches the two-temperature state, applies the stack each step, samples the
  opted-in observables (`n_e`, plasma frequency, heat flux, dwell), and returns the owned report
  with the provenance log attached

#### Scenario: The world is named at its entry
- **WHEN** a descent world is configured through `CfdConfigBuilder::compressible_march`
- **THEN** the name is supplied at the entry, and the built config carries it with no builder-side
  naming method and no default name

#### Scenario: Reference scales are set through the builder
- **WHEN** the dimensional anchors of the physical projections are configured
- **THEN** they are supplied through a builder method, and the anchors' fields are not publicly
  writable
