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

- [x] 2.1 Add `BurnEnvelope<R>` (throttle floor/ceiling, `max_ct`, ignition q window
      `[q_min, q_max]`, propellant floor, max descent rate) and `burn: Option<BurnEnvelope<R>>`
      on `SafetyEnvelope`; keep the 3-arg `new()` signature yielding `burn: None`; add the
      explicit burn-axis builder step (`with_burn_envelope`-style)
- [x] 2.2 Extend `CyberneticCorrect` to enforce active burn axes with the existing
      sense → clamp → `Err` pattern: throttle clamp into `[floor, min(ceiling, ct_ceiling(q∞))]`
      (dynamic C_T cap from the configured thrust reference + reference area), logged when it
      changes the command; propellant-floor and descent-rate breaches log + return the same
      `Err(PhysicalInvariantBroken)`; sensing via configurable field names (the
      `heat_flux_field` pattern)
- [x] 2.3 Tests with synthetic scalar fields: dynamic cap binds below the static ceiling;
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

- [x] 4.1 Harness test: march a corridor-class coupled world N steps twice — plain stack vs.
      stack + `PropulsionStub` at zero throttle — assert reports, final fields (scalars, force,
      command channels, regime), and logs bit-identical; failure names the first diverging
      artifact
- [x] 4.2 Negative control: a deliberately non-inert stub configuration (test-local) makes the
      harness fail, proving the guard detects regressions

## 5. Atmosphere to the ground (examples/avionics_examples)

Example DATA (pinned to RAM-C), not general-purpose — so it stays in the example and is verified
by re-running the example (task 7), never by an example-crate unit test (house rule: examples are
example code only; general-purpose logic lives in a lib crate and is tested there).

- [x] 5.1 Extend `ATMOSPHERE` in `src/shared/constants.rs` with US-1976 rows at 0, 5, 10, 15,
      20, 25 km (four-column format, ascending order, existing five rows byte-identical), each
      row commented with its US-1976 pinpoint
- [x] 5.2 Consistency ensured by construction: `a = √(γ R_s T)` at γ = 1.4, R = 287 for each new
      row (340.2 / 320.5 / 299.5 / 295.1 / 295.1 / 298.4 m/s); `n_tot` monotone-decreasing across
      the whole table (2.5e25 → 7.0e19). Verified end-to-end by task 7's example re-run.
- [x] 5.3 `DescentSchedule::sample` is unchanged (already clamps to table ends); its bracket/clamp
      behavior is tested in `deep_causality_cfd`. The extended table's corridor sampling (well
      above 30 km) is proven unchanged by task 7's bit-identical re-run.

## 6. Value-bracketed table lookup (general-purpose → `deep_causality_cfd`)

Deviation from the original D7/D8 (loader in `avionics_examples::shared`): per the house rule, the
**reusable** core — value-bracketed linear interpolation over sorted-unique keyed rows with a clamp
marker — lands in a lib crate and is tested there; the WorldRow-bound CSV read + `EffectLog`
provenance stamping is example glue deferred to M5 (which owns the retropulsion example).

- [x] 6.1 `KeyedTable<R>` + `KeyedInterpolation<R>` in `deep_causality_cfd`
      (`src/types/keyed_table.rs`, re-exported from `lib.rs`): the N-column generalization of
      `DescentSchedule::sample`. `new()` sorts ascending by key, rejects duplicate keys and ragged
      rows; `interpolate()` brackets by value, clamps to the nearest end row with the `clamped`
      marker, and reports the bracketing indices. Loader is pure (no logging).
- [x] 6.2 Interpolation contract as above: sort ascending, reject duplicates, bracket by value,
      interpolate every column linearly, clamp out-of-range with the marker; result carries the
      interpolated values + bracket indices + `clamped` marker.
- [x] 6.3 Tests in `deep_causality_cfd` (`tests/types/keyed_table_tests.rs`, own Bazel target
      `types_keyed_table`): run-order rows sort ascending; `dT = −15` brackets the −25/−5 rows with
      columns between; exact-key and end-key handling; `dT = −60` clamps to the first row with the
      marker; duplicate-key, empty, and ragged-row rejection.
- [ ] 6.4 (M5) Example glue: a `weather_table` loader reading `weather_table.csv` via
      `deep_causality_file::read_rows` with a `WorldRow::SCHEMA`-bound `FromTableRow` row type,
      feeding `KeyedTable`, and stamping a clamped result into the flight `EffectLog`. Deferred to
      the M5 retropulsion example (example code, no example-crate tests).

## 7. Corridor inheritance re-run (guard prong A)

- [x] 7.1 `cargo run --release -p avionics_examples --example plasma_blackout_corridor` — exit 0,
      all 13 gates pass, witnesses bit-identical to the committed `output.txt` (only the
      non-deterministic wall-clock line differs)
- [x] 7.2 `cargo run --release -p avionics_examples --example plasma_blackout_weather` — exit 0,
      all 8 table gates pass, every witness bit-identical (onset spread 4.2 s, drift 68.75/45.93 m,
      4.0 σ). NOTE: the committed weather `output.txt` carries a stale "Audit:" summary line that no
      longer exists in any repo source — a pre-existing drift unrelated to the atmosphere append.

## 8. Verification and PR preparation (SDD)

- [x] 8.1 `bazel test //deep_causality_cfd/...` green for the touched targets (`types_flow`,
      `types_keyed_table`); no example-crate tests (examples are example code only)
- [x] 8.2 `make format && make fix` — clippy clean without suppressions
- [ ] 8.3 `make test` and `make check` (full-repo SDD pre-PR gate — run before PR)
- [x] 8.4 Prepare the commit message(s) per task group and hand to the user (never commit);
      draft the PR summary referencing this change and the roadmap milestone M2
