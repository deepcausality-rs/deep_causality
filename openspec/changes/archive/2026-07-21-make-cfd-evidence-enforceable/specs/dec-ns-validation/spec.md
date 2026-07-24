## ADDED Requirements

### Requirement: Isolated-cylinder rung gates its published references

The `dec_cylinder_verification` example SHALL gate the shedding Strouhal number `St = f·D/U` and the
cycle-mean drag coefficient `C_d` against the published laminar benchmarks it already cites (Williamson
1996 for `St`; Dröge & Verstappen 2005 and the Lehmkuhl lineage for `C_d`), and SHALL exit non-zero when a
gate breaks or when the solver returns an error.

The harness currently contains no assertion and no `process::exit` call: on a solver `Err` it prints,
breaks the march, then reports `St` and `C_d` computed from the *truncated* series and returns zero. That
behaviour contradicts the convention `verification/README.md` advertises for every program in the suite,
and it is the crate's only isolated-cylinder validation.

Because the affordable default grid (8 cells/D) is below reference-grid quality, the `St` and `C_d` bounds
MAY be pinned tripwires rather than reference gates at the default configuration. Whichever class is used
SHALL be declared per the evidence-class requirement, and the reference values SHALL remain printed
alongside the measured ones so the offset stays visible.

#### Scenario: A solver error fails the run

- **WHEN** `solver.step` returns `Err` during the march
- **THEN** the harness reports the failure and exits non-zero, and does not report `St` or `C_d` derived
  from the truncated series

#### Scenario: Strouhal and drag are gated, not merely reported

- **WHEN** the harness completes a full march
- **THEN** `St` and `C_d` are each compared against a declared bound, the comparison result is printed as
  PASS or FAIL with its evidence class, and any FAIL exits non-zero

#### Scenario: Reference values stay visible next to the measurement

- **WHEN** the gate block is printed
- **THEN** the Williamson `St` and the Dröge–Verstappen / Lehmkuhl `C_d` band appear next to the measured
  values, with the grid resolution stated, so an under-resolved pass is not read as reference agreement

## MODIFIED Requirements

### Requirement: Lid-driven cavity rung (coarse CI + example)
The ladder SHALL include the Re-1000 lid-driven cavity, split by cost
per the tests-are-fast / examples-verify division: a single coarse rung
in CI compared against the Ghia et al. (1982) centerline tables with a
pinned RMSE gate at a short spin-up horizon (tests stay fast), and an
example program carrying the thorough verification — the
refinement-trend mode (coarse → finer RMSE strictly decreasing at
time-converged horizons, gated, nonzero exit on violation) and the
full-resolution run emitting centerline CSVs and the detected
vortex-center table (primary and corner eddies) against Ghia's values.

The pinned RMSE bounds carry headroom measured from their own pinning run and are therefore regression
tripwires, not agreement claims against Ghia; they SHALL be labelled `tripwire` per the evidence-class
requirement. The committed baseline artifact SHALL be a complete run carrying the RMSE, the vortex-center
table and the verdict line, and its header configuration SHALL match the configuration whose numbers the
harness and `verification/README.md` report.

#### Scenario: Coarse cavity gates in CI
- **WHEN** the coarse cavity rung completes in CI
- **THEN** centerline RMSE against the Ghia tables is within the pinned gate

#### Scenario: Example verifies the refinement trend
- **WHEN** the cavity example runs in its trend mode
- **THEN** the finer grid's time-converged centerline RMSE is within its pinned gate and strictly below the coarse grid's, with a nonzero exit on violation

#### Scenario: Example emits centerlines and the vortex table
- **WHEN** the cavity example runs at full resolution
- **THEN** it writes centerline CSVs and the detected vortex centers alongside the Ghia reference values

#### Scenario: The committed baseline is a finished run
- **WHEN** the committed cavity `baseline.txt` is read
- **THEN** it carries the centerline RMSE, the vortex-center table and the verdict line, rather than
  terminating mid-march as a progress trace

#### Scenario: Pinned bounds are not presented as Ghia agreement
- **WHEN** the cavity gate block is printed or documented
- **THEN** each pinned RMSE bound is marked `tripwire`, while the Ghia table values it is compared against
  are shown separately as the reference
