## 1. RetroThrust production stage (deep_causality_cfd)

- [x] 1.1 Add `RetroThrust` to `types/flow/retropulsion.rs` (beside `PropulsionStub`, re-exported
      from `lib.rs`): reads the throttle from the throttle channel or the published
      `"commanded_throttle"`; at throttle > 0 forms `−(T/m)·v̂` (`T = throttle·thrust_ref`, `m` the
      carried `"mass"`, `v̂` the flight-velocity axis) and composes it via `add_aero_force`; errors on
      a non-positive mass under active throttle; strictly inert at throttle ≤ 0
- [x] 1.2 Depletion: reduce `"propellant"` and `"mass"` by `ṁ·Δt` (`ṁ` from
      `propellant_mass_flow_kernel`, `Isp` a stage parameter), using the start-of-step mass for the
      thrust normalization; set `"ignited"`
- [x] 1.3 Tests (`tests/types/flow/retropulsion_tests.rs`): thrust composes onto lift (axial gains
      `−T/m`, lateral unchanged); propellant/mass fall by `ṁ·Δt`; `"ignited"` set; zero/absent
      throttle strictly inert; active-without-mass errors; navigation reads the summed force

## 2. PlumeObstruction production stage (deep_causality_cfd)

- [x] 2.1 Add `PlumeObstruction` to `types/flow/retropulsion.rs`: at throttle > 0, derive
      `C_T = srp_thrust_coefficient_kernel(T, q∞, S_ref)` (sizing the plume input via
      `momentum_flux_ratio_kernel`), read `srp_preserved_drag_fraction_kernel(C_T)`, and scale the
      axial forebody drag on the channel by the preserved fraction; strictly inert at throttle ≤ 0
- [x] 2.2 Optional state-realism imprint behind an explicit opt-in: a `plume_mask_2d` mask from
      `cordell_braun_plume_boundary_kernel` at the world's `C_T` applied through the `ForcingRegion`
      seam; the imprint MUST NOT alter the force-channel decrement (drag closure is the A0 fraction
      with or without it)
- [x] 2.3 Tests: the applied fraction equals `srp_preserved_drag_fraction_kernel(C_T)` within the
      digitization tolerance (cross-check against M1's band); imprint-on vs imprint-off give the same
      force-channel decrement; zero throttle strictly inert

## 3. Flight-regime classifier axes (deep_causality_cfd)

- [x] 3.1 Extend `RegimeClass` (`corridor.rs`) with a Mach regime, a thrust state, and a touchdown
      flag; extend `RegimeClassify` to read `"flight_mach"` (configurable thresholds), `"ignited"`,
      and `"flight_altitude"` (configurable floor); default the new axes neutral when their scalars
      are absent
- [x] 3.2 Fold the new axes into `key()` so a Mach crossing, a burn↔coast transition, or a touchdown
      is a logged regime change; the continuous values stay out of the key (no spurious re-logs)
- [x] 3.3 Tests (`tests/types/flow/corridor_tests.rs`): each axis reads its scalar; the corridor
      classification is bit-identical when the burn scalars are absent; a Mach crossing logs once and
      the same band re-logs nothing; a burn↔coast flip logs; a touchdown logs

## 4. Burn-leg cascade harness (deep_causality_cfd)

- [x] 4.1 Harness test extending the M2 inheritance-guard pattern: march a corridor-class coupled
      world through a commanded burn (RetroThrust + PlumeObstruction + the extended classifier), and
      assert the ordered regime cascade appears in provenance (aero → thrust-dominated, the Mach
      crossings under thrust, burn↔coast), the applied decrement is inside M1's band, and no step
      captured an error (integrity)
- [x] 4.2 Register new/edited test files in their `mod.rs` chain and the `tests/BUILD.bazel` globs
      (`bazel test //deep_causality_cfd/...`)

## 5. Verification and PR preparation (SDD)

- [x] 5.1 `bazel test //deep_causality_cfd/...` green; the corridor + weather examples re-run
      bit-identical (the classifier extension is inert on the corridor's scalars)
- [x] 5.2 `make format && make fix` — clippy clean without suppressions
- [ ] 5.3 `make test` and `make check` (full-repo SDD pre-PR gate)
- [x] 5.4 Prepare the commit message(s) per task group and hand to the user (never commit); draft the
      PR summary referencing this change and roadmap milestone M3
