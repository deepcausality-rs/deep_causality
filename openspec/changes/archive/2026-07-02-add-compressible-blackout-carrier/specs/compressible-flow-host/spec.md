## ADDED Requirements

### Requirement: A marcher seam shares the coupled-loop machinery

`deep_causality_cfd` SHALL extract the coupled-loop machinery (`run_coupled`, `run_until`,
`MarchPause`, O(1) `fork`, `continue_march`, and the verbatim `alternate_context` /
`alternate_state` / `alternate_value` vocabulary with its audit markers and never-alternated error
channel) behind a crate-internal marcher seam, so exactly one implementation of the resumable loop
serves both the incompressible QTT carrier and the compressible carrier. The refactored QTT host
SHALL preserve its public API and produce bit-identical results under the existing equivalence,
pause, and alternation tests. Composition SHALL remain static (no `dyn`).

#### Scenario: The QTT host is unchanged through the seam
- **WHEN** the existing QTT DSL-equivalence, pause/fork, and alternation tests run after the seam
  refactor
- **THEN** every test passes unchanged, and the QTT public API is source-compatible

#### Scenario: One pause contract for both carriers
- **WHEN** a compressible run pauses at a predicate and forks
- **THEN** the fork shares state by reference (no tensor copy until a write), captures step errors
  into the pause's error channel, refuses alternation on an errored fork with only the audit
  entry, and resumes in an alternated world via `continue_march` — the same contract the QTT host
  already honors

### Requirement: The compressible two-temperature marcher runs in the CfdFlow coupled loop

`deep_causality_cfd` SHALL provide a compressible march host in the `CfdFlow` DSL: an owned config
plus builder (the same config→run split) driving the existing compressible two-temperature marcher
over its `EulerStateTt` state, with `run_coupled` hosting the between-step `PhysicsStage` stack
and the blackout sampler. Precision SHALL remain a parameter, and the marcher's numerics SHALL be
consumed as-is (no solver changes).

#### Scenario: A coupled compressible march produces the blackout report
- **WHEN** a compressible config is run with `run_coupled`, a coupling stack, and a blackout
  trigger
- **THEN** the run marches the two-temperature state, applies the stack each step, samples the
  opted-in observables (`n_e`, plasma frequency, heat flux, dwell), and returns the owned report
  with the provenance log attached

### Requirement: Evolved-state projections replace reconstruction

The compressible host SHALL publish per-cell projections of the *evolved* state into the
`CoupledField` each step — at minimum `"speed"`, `"T_tr"`, `"T_ve"`, and `"n_tot"` — so the
corridor consumes marched post-shock quantities instead of the recovery-temperature
reconstruction. A thin stage SHALL compute the Park rate-controlling `"T_a" = √(T_tr·T_ve)` from
the evolved fields, and `IonizationStage` SHALL accept a per-cell density field in place of the
scalar config density. The reconstruction stages (`RecoveryTemperatureStage`,
`VibrationalLagStage`) SHALL remain available for the QTT surrogate path.

#### Scenario: Ionization reads the evolved controller
- **WHEN** the coupled step publishes evolved `T_tr`/`T_ve`/`n_tot` and the stack computes `T_a`
- **THEN** the ionization stage relaxes toward the Saha target at the evolved controller with the
  per-cell density, and the resulting `n_e` requires no imposed Rankine-Hugoniot jump and no
  station-level density constant

### Requirement: A scheduled freestream closes the descent loop

The compressible host SHALL accept a freestream *schedule*: a cited standard-atmosphere table
(altitude → density, temperature, sound speed) evaluated at the truth vehicle's current state, so
the navigation trajectory drives the flow inflow and the flow drives the navigation — a continuous
descent instead of station switches. The host SHALL rebuild the solver when the scheduled
freestream drifts beyond a configured tolerance, and SHALL log each rebuild to the provenance
channel.

#### Scenario: The descent sweeps the flight conditions continuously
- **WHEN** one coupled run descends through the altitude schedule
- **THEN** the freestream follows the truth trajectory through the atmosphere table, solver
  rebuilds occur only past the drift tolerance (each logged), and no per-station constants are
  supplied beyond freestream and geometry
