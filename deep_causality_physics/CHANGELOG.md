# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.8.1](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_physics-v0.8.0...deep_causality_physics-v0.8.1) - 2026-07-24

### Added

- *(deep_causality_physics)* add propulsion kernel family (SRP Stage 1)

### Other

- *(deep_causality_cfd)* add READMEs for the two QTT rank-lever gates; refresh the wake README
- *(deep_causality_physics)* improved test coverage

## [0.8.0](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_physics-v0.7.0...deep_causality_physics-v0.8.0) - 2026-07-14

### Added

- *(deep_causality_quantum)* new crate — quantum-information kernels move out of physics (Phase 1)

### Other

- build(bazel)P: Updated Bazel config
- Improved test coverage.

## [0.7.0](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_physics-v0.6.3...deep_causality_physics-v0.7.0) - 2026-07-08

### Added

- *(cfd,physics,file,examples)* study DSL (sweep, Gates, run_owned, duct_march) + three gated examples
- *(cfd,physics,examples)* finite-rate ionization network, two-stage counterfactual sweep, review fixes, MDAO positioning
- *(physics,cfd,avionics_examples)* uncalibrated finite-rate ionization network (lever 3)
- *(cfd)* navigation engine (Stage 2) — ESKF + regime switch, in the CFD crate
- *(physics)* synthetic sensors + closed-loop nav gate (Stage 2.2)
- *(physics)* cohesive ReentryNavEngine — KS + ESKF + two-clock (Stage 2.1c)
- *(physics)* 17-state ESKF covariance + measurement update (Stage 2.1b)
- *(physics)* strapdown-INS error state + drift laws — nav-engine core (Stage 2.1a)
- *(cfd)* BodyFittedCoordinate3d — spherical-shell fitted 3-D metric (Stage 1.1b)
- *(physics,cfd)* Stage 0 — KS conformal core, constraint projection, blackout coupling seam
- *(physics)* Gap-3 trajectory-axis spec-readiness + 3 feasibility studies + promoted clock/Kepler kernels
- *(deep_causality_physics)* make nuclear kernels generic over RealField
- *(deep_causality_physics)* make condensed-matter kernels generic over RealField
- *(deep_causality_physics)* add Park-2T hypersonic reacting/ionization kernels
- *(deep_causality_physics)* removed unused deps
- *(deep_causality_physics)* opt-in Quasi-Monte-Carlo collapse for the uncertain inflow
- *(cfd)* scaffold deep_causality_cfd and migrate the fluid stack + tests
- *(deep_causality_physics)* staircase-vs-aperture-resolved no-slip toggle + cylinder validation harness
- *(deep_causality_physics)* wire aperture-resolved immersed no-slip into the DEC solver
- *(deep_causality_physics)* one-sided wall-normal friction diagnostic (Kirkpatrick true Δh)
- *(deep_causality_physics)* free outflow tangential edges (zero-gradient outflow)
- *(deep_causality_physics)* pressure surface-force diagnostic on cut bodies
- *(deep_causality_physics)* free-slip / far-field boundary zone
- *(deep_causality_physics)* inflow/outflow boundary zones + open-projection wiring
- *(deep_causality_physics)* inflow/outflow boundary zones + open-projection wiring
- *(deep_causality_physics)* composable boundary-zone abstraction (static dispatch)
- *(deep_causality_physics)* CFD Stage-4 Group C — the uncertain-inflow zone
- *(cfd)* Small-cut-cell stabilization — CFD Stage 4 B1–B3 (cell-merging; inherent stability finding)
- *(cfd)* Cut-cell cylinder wake harness — CFD Stage 4 Group D
- feat(deep_causality_physics):
- feat(deep_causality_physics):
- *(deep_causality_physics)* Add the wall-bounded DEC Navier-Stokes
- *(deep_causality_fft)* Add FFT crate, deep_causality_par, and the spectral Poisson solve (closes add-fft)
- *(deep_causality_physics)* Update  readme with parallel flag.
- *(deep_causality_physics)* Add the periodic DEC-native incompressible
- *(deep_causality_physics)* Add typed fluid-dynamics form units incl. the

### Fixed

- *(haft)* [**breaking**] align law docs with proved theory; make effect-system reference impl lawful
- *(deep_causality_rand)* Format and linting
- *(deep_causality_physics)* Fixed bazel test config
- *(deep_causality_topology)* Diagnosis (2026-06-12, task 1.3 — budget probe on the 32³ Re-1600 trajectory): hypothesis (1) confirmed. The convective power ⟨u, −i_u(du)⟩_M is exactly zero on the smooth single-mode initial state (−9e-16), turns positive as the spectrum fills (+0.59 at t* 3.1, +28 at 7.9), and overwhelms the viscous sink (always properly negative) at t* ≈ 8.5 — energy growth follows.
- fixed sone doctest warnings

### Other

- *(num)* split deep_causality_num into num-core + algebra + complex + dual
- *(deep_causality_physics)* fixed miri test failures.
- *(physics,tensor,file)* skip Miri-incompatible tests
- *(core)* [**breaking**] enforce the W-invariant — value-XOR-error as one channel
- strengthen physics quantity assertions and fix review nits
- *(deep_causality_physics)* split quantity tests into per-quantity leaves
- *(deep_causality_physics)* close reachable coverage gaps; document the rest
- raise test coverage across 8 crates.
- Generated new SBOM for all crates.
- *(papers)* Reorganized publication by moving each paper into the crate where it is actually implemented.
- Merge remote-tracking branch 'origin/main'
- *(deep_causality_physics)* updated README
- *(deep_causality_physics)* [**breaking**] remove the fluid-dynamics theories (consolidated into deep_causality_cfd)
- *(dec-solver)* warm-start the λ (cut-face multiplier) block in the weighted projection
- *(dec-solver)* projection CG warm-start + cycle-mean cylinder drag
- *(deep_causality_physics)* cross-domain UncertainBoundarySource
- *(deep_causality_physics)* Improved testing.
- *(deep_causality_physics)* Verify the convective-instability fix —
- added new specs to fix a bug
- *(deep_causality_topology)* Add compiled DEC stencil tables and the
- *(deep_causality_num)* Add table-based fast path for Float106::sin_cos
- *(deep_causality_topology)* Add DEC solver benchmark, eliminate
- *(deep_causality_topology)* Memoize boundary matrices and preserve
- *(deep_causality_physics)* Consolidate quantity tests into tests/quantities/
- *(deep_causality_physics)* Consolidate quantities to 13 coherent files (19 → 13)
- *(deep_causality_physics)* Consolidate all quantities into src/quantities/

## [0.6.3](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_physics-v0.6.2...deep_causality_physics-v0.6.3) - 2026-06-12

### Other

- *(release)* bump deep_causality_core to 0.10.0 and realign dependent pins

## [0.6.2](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_physics-v0.6.1...deep_causality_physics-v0.6.2) - 2026-06-09

### Added

- *(deep_causality_num)* forward-mode autodiff surface over Dual
- *(deep_causality_num)* split Real from RealField; blanket the float tower
- *(deep_causality_num)* split Real out of RealField

### Other

- *(openspec)* retarget the calculus change to deep_causality_calculus
- *(deep_causality)* fix rustdoc intra-doc link warnings
- enforce repo-wide `unsafe_code = "forbid"`; remove avoidable unsafe

## [0.6.1](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_physics-v0.6.0...deep_causality_physics-v0.6.1) - 2026-05-29

### Other

- updated the following local packages: deep_causality_core

## [0.6.0](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_physics-v0.5.2...deep_causality_physics-v0.6.0) - 2026-05-26

### Added

- *(physics)* add compressible Newtonian NS regime evaluators (Group 14)
- *(physics)* add Stokes regime evaluator (Group 13)
- *(physics)* add Euler regime evaluator (Group 12)
- *(physics)* add Euler regime evaluator (Group 12)
- *(physics)* add incompressible Newtonian NS regime evaluator (Group 11)
- *(physics)* ideal-flow primitive kernels (group 10)
- *(physics)* boundary-layer kernels (group 9)
- *(physics)* compressible-flow thermodynamic kernels (group 8)
- *(physics)* coherent-structure detector kernels (group 7)
- *(physics)* turbulence quantity kernels (group 6)
- *(physics)* dimensionless number kernels (group 5)
- *(physics)* constitutive viscous-stress kernels (group 4)
- *(physics)* typed vector/tensor surface for fluids + kinematics group
- *(physics)* add-fluid-dynamics-kernels — spec + group 1 scaffolding
- *(deep_causality_topology)* Manifold generic widening over ChainComplex (R4.5)
- *(deep_causality_physics)* flattened unit type folder.
- *(deep_causality_physics)* Generalize MHD grmhd/ideal_induction and Lund fragmentation cluster over R: RealField
- *(deep_causality_physics)* Eliminate residual <f64> turbofish from kernels and wrappers across thermodynamics, waves, photonics, dynamics, nuclear, materials
- *(deep_causality_physics)* Generalize astro / fluids / mhd kernels over R: RealField
- *(deep_causality_physics)* Generalize dynamics hub quantities over R: RealField
- *(deep_causality_physics)* Generalize MaxwellSolver over R: RealField
- *(deep_causality_physics)* Generalize quantum kernels over R: RealField; FloatType alias for examples
- *(deep_causality_multivector, deep_causality_physics)* Generalize HilbertState/HopfState over R: RealField, no f64 defaults
- *(deep_causality_physics)* Generalize nuclear FourMomentum and Hadron over R: RealField
- *(deep_causality_physics)* Generalize condensed scalar leaves over R: RealField
- *(deep_causality_physics)* Generalize Ratio, Time, and Temperature unit types over R: RealField
- *(deep_causality_physics)* Generalize Boltzmann factor and heat capacity kernels over R: RealField
- *(deep_causality_physics)* Generalize Energy unit type over R: RealField
- *(deep_causality_physics)* Generalize nuclear AmountOfSubstance/HalfLife and radioactive_decay over R: RealField
- *(deep_causality_physics)* Generalize dynamics PhysicalVector and torque/angular_momentum kernels over R: RealField
- *(deep_causality_physics)* Generalize fluids Pressure and hydrostatic/Bernoulli kernels over R: RealField
- *(deep_causality_physics)* Generalize relativity spacetime kernels and PhaseAngle/SpacetimeVector over R: RealField
- *(deep_causality_physics)* generalize em fields + PhysicalField over RealField
- *(deep_causality_physics)* generalize mhd plasma and ideal kernels over RealField
- *(deep_causality_physics)* generalize photonics beam + polarization over RealField
- *(deep_causality_physics)* generalize photonics ray optics over RealField
- *(deep_causality_physics)* generalize Probability and dynamics/estimation kernels over RealField
- *(deep_causality_physics)* generalize 18 pure-leaf scalar wrappers over RealField
- *(deep_causality_physics)* generalize materials kernels over RealField
- *(deep_causality_topology)* Implemented specs for generalizing topology crate over RealField
- *(topology)* genericize Manifold over ChainComplex (Stage B)

### Fixed

- *(deep_causality_physics)* validate raw-scalar inputs in 3 fluid kernels
- *(deep_causality_physics, deep_causality_multivector)* Address review feedback on the precision-parametric refactor

### Other

- *(deep_causality_topology)* lazy Hodge ⋆ population via OnceLock
- *(deep_causality_physics)* Updated README file.
- *(deep_causality_physics)* increase coverage of fluid kernels and theory wrappers (+35 tests)
- *(deep_causality_physics)* promote CauchyStress to dedicated ViscousStress / ReynoldsStress newtypes
- *(physics)* act on fluid-dynamics verification-report items
- *(physics)* add reference-solution verification tests for NS regimes
- git commit -m "$(cat <<'EOF'
- Updated example Readme.
- code formatting and linting.
- *(deep_causality_physics)* improve test coverage.
- Minor fixes
- *(deep_causality_physics)* Improve coverage for kernels/*/quantities.rs
- *(deep_causality_physics)* Updated documentation of the physics crate.
- *(deep_causality_physics)* Restructure tests tree to mirror src/kernels/ layout; update Bazel
- Fix clippy lints, panics, and constants-mismatch bias in chronometric example
- Applied minor fixes and lints.
- Add chronometric package for J2-corrected weak-field GM inversion

## [0.5.2](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_physics-v0.5.1...deep_causality_physics-v0.5.2) - 2026-03-12

### Other

- Updated all SBOMS to reflect lates depdency versions.

## [0.5.1](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_physics-v0.5.0...deep_causality_physics-v0.5.1) - 2026-02-09

### Other

- updated all cargo dependencies to the latest version.

## [0.5.0](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_physics-v0.4.0...deep_causality_physics-v0.5.0) - 2026-01-22

### Added

- *(deep_causality_tensor)* Finalized MLX removal.
- *(deep_causality_tensor)* Removed MLX backed.
- *(deep_causality_topology)* Initial implementation of Lattice Gauge Field.

### Other

- *(deep_causality_topology)* Renamed Guage Groups to their definition names.
- *(deep_causality_physics)* Fixed  numerous bugs.
- Updated SBOM of and applied docstring fixes.
- Updated SBOM of recently changed crates.
- you added specification for removing the MLX backend from all affected crates.
- Applied lints and fixes across crates.
- *(deep_causality_topology)* Refeactoring to allow for genric fields and ComplexFiels in GaugeField and LatticeGauge Field
- Merge branch 'deepcausality-rs:main' into main

## [0.4.0](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_physics-v0.3.1...deep_causality_physics-v0.4.0) - 2026-01-09

### Added

- *(deep_causality_topology)* Fixed a number of lints
- *(deep_causality_physics)* Expanded gauge field usage for calculations in all gauge theories. Made ADM ops generic over Field. Updated examples. Added docs.
- *(deep_causality_physics)* Completed integration of new DoubleFloat Type.
- *(deep_causality_topology)* Made all topology types and extensions generic to work with new DoubleFloat Type.
- *(deep_causality_physics)* Updated electro_weak gauge theory and example.
- *(deep_causality_physics)* Renamed qed to electromagnetism. Fixed a number of issues and updated example.
- *(deep_causality_physics)* Updated electro_weak gauge theory and example.
- *(deep_causality_physics)* Updated electro_weak gauge theory and example.
- *(deep_causality_physics)* Polished GR example.
- *(deep_causality_physics)* Completed Gauge based GR theory and added an example.
- *(deep_causality_physics)* Updated and improved Gauge based GR theory.
- *(deep_causality_physics)* Implemented Gauge GR theory.
- *(deep_causality_physics)* Immproved Electroweak implementation.
- *(deep_causality_physics)* Immproved QED implementation.
- *(deep_causality_topology)* Updated and improved Gauge field impl and tests.
- *(deep_causality_physics)* Added Weak and Electroweak theories with tests and examples.
- *(deep_causality_physics)* Implemented QED theory.
- *(deep_causality_physics)* Implemented kerneles required for implementing subsequent gauge theories.

### Fixed

- *(deep_causality_physics)* Fixed some name spaces issues.

### Other

- *(deep_causality_physics)* increased test coverage.
- *(deep_causality_physics)* increased test coverage.
- *(deep_causality_physics)* increased test coverage.
- Updated Bazel build and test config.
- *(deep_causality_physics)* increased test coverage.
- updated project wide SBOM files.
- updated project wide copyright note.
- Addes specs for gauge fiels. Prepared implementation.
- Removed unused feature flag.

## [0.3.1](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_physics-v0.3.0...deep_causality_physics-v0.3.1) - 2025-12-31

### Other

- Updated SBOMs to trigger release.

## [0.2.0](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_physics-v0.1.1...deep_causality_physics-v0.2.0) - 2025-12-18

### Fixed

- *(deep_causality_physics)* Updated PhysicsError impl and usage.
- *(deep_causality_physics)* Fixed a number of physics related bugs. Updated tests for verification.

### Other

- *(deep_causality_physics)* Increased test coverage.
- *(deep_causality_physics)* Increased test coverage.

## [0.1.1](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_physics-v0.1.0...deep_causality_physics-v0.1.1) - 2025-12-14

### Added

- *(deep_causality_physics)* Added more physics examples.
- *(deep_causality_physics)* Updated GRMHD example.
- *(deep_causality_physics)* Implemented and tested mhd module.
- *(deep_causality_physics)* Implemented and tested photonics module.
- *(deep_causality_physics)* Updated README.md
- *(deep_causality_physics)* Implemented initial condensed matter physics support.

### Fixed

- *(deep_causality_physics)* Updated Bazel config.

### Other

- *(deep_causality)* Increased test coverage.
- *(deep_causality)* Code linting and fixes.
- *(deep_causality_physics)* Increased test coverage.
- *(deep_causality_physics)* Increased test coverage.
