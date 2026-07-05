## 1. Study primitives (deep_causality_cfd)

- [x] 1.1 `sweep` in the flow module: order-preserving, `Result`-collecting, first error
  wins; `scoped_map`-backed under the `parallel` feature, plain map without it; output type
  fully generic. Docstring states the side-effect rule (closures print or write files after
  the sweep, not inside it).
- [x] 1.2 `Gates` in the flow module: `new(title)`, `gate(label, pass, detail)`,
  `finish() -> bool` printing the `[PASS]`/`[FAIL]` lines and the verdict line, nothing else.
- [x] 1.3 `run_owned` on `MarchPipeline`: internal `materialize` plus `.on(...).run()`;
  docstring says when the B1 borrow form is the right choice instead. (The uncertain pipeline
  carries no mesh in its config, so it has no one-shot form; recorded in design D3.)
- [x] 1.4 Mirrored tests: sweep order and bit-identity across the feature flag, first-error
  semantics, empty input; `Gates` verdict and output lines; `run_owned` equals the explicit
  form on the same case. Bazel coverage via the existing flow-suite glob.

## 2. Table construction (deep_causality_file)

- [x] 2.1 `NumericTable::from_columns((name, unit) pairs, rows)` with the existing
  rectangularity validation; mirrored test proving equality with the explicit constructor and
  rejection of ragged rows.

## 3. Duct march (deep_causality_physics + deep_causality_cfd)

- [x] 3.1 Isentropic kernels in `deep_causality_physics/src/kernels/`: area-Mach relation and
  the isentropic pressure, temperature, and density ratios; full citations, source PDF in
  `papers/`, wrappers and registration per the kernel contract; mirrored tests (subsonic and
  supersonic branches, throat limit, frozen checks).
  -> Landed in `kernels/fluids/compressible.rs` beside the existing compressible kernels
  (isentropic pressure/temperature/density ratios, area-Mach relation), Anderson-cited,
  NaN-rejecting guards, wrappers registered; 24 kernel tests green.
- [x] 3.2 `DuctConfig` in `flow_config`: area profile (table or analytic variant), inlet
  stagnation state, back pressure, resolution, stop condition; validating constructor
  (positive areas, gamma > 1, finite states) in the schedule-validation style.
  -> `flow_config/duct_config.rs`: Table and ConvergingDiverging (parabolic, smooth-throat)
  area profiles, DuctInlet, DuctStop; validating constructor refuses ragged/non-positive
  geometry, back_pressure >= p0, gamma <= 1, cells < 8, zero budgets; 12 tests green.
- [x] 3.3 The `CfdFlow::duct_march` runner over the 1-D compressible Euler solver: quasi-steady
  march with a residual-gated stop, `Report` with `"x"`, `"mach_profile"`,
  `"pressure_profile"`, `"shock_position"`, `"thrust_coefficient"`; expiry above the residual
  gate is an error naming the budget.
  -> `flow/duct_march_run.rs`: quasi-1-D driver, residual-gated quasi-steady march, Report
  with x/mach/pressure profiles, thrust coefficient, and shock_position omitted (not a
  sentinel) on shock-free runs; budget expiry errors name residual and budget.
- [x] 3.4 Mirrored tests plus a verification run: shock-free profile against the area-Mach
  relation within the derived band; a shocked case placing the shock at the analytic position
  within the grid-spacing band; convergence-failure error path.
  -> `flow/duct_march_tests.rs`: shock-free profile within 5 percent of the area-Mach
  relation at 128 cells (measured; interior stations, throat +-0.1 excluded); shocked case
  places the shock within 12 cell widths of the closed-form position (isentropic + RH +
  subsonic recovery bisection); expired budget errors loudly. Two reference-side bugs found
  and fixed during measurement: the analytic comparison had a linear profile against the
  config's parabolic one, and the shock-position bisection direction was inverted.

## 4. The three examples (examples/avionics_examples/cfd/)

- [x] 4.1 `nozzle_operating_map`: back-pressure schedule read through the group-1 reader,
  `sweep` over `duct_march`, per-row analytic gates (critical ratio, shock position,
  area-Mach), operating-map table through the group-1 writer, `Gates` verdict, `FloatType`
  throughout, wrong-usage test (non-numeric schedule cell names file, row, and column),
  README in the established convention, Cargo.toml registration, recorded `output.txt`.
  -> Six ratios 0.90 to 0.10 on the 2:1:2 parabolic nozzle; measured: shock marches 0.656 to
  0.930 m as p/p0 falls 0.90 to 0.60 (each within 12 cells of the closed form), supersonic
  rows exit at M 2.12 vs design 2.197, first critical 0.937 / exit shock 0.513 printed as
  references. Wrong-usage run names file, row 4, and column, exits 1, writes nothing.
  5 gates green, sub-second wall-clock. Shares the blackout utils (ft), main on top,
  prints in utils_print.rs.
- [x] 4.2 `viv_resonance_margin`: airspeed schedule, `sweep` + `run_owned` over the validated
  cylinder configuration, `dominant_frequency` extraction, margin table, Strouhal-band and
  margin gates, stated Reynolds range, README, registration, recorded `output.txt`.
  -> Four airspeeds, Re 100 to 160, D = 2 mm, f_struct = 150 Hz (stated demonstration value).
  Measured: St 0.1818 to 0.1909 (on the Williamson laminar reference), f_shed 68 to 115 Hz,
  worst margin 0.236 vs the 0.15 placard. The agent-drafted ST_BAND (0.20, 0.30) claimed
  measurements that did not match the run; re-pinned from the actual measurement to
  (0.16, 0.21) with the corrected justification. 3 gates green, exit 0, 142 s wall-clock.
- [x] 4.3 `flight_envelope_placard`: Mach-altitude matrix through the reader, pointwise
  computation with existing kernels plus dynamic pressure, placard gates that name any
  out-of-envelope point, placard table, README, registration, recorded `output.txt`; includes
  the out-of-envelope negative test. -> 16-point corridor (M 0.5/5 km to M 5/40 km), placards
  60 kPa / 1700 K, recorded peaks q = 23.7 kPa at M 1.20/11 km and T0 = 1502.1 K at
  M 5.00/40 km, all gates green in <0.01 s; `mach_alt_matrix_exceeds.csv` adds M 1.50/5 km
  (q = 85.1 kPa), named in the FAIL line, exit 1.
- [x] 4.4 Run all three end to end; every gate green, exit 0; outputs recorded.
  -> nozzle 5/5, placard 3/3, viv 3/3 gates green, all exit 0, outputs recorded after two
  convention passes: shared utils::ft with main at the top and per-example utils_print.rs,
  then the configuration/execution split (model_config.rs holds case descriptions, model.rs
  holds domain logic with at most three structs, model_types.rs when more). All three
  READMEs' file tables reflect the final layout.

## 5. Finalize

- [x] 5.1 `make format && make fix`; full file, physics, and cfd suites green plus the
  examples crate building and running; clippy clean (fix, never allow); Bazel registration
  checked for every new test module; no `unsafe`/`dyn`/lib macros; float literals only in
  cited constants, tests, and examples.
  -> 100 file + 1732 physics + 628 cfd tests green; clippy 0 across all four crates (all
  targets); the three examples run green (5/5, 3/3, 3/3 gates); new test modules ride the
  existing Bazel globs (`kernels/fluids`, flow and flow_config suites).
- [x] 5.2 `openspec validate add-cfd-study-dsl-and-examples --strict`; update the
  common-examples note (Group 2 status) and the dsl-review note (S1 to S5 shipped); prepare
  the commit message for review (never commit).
  -> Validates strict; both notes updated; commit message prepared and handed to the user.
