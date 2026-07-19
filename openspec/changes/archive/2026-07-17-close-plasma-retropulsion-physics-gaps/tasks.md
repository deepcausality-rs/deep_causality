## 1. Scaffolding and registration (compiles empty)

- [x] 1.1 Create `src/kernels/propulsion/` with `mod.rs` and topic files
      (`performance.rs`, `nozzle.rs`, `srp.rs`, `plume.rs`, `descent.rs`, `wrappers.rs`), SPDX
      headers and `//!` module docs
- [x] 1.2 Register the chain: `kernels/mod.rs` (`pub(crate) mod propulsion;`),
      `lib.rs` flat re-export, `quantities/mod.rs` + new `quantities/propulsion/mod.rs`,
      `constants/mod.rs` + new `constants/propulsion.rs`
- [x] 1.3 Verify which quantities already exist before adding any: `Force`, `Acceleration`,
      `Area`, `Pressure`, `Temperature`, `Density`, `Speed`, `Length`, `Mass` all exist and are
      reused; added only `MassFlowRate`, `FlowBranch`, `NozzleExitState`, `PlumeGeometry` under
      `quantities/propulsion/`
- [x] 1.4 Standard gravity already present as `EARTH_GRAVITY_ACCELERATION` /
      `constants::G` (9.80665) — reused, no new constant added
- [x] 1.5 Mirror the tests tree: `tests/kernels/propulsion/mod.rs` + one `<topic>_tests.rs`
      per topic file, registered in `tests/kernels/mod.rs`; added the `kernels_propulsion`
      `rust_test_suite` to `tests/BUILD.bazel`

## 2. Sources and digitization

- [x] 2.1 Public PDFs committed to `deep_causality_physics/papers/` (SDD rule — public only):
      Jarvinen & Adams 1970 (`jarvinen_adams_1970_ntrs_19720005324.pdf`), the Korzun–Cruz–Braun
      survey (`korzun_braun_cruz_srp_survey.pdf`), and Cordell's Georgia Tech dissertation
      (`cordell_2013_srp_analytic.pdf`, 355 pp., DSpace bitstream). The paywalled Cordell–Braun
      JSR article was NOT downloaded; the dissertation is the primary citation.
- [x] 2.2 Digitized the Jarvinen–Adams central-nozzle preserved-drag-vs-C_T curve (Fig. 32/56,
      M∞ = 2.0) and the unpowered baseline `C_A0(M)` (M ∈ [0.6, 2.0]) into
      `constants/propulsion.rs`, each under a source-block comment with figure pinpoint and
      digitization uncertainty
- [x] 2.3 Recorded the jet-penetration→blunt-flow transition (central, C_T ≈ 1 at M = 2, fixed
      in P_ej/P∞ ≈ 7.0–7.2) AND the separate peripheral C_T ≈ 3 rippling onset as cited
      constants, plus the Cordell Mach/γ validity envelopes as exported bounds. **Correction
      from digitized sources:** the original spec's "C_T ≈ 3 bow-shock instability" for the
      central nozzle was wrong — that onset is peripheral; the central-nozzle transition is at
      low C_T. Kernel renamed `srp_stability_margin` → `srp_flow_regime_margin`; spec/design
      updated.
- [x] 2.4 Transcribed the Cordell analytic plume model (Ch. III Eqs. 7–40) — terminal shock
      (Sibulkin + stagnation balance), Charwat barrel shape, mass-flow scaling — with the
      dissertation's printed Table 13 (jet-edge Mach), Fig. 54 (terminal Mach), and Fig. 55
      (standoff/body-dia) comparison values captured as pointwise test anchors

## 3. Kernels + pointwise tests (one topic at a time, tests beside each)

- [x] 3.1 `performance.rs`: `propellant_mass_flow_kernel`, `tsiolkovsky_delta_v_kernel` +
      published-value (Merlin-1D T/Isp, unit-log-ratio) and rejection tests
- [x] 3.2 `nozzle.rs`: `inverse_area_mach_kernel` (branch-selected; round-trip vs the existing
      forward kernel; NACA 1135 table values) and `nozzle_exit_state_kernel` (composes existing
      isentropic kernels — no formula restated) + published nozzle-case tests
- [x] 3.3 `srp.rs`: `srp_thrust_coefficient_kernel`, `momentum_flux_ratio_kernel`,
      `srp_preserved_drag_fraction_kernel`, `jarvinen_adams_baseline_axial_coefficient_kernel`,
      `srp_total_axial_force_coefficient_kernel`, `srp_flow_regime_margin_kernel` + digitized-
      point, drag-collapse-structure, non-monotone-band, and out-of-domain-rejection tests
- [x] 3.4 `plume.rs`: `PlumeGeometry` quantity + sub-kernels (`prandtl_meyer_kernel`,
      `choked_mass_flow_kernel`, `srp_post_bow_shock_total_pressure_kernel`,
      `srp_terminal_shock_mach_kernel`, `srp_jet_edge_mach_kernel`) +
      `cordell_braun_plume_boundary_kernel` + Table-13 / Fig-54 / Fig-55 anchor tests,
      throttle-sensitivity, and validity-envelope tests
- [x] 3.5 `descent.rs`: `stopping_distance_kernel`, `ignition_altitude_kernel`,
      `suicide_burn_deceleration_kernel` + kinematic-identity, integrated-touchdown-invariant,
      and thrust-to-weight/ground-contact rejection tests
- [x] 3.6 `wrappers.rs`: one `PropagatingEffect` wrapper per kernel (kernel name minus
      `_kernel`), wired through the module re-exports, with success/error-arm wrapper tests

## 4. Verification

- [x] 4.1 `bazel test //deep_causality_physics/...` green (184 tests); every added kernel's
      nominal, limit, and rejection paths covered (per the crate's 100%-coverage rule)
- [x] 4.2 `bazel build //deep_causality_physics/...` and the new `kernels_propulsion` suite green
- [x] 4.3 `cargo fmt` + `cargo clippy -p deep_causality_physics --all-targets` clean (0 warnings;
      lints fixed, never suppressed)
- [x] 4.4 Duplication gate: the family calls the existing
      `isentropic_pressure_ratio_kernel` / `isentropic_temperature_ratio_kernel` /
      `area_mach_ratio_kernel` / `speed_of_sound_ideal_gas_kernel`; no isentropic or area-Mach
      formula is restated under `kernels/propulsion/` (the composition kernels call siblings)
- [x] 4.5 Adversarial verification pass over all five kernel files (physics/soundness). Three
      findings, all resolved: (a) `inverse_area_mach_kernel` subsonic bracket now rejects
      absurd area ratios (A/A* > ~5.8e8) instead of silently converging — fixed + test; (b)
      `srp_terminal_shock_mach_kernel` now guards `γ > 1` like its siblings — fixed + test; (c)
      the Cordell barrel-outflow `dx/norm` factor was flagged as physically non-standard —
      checked against the dissertation PDF (Eq. 38, p. 92), confirmed a faithful verbatim
      transcription of the published model, kept with a comment pinning it to Eq. 38 (removing
      it would deviate from the cited source). Re-ran: clippy clean, 184 physics tests pass.
- [x] 4.6 Prepare the commit message and hand it to the user (never commit; golden rule)
