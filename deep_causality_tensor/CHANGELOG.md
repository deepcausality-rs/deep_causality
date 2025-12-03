# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.10](https://github.com/marvin-hansen/deep_causality/compare/deep_causality_tensor-v0.1.9...deep_causality_tensor-v0.1.10) - 2025-12-03

### Added

- *(deep_causality_sparse)* Fixing auto-release
- *(deep_causality_tensor)* Added full set of new algebraic trait system to CausalTensor.
- *(deep_causality_topology)* Initial implementation of topology data structures.

### Other

- Regenerated SBOM.
- Updated dev dependencies across the repo.
- *(deep_causality_topology)* Fixed discrete differential geometry examples.
- *(deep_causality_tensor)* Added implementation of BoundedAdjunction trait as type extension.
- *(deep_causality_haft)* Added BoundedAdjunction trait.
- Merge branch 'deepcausality-rs:main' into main
- Merge remote-tracking branch 'origin/main'
- Restored manually generated SBOM to restore Dependency and licence scan.

## [0.1.9](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_tensor-v0.1.8...deep_causality_tensor-v0.1.9) - 2025-11-23

### Other

- Merge branch 'deepcausality-rs:main' into main

### Removed

- removed all manually generated SBOM files

## [0.1.8](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_tensor-v0.1.7...deep_causality_tensor-v0.1.8) - 2025-11-23

### Added

- *(deep_causality_multivector)* Added General Relativistic Magnetohydrodynamics example
- *(deep_causality_tensor)* refactored public Tensor API as a trait.
- *(deep_causality_multivector)* Added BoundedComonad impl as type extension.
- *(deep_causality_tensor)* Implemented CoMonad for Causaltensor
- *(deep_causality_haft)* Introduced Default trait bound to MonadEffect types to simplify error case handling. Update downstream deps.
- *(deep_causality_tensor)* Reformatted ein_sum example.

### Other

- Merge branch 'deepcausality-rs:main' into main

## [0.1.7](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_tensor-v0.1.6...deep_causality_tensor-v0.1.7) - 2025-11-05

### Added

- *(deep_causality_tensor)* Updated EinSumOp for better ergonomics.
- *(deep_causality_tensor)* Added From impl for value and reference to CausalTensor.
- *(deep_causality_tensor)* Refactored code organization and improved documentation of public API.
- *(deep_causality_tensor)* Refactored code organization
- Optimize N-dimensional tensor trace calculation
- *(deep_causality_tensor)* Updated and tested  implementation for Einstein Sum Convention.
- *(deep_causality_tensor)* Added initial implementation for Einstein Sum Convention.

### Other

- *(deep_causality_ast)* Improved test organization.
- *(deep_causality_tensor)* Improved test coverage and test organization.
- The mat_mul_2d function has been updated to use direct array access with
- *(deep_causality_tensor)* Improved test coverage of implementation for Einstein Sum Convention.
- linting and formatting.
- Optimize 2D tensor trace calculation
- Added new test cases to deep_causality_tensor/src/types/a
- *(deep_causality_tensor)* Added example code and updated Readme for  Einstein Sum.
- Updated Bazel config
- *(deep_causality_tensor)* Documented implementation for Einstein Sum Convention.
- Updated SBOM for all crates.

## [0.1.6](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_tensor-v0.1.5...deep_causality_tensor-v0.1.6) - 2025-10-31

### Added

- *(deep_causality_tensor)* Moved TensorProduct implementation from an extension into the CausalTensor  type.
- *(deep_causality_tensor)* Added TensorProduct
- *(deep_causality)* Added MaybeUncertain and CausalTensor to PropagatingEffect. Updated tests.
- *(deep_causality_tensor)* Added Higher Kinded Type extension.

### Other

- *(deep_causality_tensor)* Added tests for TensorProduct
- *(deep_causality_tensor)* minor lints.
- *(deep_causality_tensor)* lints, fixes, and more tests.

## [0.1.5](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_tensor-v0.1.4...deep_causality_tensor-v0.1.5) - 2025-10-02

### Added

- *(deep_causality_tensor)* Made CausalTensor no-copy / clone for broader usage with complex data ttpes i.e. Uncertain<T>.

## [0.1.4](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_tensor-v0.1.3...deep_causality_tensor-v0.1.4) - 2025-09-25

### Other

- Updated SBOM for all crates.

## [0.1.2](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_tensor-v0.1.1...deep_causality_tensor-v0.1.2) - 2025-09-22

### Fixed

- *(deep_causality)* Removed last internal macros and removed dependency on deep_causality_macro crate.

### Other

- Updated SBOM script to generate hash signature together with the SBOM.

## [0.1.1](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_tensor-v0.1.0...deep_causality_tensor-v0.1.1) - 2025-09-21

### Other

- Updated Cargo configuration and feature propagation across the entire repo.
