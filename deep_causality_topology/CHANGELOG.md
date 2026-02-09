# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.0](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_topology-v0.4.0...deep_causality_topology-v0.5.0) - 2026-02-09

### Other

- updated all cargo dependencies to the latest version.
- Fixed more lints.
- *(deep_causality_topology)* Added Stry_from_phase to gauge link.
- *(deep_causality_topology)* Added Source of the Field to the Lattice Gauge Field.
- *(deep_causality_num)* Added is_nan, is_infinite, and is_finite to Float106 type

## [0.4.0](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_topology-v0.3.0...deep_causality_topology-v0.4.0) - 2026-01-22

### Added

- *(deep_causality_topology)* Added SE3 Gauge Group of rigid body motions. .
- *(deep_causality_tensor)* Finalized MLX removal.
- *(deep_causality_tensor)* Removed MLX backed.
- *(deep_causality_topology)* Removed MLX backed.
- *(deep_causality_multivector)* Removed MLX backed.
- *(deep_causality_topology)* Updated and revised implementation of Lattice Gauge Field.
- *(deep_causality_topology)* Updated and revised implementation of Lattice Gauge Field.
- *(deep_causality_topology)* Initial implementation of Lattice Gauge Field.

### Fixed

- *(deep_causality_topology)* Applied lints and fixes.

### Other

- *(deep_causality_topology)* Renamed Guage Groups to their definition names.
- *(deep_causality_num)* Renamed DoubleFloat to Float106 for consistency with existing float types.
- *(deep_causality_topology)* Fixed  numerous bugs.
- remoced unneccessary trait bounds.
- Updated SBOM of and applied docstring fixes.
- Updated SBOM of recently changed crates.
- Updated Bazel config
- Applied lints and fixes across crates.
- *(deep_causality_topology)* Applied lints and fixes.
- *(deep_causality_topology)* Refeactoring to allow for genric fields and ComplexFiels in GaugeField and LatticeGauge Field
- *(deep_causality_topology)* Initial verification of the Lattice Gauge Field.
- *(deep_causality_topology)* Applied lints and fixes.
- *(deep_causality_topology)* Applied lints and fixes.
- *(deep_causality_topology)* Applied lints and fixes.
- *(deep_causality_topology)* Increased test coverage, applied lints and fixes.
- *(deep_causality_topology)* Increased test coverage, applied lints and fixes.
- *(deep_causality_topology)* Increasded test coverage, applied lints and fixes.
- *(deep_causality_topology)* Increasded test coverage, applied lints and fixes.
- *(deep_causality_topology)* Increasded test coverage.
- Merge branch 'deepcausality-rs:main' into main
- *(deep_causality_topology)* Updated SBOM
- *(deep_causality_topology)* Added exaple for Lattice Gauge Field impl.
- *(deep_causality_topology)* Documented  and revised Lattice Gauge Field impl.
- *(deep_causality_topology)* Reviewed initial implementation of Lattice Gauge Field.
- *(deep_causality_topology)* Tested initial implementation of Lattice Gauge Field.

## [0.3.0](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_topology-v0.2.5...deep_causality_topology-v0.3.0) - 2026-01-09

### Added

- *(deep_causality_topology)* Fixed a number of lints
- *(deep_causality_topology)* Fixed MLX gated impl to use new manifold generics.
- *(deep_causality_physics)* Expanded gauge field usage for calculations in all gauge theories. Made ADM ops generic over Field. Updated examples. Added docs.
- *(deep_causality_physics)* Completed integration of new DoubleFloat Type.
- *(deep_causality_topology)* Fixed some tests.
- *(deep_causality_topology)* Made all topology types and extensions generic to work with new DoubleFloat Type.
- *(deep_causality_physics)* Updated and improved Gauge based GR theory.
- *(deep_causality_physics)* Implemented Gauge GR theory.
- *(deep_causality_physics)* Immproved Electroweak implementation.
- *(deep_causality_physics)* Immproved QED implementation.
- *(deep_causality_topology)* Restructed hkt modules for better code organization.
- *(deep_causality_physics)* Implemented kerneles required for implementing subsequent gauge theories.
- *(deep_causality_topology)* Finalized and reviewed Gauge Field and related types.
- *(deep_causality_topology)* Implemented HKT traits for Gauge Field and related types.
- *(deep_causality_topology)* Completed implementation. Added tests.
- *(deep_causality_topology)* Initial implementation of Gauge Field.
- *(deep_causality_topology)* Removed unused dependency.
- *(deep_causality_multivector)* Migrated to dedicted pure HKT trait.
- *(deep_causality_topology)* Completed transition to GAT based HKT.

### Other

- *(deep_causality_topology)* Fixed some tests.
- *(deep_causality_physics)* increased test coverage.
- *(deep_causality_topology)* increased test coverage.
- Updated Bazel build and test config.
- repo wide lints and fixes
- repo wide lints and formatting.
- *(deep_causality_num)* increased test coverage.
- updated project wide SBOM files.
- updated project wide copyright note.
- Addes specs for gauge fiels. Prepared implementation.
- Migrated to dedicted pure HKT trait.

## [0.2.5](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_topology-v0.2.4...deep_causality_topology-v0.2.5) - 2025-12-31

### Other

- updated the following local packages: deep_causality_multivector

## [0.2.4](https://github.com/marvin-hansen/deep_causality/compare/deep_causality_topology-v0.2.3...deep_causality_topology-v0.2.4) - 2025-12-31

### Added

- *(deep_causality_multivector)* separated MLX code into dedicted files for better maintainabiliy.
- *(deep_causality_multivector)* Added algebraic trait impl for MultiField.
- *(deep_causality_topology)* Updated benchmarks.
- *(deep_causality_topology)* Added new benchmark for multi backend support.
- *(deep_causality_topology)* Added new lattice types and initial multi backend supporte.
- *(deep_causality_tensor)* Fixed tests for new backend agnostic CPU impl.
- *(deep_causality_topology)* Imlemented initial MLX acceleration. Closes #432
- *(deep_causality_metric)* Integrated new metric crate across the repo.

### Fixed

- *(deep_causality_topology)* Minor fixes and lints.

### Other

- Lots of lints, formatting, and minor fixes.
- Lots of lints, formatting, and minor fixes.
- Minor lints.

## [0.2.3](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_topology-v0.2.2...deep_causality_topology-v0.2.3) - 2025-12-18

### Fixed

- *(deep_causality_topology)* Fixed a number of topology related bugs. Updated tests for verification.

## [0.2.2](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_topology-v0.2.1...deep_causality_topology-v0.2.2) - 2025-12-14

### Other

- *(deep_causality_topology)* Increased test coverage.

## [0.2.1](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_topology-v0.2.0...deep_causality_topology-v0.2.1) - 2025-12-12

### Other

- updated the following local packages: deep_causality_multivector

## [0.2.0](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_topology-v0.1.0...deep_causality_topology-v0.2.0) - 2025-12-12

### Added

- *(deep_causality_physics)* Added generalized_master_equation_kernel to dynamic module.
- *(deep_causality_topology)* Added Regge Calculus Curvature and tests. Improved error handling by updating the  TopologyError.  Added SimplicialComplexBuilder for constructing correct SimplicialComplex.
- *(deep_causality_topology)* Removed non-std config. Updated Bazel config.

### Fixed

- *(deep_causality_physics)* Fixed a subliminal bug around Laplacian and  Boundary Operator Indexing.

### Other

- Added a lot more examples across physics and medicine.
- Added two new physics exmples.
- *(deep_causality_topology)* Improved test coverage.
- *(deep_causality_topology)* Improved test coverage.
- *(deep_causality_topology)* Improved test coverage.
- *(deep_causality_topology)* Improved test coverage.

## [0.1.0](https://github.com/marvin-hansen/deep_causality/releases/tag/deep_causality_topology-v0.1.0) - 2025-12-03

### Added

- *(deep_causality_multivector)* Ported CausalMultiVector to use Field instead of the broader Num trait to ensure correct math.
- *(deep_causality_topology)* Added examples and README.md
- *(deep_causality_topology)* refactored for better code organization.
- *(deep_causality_topology)* refactored for performance improvements and better code organization.
- *(deep_causality_topology)* Added test utils
- *(deep_causality_topology)* Improved Manifold impl.
- *(deep_causality_topology)* Added Graph topology with HKT types.
- *(deep_causality_topology)* Added Hypergraph topology with HKT types.
- *(deep_causality_topology)* Added HKT and functional trait impl for Manifold topology.
- *(deep_causality_topology)* Added Manifold topology.
- *(deep_causality_topology)* Added PointCloud topology. Added TopologyError. Added Topology Trait hierarchy.
- *(deep_causality_topology)* Initial implementation of topology data structures.

### Other

- Regenerated SBOM.
- *(deep_causality_topology)* Fixed discrete differential geometry examples.
- *(deep_causality_topology)* Fixed discrete differential geometry on simplicial complexes. Fixed Cup product on topology.
- *(deep_causality_topology)* Moved to core traits instead of std for better protability.
- *(deep_causality_topology)* Added examples and debugging discrete differential geometry.
- *(deep_causality_topology)* Added discrete differential geometry on simplicial complexes:Added new tests.
- *(deep_causality_topology)* implemented algebraic structure for Chain<T> in deep_causality_topology. Added new tests.
- *(deep_causality_topology)* Added SBOM.
- *(deep_causality_topology)* Increased test coverage.
- *(deep_causality_topology)* Added test coverage.
