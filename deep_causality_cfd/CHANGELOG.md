# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_cfd-v0.1.0...deep_causality_cfd-v0.1.1) - 2026-08-25

### Added

- *(website)* rework the CFD landing page and re-sync every published number to its artifact

### Fixed

- *(release)* bump algebra to 0.3.0 so the new traits are actually published
- *(deep_causality_cfd)* fixes std feat bug.
- *(clippy)* silence chunks_exact_to_as_chunks and useless_format

### Other

- *(deep_causality_cfd)* Updated SBOM.
- Updated version requirmentes
- *(cfd)* run slow verification harnesses monthly, not nightly
- Merge remote-tracking branch 'origin/main'

## [0.1.0](https://github.com/deepcausality-rs/deep_causality/releases/tag/deep_causality_cfd-v0.1.0) - 2026-08-12

### Added

- *(bazel)* build and run the library-crate examples under Bazel
- *(cfd)* close the QTT solver's numerical envelope (items 12, 13, 14; 10 partial)
- *(cfd)* add DEC scalar transport and a real Fourier-law wall heat flux
- *(cfd)* enforce the BlendedMap validity guarantee, wire the constrained-edge
- *(cfd)* make the verification evidence layer enforceable
- *(deep_causality_cfd)* typed witnesses for commit, re-seeds, transitions, and peak bond
- *(deep_causality_cfd)* typed witnesses for commit, re-seeds, transitions, and peak bond
- *(deep_causality_cfd)* stopping-burn guidance + fork economics on branch reports
- *(deep_causality_cfd)* Enabled crate publishing to crates.io
- *(deep_causality_cfd)* M5 plasma-retropulsion example — blackout exit to touchdown
- *(deep_causality_cfd)* M4 terminal descent — guidance, ignition commit, live envelope enforcement
- *(deep_causality_cfd)* M3 coupled stages — RetroThrust, PlumeObstruction, flight-regime axes
- *(deep_causality_cfd)* M2 contracts — burn envelope, inheritance guard, atmosphere, keyed-table lookup
- *(deep_causality_cfd)* blackout-coupling-interface — throttle channel, additive force, A0 stub
- *(deep_causality_cfd)* SRP momentum-jet de-risk — risk-1 amber cause revised to harness capacity
- *(deep_causality_cfd)* de-risk SRP plume coupling — forcing seam, measurements, AMBER verdict
- *(cfd,examples)* origin-form counterfactual + ensemble machinery; migrate plasma_blackout_weather (DSL rework 7.5, 4.7c/4.8)
- *(examples)* migrate plasma_blackout_corridor to the grammar — the acid test (DSL rework 7.6)
- *(cfd)* the refine two-round machinery + public StudyView::of (DSL rework 4.9)
- *(cfd)* event-fork counterfactual campaign verbs + carrier lifetime refinement (DSL rework 4.7a/4.7b)
- *(cfd)* the campaign grammar core — study phases, gates, verdict (DSL rework group 4)
- *(cfd)* StudyEffect substrate — the CDL effect pattern for the study grammar
- *(cfd)* MarchState + named-stage coupled march builder (DSL rework group 2)
- *(cfd)* Marchable trait + singular continue_with (DSL rework group 2, additive)
- *(cfd,physics,file,examples)* study DSL (sweep, Gates, run_owned, duct_march) + three gated examples
- *(file,cfd)* CFD file IO seams — typed tables, sensor traces, snapshot/resume
- *(cfd,physics,examples)* finite-rate ionization network, two-stage counterfactual sweep, review fixes, MDAO positioning
- *(physics,cfd,avionics_examples)* uncalibrated finite-rate ionization network (lever 3)
- *(deep_causality_par)* scoped fork-join + parallel counterfactual fan-out
- *(deep_causality_cfd)* compressible blackout carrier — one continuous descent
- *(cfd)* noisy GNSS fixes + recombination doc; sync and archive the corridor spec
- *(cfd)* corridor fidelity upgrades — Park-2T controller, wall heat flux, IMU drift
- *(examples)* plasma-blackout corridor flagship (Stage 5) — chain [1]-[7] in the CfdFlow DSL
- *(cfd)* Flow DSL redesign (Stage 4) — TrajectoryNav, alternation verbs, resumable fork
- *(cfd)* corridor composition stages (Stage 3) — classify, branch, cybernetic gate
- *(cfd)* navigation engine (Stage 2) — ESKF + regime switch, in the CFD crate
- *(cfd)* real marcher→④ aero-force coupling, replacing the stub (Stage 1.3)
- *(cfd)* body-fitted 3-D compressible marcher over MetricProvider3d (Stage 1.1d)
- *(cfd)* body-fitted 3-D compressible marcher over MetricProvider3d (Stage 1.1d)
- *(cfd)* BodyFittedCoordinate3d — spherical-shell fitted 3-D metric (Stage 1.1b)
- *(cfd)* 3-D MetricProvider3d seam + Cartesian capture limit (Stage 1.1a)
- *(physics,cfd)* Stage 0 — KS conformal core, constraint projection, blackout coupling seam
- *(cfd)* extend the .couple seam with the blackout navigation channels (Stage 0.1/0.2)
- *(physics)* Gap-3 trajectory-axis spec-readiness + 3 feasibility studies + promoted clock/Kepler kernels
- *(deep_causality_cfd)* Gap-3 chemistry fidelity — T_ve-controlled ionization (12× → 1.1×)
- *(deep_causality_cfd)* Tier-B Stage 6 — 3-D marcher + 3-D acoustic inverse, rank-lever verification (5.2/6.2)
- *(deep_causality_cfd)* D10 closed-form acoustic inverse, Stage-5 IMEX marcher,
- *(deep_causality_cfd)* D10 closed-form acoustic-core inverse + Stage-5 IMEX marcher
- *(deep_causality_cfd)* Tier-B Stage 4 — RAM-C stagnation line (shock fitting + reused LER)
- *(deep_causality_cfd)* Tier-B Stage 3 — IMEX split-acoustic integrator (D10)
- *(deep_causality_cfd)* MetricProvider seam — body-fit as data, not a code path (Tier-B D8)
- *(deep_causality_cfd)* round-2 QTT feasibility studies de-risking Tier-B resolutions 4/5/6
- *(deep_causality_cfd)* Tier-B compressible QTT marcher, Stages 0-2 (3-D ops, body-fitted coordinate, Sod-gated compressible flux)
- *(deep_causality_cfd)* plasma-blackout LER coupling + QTT coupling seam (Gap-2 closed)
- *(deep_causality_cfd)* add QTT tensor-rank studies (static, dynamic, nonlinear, 3-D)
- *(deep_causality_cfd)* QTT immersed body + surface observables (closes Gap 1)
- *(deep_causality_cfd)* QTT immersed body + surface observables (closes Gap 1)
- *(deep_causality_cfd)* CfdFlow wiring + observables for the QTT solver
- *(deep_causality_cfd)* QTT 2-D incompressible Navier-Stokes solver
- *(deep_causality_cfd)* QTT tensor-bridge — codec, FD operators, linear rollout
- feat(deep_causality_cfd) migrated all solver verfications into the deep_causality_cfd crate under the verfication folder
- *(deep_causality_cfd)* holistic per-solver benchmarks
- *(deep_causality_cfd)* holistic per-solver benchmarks
- *(deep_causality_cfd)* formatting
- *(deep_causality_cfd)* CSV writers over the IO effect
- *(deep_causality_cfd)* lift the sensor-fed uncertain march into the CfdFlow DSL
- *(examples)* add example_ml_rca
- *(deep_causality_cfd)* port opt-in QMC collapse into the canonical uncertain-inflow
- *(deep_causality_cfd)* graded-geometry corpus + autodiff manufactured-solution MMS seam
- *(exampke)* rewrote the TG1600 example using CfdFlow DSL.
- *(deep_causality_cfd)* Flow non-marching solver kinds + .couple multiphysics
- *(deep_causality_cfd)* Flow immersed-body diagnostics — drag/lift, wake probe + Strouhal, centerline, uniform-x seed
- *(deep_causality_cfd)* theory-naming consistency + Flow zones/cut-cell/steady-march
- *(deep_causality_cfd)* Flow DSL facade — owned march case + fluent builder
- *(cfd)* scaffold deep_causality_cfd and migrate the fluid stack + tests

### Fixed

- *(deep_causality_cfd)* reject non-finite seed widths; build the cut registry from the graded metric
- *(deep_causality_cfd)* repair navigation error paths, snapshot versioning, and doc accuracy
- *(deep_causality_cfd)* reject a non-finite measurement in the ESKF update
- *(deep_causality_cfd)* close the error-state Kalman filter's correctness defects
- *(deep_causality_cfd)* correct the RAM-C vibrational-relaxation reduced mass (μ 7.0 → 14.007)
- *(cfd)* correct a false deferral, a wrong rationale, and two misplaced files
- *(deep_causality_cfd)* Put crates.io publication on hold until the internal validation has been completed.
- *(examples)* make the retropulsion counterfactual measure the flight
- *(examples)* score retropulsion branches from the flown state, and rebuild the gates on it
- *(deep_causality_cfd)* Minor fix
- *(deep_causality_cfd)* Code formatting
- *(deep_causality_physics)* Fixing 10MB max upload limit on crates.io
- *(deep_causality_cfd)* Seed::TaylorGreenVortex panicked with an index-out-of-bounds on
- *(deep_causality_cfd)* Fixed miri test config

### Other

- *(website)* correct world.rs line citations after the config-builder refactor
- *(deep_causality_cfd)* [**breaking**] route every config family through CfdConfigBuilder
- *(deep_causality_cfd)* Updated README and enables crate publishing to crates.io
- *(deep_causality_cfd)* Updated README.md
- *(deep_causality_cfd)* reconcile all 125 catalogued doc rows, trace constants, retire Gates
- *(deep_causality_cfd)* reconcile docs with code, trace constants, retire Gates
- *(openspec)* resolve audit B-1, sync specs, archive fix-ramc-vibrational-relaxation-pair
- *(deep_causality_cfd)* correct stale figures, commands and links in study/verification READMEs
- *(deep_causality_cfd)* README for every study;
- *(deep_causality_cfd)* add READMEs for the two QTT rank-lever gates; refresh the wake README
- *(deep_causality_cfd)* split three oversized flow modules by seam
- *(examples)* move plasma-blackout examples into plasma_blackout/{corridor,weather}
- release
- *(deep_causality_cfd)* internal source re organization.
- Improved test coverage.
- release
- *(num)* split deep_causality_num into num-core + algebra + complex + dual
- *(deep_causality_cfd)* speed up slow solver tests (25.6s -> 10.4s)
- *(cfd)* adapt to CausalEffect (EffectValue::Value/None -> CausalEffect::value/none)
- *(cfd)* adapt to core rename intervene → alternate_value
- *(core)* [**breaking**] enforce the W-invariant — value-XOR-error as one channel
- code formatting and linting.
- code formatting and linting.
- code formatting and linting.
- *(deep_causality_cfd)* Increased test coverage
- code formatting and linting.
- *(deep_causality_cfd)* Increased test coverage
- *(cfd)* update README to the reworked two-level CfdFlow grammar
- *(openspec)* sync rework-cfd-flow-dsl delta specs to main specs and archive
- *(cfd)* [**breaking**] retire the legacy IO writers, fail, and the five march entries (DSL rework group 6)
- *(cfd)* compile_fail doctests for the study phase discipline (DSL rework group 5.4)
- *(openspec)* Added new spec add-cfd-file-io and related notes
- *(deep_causality_cfd)* Added a README.
- Restructured the avionics example folder.
- *(deep_causality_cfd)* impoved BAzel test config
- *(deep_causality_cfd)* impoved test coverage
- *(deep_causality_cfd)* Fixed various docstring issues,
- *(cfd)* 3-D rank-lever gate — body-fitted O(1) vs Cartesian growth (Stage 1.1c)
- *(openspec)* finalize Tier-B compressible marcher — Stages 0–6 built, scope labels, status notes
- *(deep_causality_cfd)* QTT Taylor-Green verification example
- raise test coverage across 8 crates.
- *(deep_causality_cfd)* Improved test coverage;
- Generated new SBOM for all crates.
- Added a readme with an analysis of the verification results in the cfd crate
- *(papers)* Reorganized publication by moving each paper into the crate where it is actually implemented.
- *(deep_causality_cfd)* close coverage gaps with corner/error-path tests
- *(deep_causality_cfd)* add Bazel BUILD config for tests
- *(deep_causality_cfd)* split configuration from workflow — CfdConfigBuilder + CfdFlow (B1)
