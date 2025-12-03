# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.3](https://github.com/marvin-hansen/deep_causality/compare/deep_causality_multivector-v0.1.2...deep_causality_multivector-v0.1.3) - 2025-12-03

### Added

- *(deep_causality_sparse)* Fixing auto-release
- *(deep_causality_num)* Increased test coverage.
- *(deep_causality_sparse)* Added full set of new algebraic trait system to CsrMatrix.
- *(deep_causality_tensor)* Added full set of new algebraic trait system to CausalTensor.
- *(deep_causality_multivector)* Minor code re-organization.
- *(deep_causality_num)* Added market traits for Associative, Distributive, and Commutative. Updated algebra traits accordingly and fixed downstream.
- *(deep_causality_multivector)* Added full set of new algebraic trait system to CausalMultiVector.
- *(deep_causality_multivector)* Ported CausalMultiVector to use Field instead of the broader Num trait to ensure correct math.
- *(deep_causality_num)* Update all tests for Complex Number type with proper algebraic traits.
- *(deep_causality_topology)* Initial implementation of topology data structures.

### Other

- Regenerated SBOM.
- *(deep_causality_multivector)* Improved test coverage
- Updated dev dependencies across the repo.
- Updated Dev dependencies.
- *(deep_causality_topology)* Added test coverage.
- *(deep_causality_multivector)* Added implementation of BoundedAdjunction trait as type extension.
- *(deep_causality_haft)* Added BoundedAdjunction trait.
- Merge branch 'deepcausality-rs:main' into main
- Merge remote-tracking branch 'origin/main'
- Restored manually generated SBOM to restore Dependency and licence scan.

## [0.1.2](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_multivector-v0.1.1...deep_causality_multivector-v0.1.2) - 2025-11-23

### Other

- Merge branch 'deepcausality-rs:main' into main

### Removed

- removed all manually generated SBOM files

## [0.1.1](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_multivector-v0.1.0...deep_causality_multivector-v0.1.1) - 2025-11-23

### Added

- *(deep_causality_multivector)* Minor lints.
- *(deep_causality_multivector)* Added General Relativistic Magnetohydrodynamics example
- *(deep_causality_multivector)* Added General Relativistic Magnetohydrodynamics example
- *(deep_causality_multivector)* Updated README.md
- *(deep_causality_multivector)* Added new example.
- *(deep_causality_multivector)* Added new example.
- *(deep_causality_multivector)* Updated README.md
- *(deep_causality_multivector)* Added benchmark for quantum ops
- *(deep_causality_multivector)* Added quantum example via Hilbert State
- *(deep_causality_multivector)* Minor lints
- *(deep_causality_multivector)* Added quantum gate and operation extension.
- *(deep_causality_multivector)* Added commutator_lie and commutator_geometric operations to API.
- *(deep_causality_multivector)* refactored core API as a trait.
- *(deep_causality_multivector)* Added BoundedComonad impl as type extension.
- *(deep_causality_multivector)* Added SBOM to multivector crate.

### Other

- *(deep_causality_multivector)* improved test coverage.
- *(deep_causality_multivector)* improved test coverage.
- *(deep_causality_multivector)* improved performance of geometric product on for sparse and dense cases.
- *(deep_causality_multivector)* Fixed incorrect basis_shift.
- *(deep_causality_multivector)* Fixed incorrect extend impl and fixed some tests.
- *(deep_causality_num)* Added test coverage for Octonion numbers.
- *(deep_causality_multivector)* Increased test coverage
- *(deep_causality_multivector)* Fixed tests.
- *(deep_causality_multivector)* Improved test coverage and added minor lints and fixes.
- *(deep_causality_multivector)* Fixed some complex algebras, added Euclidean algebra, added constructor aliases, and split algebras.
