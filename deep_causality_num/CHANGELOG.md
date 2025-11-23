# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.7](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_num-v0.1.6...deep_causality_num-v0.1.7) - 2025-11-23

### Added

- *(deep_causality_multivector)* Added quantum gate and operation extension.
- *(deep_causality_num)* Added documentation to Octonion implementation.
- *(deep_causality_num)* Added Octonion implementation.
- *(deep_causality_num)* Fixed doct tests
- *(deep_causality_num)* Reorganized internal source code.

### Other

- *(deep_causality_num)* Minor lint
- *(deep_causality_num)* Minor lint
- *(deep_causality_num)* Added test coverage for Octonion numbers.
- *(deep_causality_num)* Added test utils for Octonion testing

## [0.1.6](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_num-v0.1.5...deep_causality_num-v0.1.6) - 2025-11-05

### Added

- *(deep_causality_num)* This change introduces a new Quaternion type accompanied by a comprehensive suite.

### Fixed

- *(deep_causality_num)* Minor fixes and lints.
- *(deep_causality_num)* The Display implementation for Quaternion has been refined to correctly handle the signs of its components, producing a more standard and readable mathematical format. All tests pass.
- *(deep_causality_num)* The slerp implementation has been corrected by removing the incorrect special case for antipodal quaternions and adding a check for nearly identical quaternions to use linear interpolation, preventing division by
- *(deep_causality_num)* The scalar division implementation has been refactored to align with Rust's standard floating-point division-by-zero behavior, which produces
- *(quaternion)* The scalar division implementation has been refactored to align with Rust's standard floating-point division-by-zero behavior, which produces

### Other

- Updated Bazel config. Applied minor lint.
- *(quaternion)* Added docstring to all public API methods.
- Updated README
- *(quaternion)* Address test inaccuracies and revert slerp regression. Added rotation methods and tests.
- Updated SBOM for all crates.

## [0.1.5](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_num-v0.1.4...deep_causality_num-v0.1.5) - 2025-10-31

### Added

- *(deep_causality)* Added ComplexTensor to PropagatingEffect.
- *(deep_causality_num)* Introduce Complex<F> type and ComplexNumber trait.

### Other

- *(deep_causality_num)* increased test coverage.
- *(deep_causality_num)* Added Numerically stable principal sqrt
- *(deep_causality_num)* minor fixes.
- *(deep_causality_num)* increased test coverage.
- *(deep_causality_num)* increased test coverage.
- *(deep_causality_num)* increased test coverage.

## [0.1.3](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_num-v0.1.2...deep_causality_num-v0.1.3) - 2025-09-25

### Other

- Updated SBOM for all crates.
- Updated SBOM for all crates.

## [0.1.2](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_num-v0.1.1...deep_causality_num-v0.1.2) - 2025-09-22

### Fixed

- *(deep_causality)* Removed last internal macros and removed dependency on deep_causality_macro crate.

### Other

- Updated SBOM script to generate hash signature together with the SBOM.

## [0.1.1](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_num-v0.1.0...deep_causality_num-v0.1.1) - 2025-09-21

### Other

- Updated Cargo configuration and feature propagation across the entire repo.
