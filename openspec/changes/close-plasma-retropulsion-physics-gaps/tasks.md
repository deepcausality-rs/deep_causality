## 1. Scaffolding and registration (compiles empty)

- [ ] 1.1 Create `src/kernels/propulsion/` with `mod.rs` and empty topic files
      (`performance.rs`, `nozzle.rs`, `srp.rs`, `plume.rs`, `descent.rs`, `wrappers.rs`), SPDX
      headers and `//!` module docs
- [ ] 1.2 Register the chain: `kernels/mod.rs` (`pub(crate) mod propulsion;`),
      `lib.rs` flat re-export, `quantities/mod.rs` + new `quantities/propulsion/mod.rs`,
      `constants/mod.rs` + new `constants/propulsion.rs`
- [ ] 1.3 Verify which quantities already exist before adding any (`Force`, scalar
      acceleration, `MassFlowRate`, `Area`); add only the missing ones per the hypersonic
      newtype pattern (validating `new()`, `new_unchecked()`, `value()`, `Default`)
- [ ] 1.4 Check `src/constants/` for standard gravity; add `STANDARD_GRAVITY_M_S2 = 9.80665`
      with a typed accessor only if absent
- [ ] 1.5 Mirror the tests tree: `tests/kernels/propulsion/mod.rs` + one `<topic>_tests.rs`
      per topic file, registered in `tests/kernels/mod.rs`; add the `kernels_propulsion`
      `rust_test_suite` to `tests/BUILD.bazel`

## 2. Sources and digitization

- [ ] 2.1 Fetch and commit the public PDFs to `deep_causality_physics/papers/`:
      Jarvinen & Adams 1970 (NTRS 19720005324) and the Korzun–Braun–Cruz survey (Georgia Tech
      repository); for Cordell–Braun use the JSR 50(4) 2013 paper if publicly obtainable, else
      Cordell's Georgia Tech dissertation (SMARTech) — cite both in docstrings either way
- [ ] 2.2 Digitize the Jarvinen–Adams central-nozzle preserved-drag-vs-C_T curve and the
      unpowered baseline `C_A0(M)` (Mach 0.4–2.0) into `constants/propulsion.rs` tables, each
      under a source-block comment (publication, figure/table, units, digitization tolerance)
- [ ] 2.3 Record the C_T ≈ 3 instability onset as a cited constant, and the digitized validity
      domains (Mach band, C_T range) as exported bounds
- [ ] 2.4 Transcribe the Cordell–Braun analytic plume-geometry equation set (the
      effective-obstruction summary: max radius, penetration length, terminal-shock standoff)
      with the paper's comparison-case values captured for the tests

## 3. Kernels + pointwise tests (one topic at a time, tests beside each)

- [ ] 3.1 `performance.rs`: `propellant_mass_flow_kernel`, `tsiolkovsky_delta_v_kernel` +
      published-value and rejection tests
- [ ] 3.2 `nozzle.rs`: `inverse_area_mach_kernel` (branch-selected; round-trip vs the existing
      forward kernel) and `nozzle_exit_state_kernel` (composes existing isentropic kernels —
      no formula restated) + published nozzle-case tests
- [ ] 3.3 `srp.rs`: `srp_thrust_coefficient_kernel`, `momentum_flux_ratio_kernel`,
      `srp_preserved_drag_fraction_kernel`, `jarvinen_adams_baseline_axial_coefficient_kernel`,
      `srp_total_axial_force_coefficient_kernel`, `srp_stability_margin_kernel` + digitized-
      point, drag-collapse-structure, non-monotone-band, and out-of-domain-rejection tests
- [ ] 3.4 `plume.rs`: `PlumeGeometry` quantity + `cordell_braun_plume_boundary_kernel` +
      published-comparison, throttle-sensitivity (two states → two geometries), and
      validity-envelope tests
- [ ] 3.5 `descent.rs`: `stopping_distance_kernel`, `ignition_altitude_kernel`,
      `suicide_burn_deceleration_kernel` + kinematic-identity, touchdown-nulling, and
      thrust-to-weight/ground-contact rejection tests
- [ ] 3.6 `wrappers.rs`: one `PropagatingEffect` wrapper per kernel (kernel name minus
      `_kernel`), wired through the module re-exports

## 4. Verification

- [ ] 4.1 `cargo test -p deep_causality_physics` green; every added kernel's nominal, limit,
      and rejection paths covered (the crate's 100%-coverage rule)
- [ ] 4.2 `bazel test //...` green including the new `kernels_propulsion` suite
- [ ] 4.3 `make format && make fix` clean (lints fixed, never suppressed)
- [ ] 4.4 Duplication gate: the new family calls, and does not restate, the existing
      compressible/isentropic kernels (inspect imports; no isentropic or area-Mach formula
      text appears under `kernels/propulsion/`)
- [ ] 4.5 Prepare the commit message and hand it to the user (never commit; golden rule)
