# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
