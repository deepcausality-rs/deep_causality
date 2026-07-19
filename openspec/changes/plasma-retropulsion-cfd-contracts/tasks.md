## 1. Coupling seam: throttle channel + additive force (deep_causality_cfd)

- [x] 1.1 Add `throttle_action: Option<R>` to `CoupledField` with `throttle_action()` /
      `set_throttle_action()` accessors mirroring `control_action` (`types/flow/coupling.rs`);
      initialize to `None` in `new()`; extend the struct docstring's channel inventory
- [x] 1.2 Add `add_aero_force(delta: [R; 3])` (read-modify-write, `None` treated as zero) beside
      `set_aero_force`, docstring naming the composition order contract (after the ④-writing
      lift stage, before loads/truth/nav)
- [x] 1.3 Tests in the mirrored tree (`tests/types/flow/coupling_tests.rs` or sibling):
      throttle round-trip + `None` default + independence from the bank channel;
      `add_aero_force` over a set force and over `None`; both channels through a
      pause-snapshot round trip

## 2. Envelope extension (deep_causality_cfd)

- [ ] 2.1 Add `BurnEnvelope<R>` (throttle floor/ceiling, `max_ct`, ignition q window
      `[q_min, q_max]`, propellant floor, max descent rate) and `burn: Option<BurnEnvelope<R>>`
      on `SafetyEnvelope`; keep the 3-arg `new()` signature yielding `burn: None`; add the
      explicit burn-axis builder step (`with_burn_envelope`-style)
- [ ] 2.2 Extend `CyberneticCorrect` to enforce active burn axes with the existing
      sense → clamp → `Err` pattern: throttle clamp into `[floor, min(ceiling, ct_ceiling(q∞))]`
      (dynamic C_T cap from the configured thrust reference + reference area), logged when it
      changes the command; propellant-floor and descent-rate breaches log + return the same
      `Err(PhysicalInvariantBroken)`; sensing via configurable field names (the
      `heat_flux_field` pattern)
- [ ] 2.3 Tests with synthetic scalar fields: dynamic cap binds below the static ceiling;
      static ceiling binds when q is high; floor clamp; propellant-floor refusal (`Err`, no
      command emitted); descent-rate refusal; clamp logged; **`burn: None` and
      no-throttle-written paths produce bit-identical behavior and log traffic to the
      pre-change gate**

## 3. Propulsion contract + inert A0 stub (deep_causality_cfd)

- [x] 3.1 New module `types/flow/retropulsion.rs` (registered in `types/flow/mod.rs`, flat
      re-exports from `lib.rs`): `PropulsionStub` reading `"commanded_throttle"` (published
      constant) or the throttle channel; docstring pins the contract names `"mass"`,
      `"propellant"`, `"ignited"` and cites this change as the seam M3 fills
- [x] 3.2 Stub semantics — inert path: at throttle ≤ 0 (or absent), return `Ok` having touched
      nothing (no force write, no scalar mutation, no log entry)
- [x] 3.3 Stub semantics — active path: deplete `"propellant"`/`"mass"` via
      `propellant_mass_flow` (existing physics kernel), set `"ignited"`, add `−T/m·v̂` via
      `add_aero_force`, apply the A0 drag decrement via `srp_thrust_coefficient` +
      `srp_preserved_drag_fraction`; pre-ignition carried mass equals the `CDA_OVER_M`-implied
      constant
- [x] 3.4 Tests: active-path seam checks (depletion consistent with the kernel, force =
      lift + thrust + decrement, `"ignited"` set); published `"commanded_throttle"` lands on
      the field each step (mirroring the `commanded_bank` test); propulsion scalars survive a
      pause snapshot
- [x] 3.5 Register any new test files in their `mod.rs` chain and verify the
      `tests/BUILD.bazel` suite globs pick them up (`bazel test //deep_causality_cfd/...`)

## 4. Inheritance-guard harness (deep_causality_cfd)

- [ ] 4.1 Harness test: march a corridor-class coupled world N steps twice — plain stack vs.
      stack + `PropulsionStub` at zero throttle — assert reports, final fields (scalars, force,
      command channels, regime), and logs bit-identical; failure names the first diverging
      artifact
- [ ] 4.2 Negative control: a deliberately non-inert stub configuration (test-local) makes the
      harness fail, proving the guard detects regressions

## 5. Atmosphere to the ground (examples/avionics_examples)

- [ ] 5.1 Extend `ATMOSPHERE` in `src/shared/constants.rs` with US-1976 rows at 0, 5, 10, 15,
      20, 25 km (four-column format, ascending order, existing five rows byte-identical), each
      row commented with its US-1976 pinpoint
- [ ] 5.2 Consistency test: `a = √(γ R_s T)` within transcription tolerance per new row;
      monotone density with altitude across the whole table
- [ ] 5.3 Sampler test against `DescentSchedule::sample`: identical samples 30–90 km between
      original and extended tables; a 15 km sample interpolates the new rows (no code change to
      `sample` itself)

## 6. Weather-table loader (examples/avionics_examples)

- [ ] 6.1 New `src/shared/weather_table.rs` (registered in `shared/mod.rs`): consumption row
      type implementing `FromTableRow` against the `WorldRow::SCHEMA` column names; loader
      reads via `deep_causality_file::read_rows`
- [ ] 6.2 Interpolation contract: sort ascending by `d_temp`; reject duplicate keys; bracket by
      value; interpolate all numeric columns linearly; clamp out-of-range dT to the nearest row
      with a `clamped` marker; result carries row + bracket + marker (loader does no logging)
- [ ] 6.3 Tests: committed CSV loads with recorded values; `dT = −15` brackets −25/−5 K;
      duplicate-key rejection; missing-column named error; `dT = −60` clamps to the −40 K row
      with the marker set; a consumer-side check that stamping a clamped result produces the
      provenance entry

## 7. Corridor inheritance re-run (guard prong A)

- [ ] 7.1 `cargo run --release -p avionics_examples --example plasma_blackout_corridor` — exit
      0, all gates pass, witnesses equal the committed `output.txt` (no band shifted, no new
      provenance)
- [ ] 7.2 `cargo run --release -p avionics_examples --example plasma_blackout_weather` — exit
      0, all table gates pass (the shared-constants edit is upstream of both examples)

## 8. Verification and PR preparation (SDD)

- [ ] 8.1 `bazel build //deep_causality_cfd/...` and `bazel test //deep_causality_cfd/...`
      green; targeted `cargo test -p avionics_examples` for the shared-lib tests
- [ ] 8.2 `make format && make fix` — clippy clean without suppressions
- [ ] 8.3 `make test` and `make check` (SDD pre-PR gate)
- [ ] 8.4 Prepare the commit message(s) per task group and hand to the user (never commit);
      draft the PR summary referencing this change and the roadmap milestone M2
