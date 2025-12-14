# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
