## MODIFIED Requirements

### Requirement: Isolated-cylinder rung gates its published references

The `dec_cylinder_verification` example SHALL gate the shedding Strouhal number `St = f·D/U` and the
cycle-mean drag coefficient `C_d` against the published laminar benchmarks it already cites (Williamson
1996 for `St`; Dröge & Verstappen 2005 and the Lehmkuhl lineage for `C_d`), and SHALL exit non-zero when a
gate breaks or when the solver returns an error.

The harness SHALL be configured through the config layer: its mesh (box domain with the immersed
disk and its merge floor), solver (viscosity, time step, CG options, warm start, and the staircase
no-slip toggle), boundary-zone tuple, seed, and observables SHALL be assembled through
`CfdConfigBuilder::march` and run through `CfdFlow::march`, with its per-step force sampling on the
pipeline's `run_with` hook. The harness SHALL NOT hand-assemble the lattice, cut-cell registry,
manifold, and solver that the configuration family already materializes.

Its environment-overridable knobs remain harness plumbing: they SHALL feed the configuration's
values rather than being replaced by it.

Because the affordable default grid (8 cells/D) is below reference-grid quality, the `St` and `C_d` bounds
MAY be pinned tripwires rather than reference gates at the default configuration. Whichever class is used
SHALL be declared per the evidence-class requirement, and the reference values SHALL remain printed
alongside the measured ones so the offset stays visible.

#### Scenario: A solver error fails the run

- **WHEN** the march returns `Err`
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

#### Scenario: The lift preserves the reported numbers

- **WHEN** the harness is run at its default knobs before and after being configured through the
  config layer
- **THEN** the reported `St`, `C_d`, and the pressure/friction split match to the printed precision
