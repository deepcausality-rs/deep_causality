## 1. Study primitives (deep_causality_cfd)

- [ ] 1.1 `sweep` in the flow module: order-preserving, `Result`-collecting, first error
  wins; `scoped_map`-backed under the `parallel` feature, plain map without it; output type
  fully generic. Docstring states the side-effect rule (closures print or write files after
  the sweep, not inside it).
- [ ] 1.2 `Gates` in the flow module: `new(title)`, `gate(label, pass, detail)`,
  `finish() -> bool` printing the `[PASS]`/`[FAIL]` lines and the verdict line, nothing else.
- [ ] 1.3 `run_owned` on `MarchPipeline` and the uncertain pipeline: internal `materialize`
  plus `.on(...).run()`; docstring says when the B1 borrow form is the right choice instead.
- [ ] 1.4 Mirrored tests: sweep order and bit-identity across the feature flag, first-error
  semantics, empty input; `Gates` verdict and output lines; `run_owned` equals the explicit
  form on the same case. Bazel coverage via the existing flow-suite glob.

## 2. Table construction (deep_causality_file)

- [ ] 2.1 `NumericTable::from_columns((name, unit) pairs, rows)` with the existing
  rectangularity validation; mirrored test proving equality with the explicit constructor and
  rejection of ragged rows.

## 3. Duct march (deep_causality_physics + deep_causality_cfd)

- [ ] 3.1 Isentropic kernels in `deep_causality_physics/src/kernels/`: area-Mach relation and
  the isentropic pressure, temperature, and density ratios; full citations, source PDF in
  `papers/`, wrappers and registration per the kernel contract; mirrored tests (subsonic and
  supersonic branches, throat limit, frozen checks).
- [ ] 3.2 `DuctConfig` in `flow_config`: area profile (table or analytic variant), inlet
  stagnation state, back pressure, resolution, stop condition; validating constructor
  (positive areas, gamma > 1, finite states) in the schedule-validation style.
- [ ] 3.3 The `CfdFlow::duct_march` runner over the 1-D compressible Euler solver: quasi-steady
  march with a residual-gated stop, `Report` with `"x"`, `"mach_profile"`,
  `"pressure_profile"`, `"shock_position"`, `"thrust_coefficient"`; expiry above the residual
  gate is an error naming the budget.
- [ ] 3.4 Mirrored tests plus a verification run: shock-free profile against the area-Mach
  relation within the derived band; a shocked case placing the shock at the analytic position
  within the grid-spacing band; convergence-failure error path.

## 4. The three examples (examples/avionics_examples/cfd/)

- [ ] 4.1 `nozzle_operating_map`: back-pressure schedule read through the group-1 reader,
  `sweep` over `duct_march`, per-row analytic gates (critical ratio, shock position,
  area-Mach), operating-map table through the group-1 writer, `Gates` verdict, `FloatType`
  throughout, wrong-usage test (non-numeric schedule cell names file, row, and column),
  README in the established convention, Cargo.toml registration, recorded `output.txt`.
- [ ] 4.2 `viv_resonance_margin`: airspeed schedule, `sweep` + `run_owned` over the validated
  cylinder configuration, `dominant_frequency` extraction, margin table, Strouhal-band and
  margin gates, stated Reynolds range, README, registration, recorded `output.txt`.
- [ ] 4.3 `flight_envelope_placard`: Mach-altitude matrix through the reader, pointwise
  computation with existing kernels plus dynamic pressure, placard gates that name any
  out-of-envelope point, placard table, README, registration, recorded `output.txt`; includes
  the out-of-envelope negative test.
- [ ] 4.4 Run all three end to end; every gate green, exit 0; outputs recorded.

## 5. Finalize

- [ ] 5.1 `make format && make fix`; full file, physics, and cfd suites green plus the
  examples crate building and running; clippy clean (fix, never allow); Bazel registration
  checked for every new test module; no `unsafe`/`dyn`/lib macros; float literals only in
  cited constants, tests, and examples.
- [ ] 5.2 `openspec validate add-cfd-study-dsl-and-examples --strict`; update the
  common-examples note (Group 2 status) and the dsl-review note (S1 to S5 shipped); prepare
  the commit message for review (never commit).
