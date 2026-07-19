## ADDED Requirements

### Requirement: The flown corridor reproduces its committed witnesses

The corridor example SHALL reproduce its committed gate witnesses after this change lands: with
the atmosphere extended below 30 km and the propulsion contracts present in the library,
`cargo run --release -p avionics_examples --example plasma_blackout_corridor` MUST complete with
exit code 0, every gate passing, and the gate witnesses (blackout window onset/exit/dwell, the
RAM-C II anchor band values, drift and reacquisition figures) equal to the committed
`output.txt`. This guard is standing: every subsequent retropulsion change that touches the
shared marcher path, the coupling stack, or the shared example constants — the de-risking
change (`plasma-retropulsion-de-risk`, whose forcing seam rides the marcher the corridor flies)
as much as M3–M5 — MUST re-run it before archive, because the extension's claim is that it only
appends.

#### Scenario: The corridor is bit-identical after the contracts land

- **WHEN** the corridor example is re-run after this change is implemented
- **THEN** its witnesses equal the committed `output.txt` values exactly, with no gate band
  shifted and no new provenance entries

### Requirement: The inert stub is proven a no-op in a coupled march

A harness test in `deep_causality_cfd` SHALL march a corridor-class coupled world twice — once
with the plain coupling stack, once with the propulsion stub composed at zero commanded
throttle — and MUST assert the two runs' reports, final coupled fields (scalars, force channel,
command channels, regime), and provenance logs are bit-identical, extending the landed
marcher-path bit-identity pattern (`unforced_carrier_matches_the_bare_marcher_bit_for_bit`,
`tests/types/flow/compressible_march_run_tests.rs`, from `plasma-retropulsion-de-risk`) to the
stage layer. This is the tested meaning of "strictly inert at zero throttle": the burn-phase
stack can contain the propulsion stages from the start, and ignition remains a
published-command event rather than a stack swap.

#### Scenario: Stub presence is invisible at zero throttle

- **WHEN** the harness marches N coupled steps with and without the stub at zero throttle
- **THEN** every compared artifact is bit-identical between the two runs

#### Scenario: The same harness detects a non-inert regression

- **WHEN** a stub (or future production stage) writes any field, force, or log entry at zero
  throttle
- **THEN** the harness fails, naming the first diverging artifact
