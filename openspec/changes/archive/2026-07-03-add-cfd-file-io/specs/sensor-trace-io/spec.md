## ADDED Requirements

### Requirement: Sensor traces load as gap-honest typed samples

`deep_causality_file` SHALL provide a loader for time-stamped, per-channel sensor traces as a
lazy `IoAction`: each channel yields samples of `(timestamp, value)` in the caller's scalar
`R`, and a missing sample at a timestamp SHALL be represented as absent rather than as a
sentinel value, so the consumer can lift presence into `MaybeUncertain` and noise into
`Uncertain` per the recorded presence-gate design. The loader SHALL NOT depend on
`deep_causality_uncertain`; the uncertain lift belongs to the consumer.

#### Scenario: An intermittent channel keeps its gaps

- **WHEN** a trace file with a dropout-prone channel is loaded
- **THEN** the returned channel contains absent entries exactly where the file has no sample,
  and no sentinel value (zero, NaN, or otherwise) stands in for a missing measurement

#### Scenario: The load is lazy and typed

- **WHEN** a trace load is described and later `.run()`
- **THEN** no filesystem access happens before `.run()`, and values arrive as exact `f64`
  lifted into `R`
