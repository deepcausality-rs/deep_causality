# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.4](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_haft-v0.2.3...deep_causality_haft-v0.2.4) - 2025-11-23

### Other

- Merge branch 'deepcausality-rs:main' into main

### Removed

- removed all manually generated SBOM files

## [0.2.3](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_haft-v0.2.2...deep_causality_haft-v0.2.3) - 2025-11-23

### Added

- *(deep_causality_haft)* Added BoundedComonad for usage with MultiVector.
- *(deep_causality_tensor)* Implemented CoMonad for Causaltensor
- *(deep_causality_haft)* Added CoMonad and Traversable trait for HKT. Added default implementations for some std types. Added examples, tests and documentation.
- *(deep_causality_haft)* Introduced Default trait bound to MonadEffect types to simplify error case handling. Update downstream deps.

### Fixed

- *(deep_causality_haft)* Fixed trait bound inconsistency in default impl for Option and Result.

### Other

- Updated Bazael config
- Merge branch 'deepcausality-rs:main' into main

## [0.2.2](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_haft-v0.2.1...deep_causality_haft-v0.2.2) - 2025-11-05

### Other

- Updated SBOM for all crates.

## [0.2.1](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_haft-v0.2.0...deep_causality_haft-v0.2.1) - 2025-10-31

### Added

- *(deep_causality_tensor)* Added Higher Kinded Type extension.
- *(deep_causality_haft)* Added more default witness types and updated example code.
- *(deep_causality_haft)* Added more default witness types and updated example code.
- *(deep_causality_haft)* Added SBOM and signature

### Other

- *(deep_causality_haft)* Updated README.md

## [0.2.0](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_haft-v0.1.0...deep_causality_haft-v0.2.0) - 2025-10-19

### Added

- *(deep_causality_haft)* Updated Bazel config.
- *(deep_causality_haft)* Added new example and updated README.md
- *(deep_causality_haft)* Added Foldable trait with tests.
- *(deep_causality_haft)* Added Applicative trait with tests.
- *(deep_causality_haft)* Added HKT Trait for Arity 5 together with Monad and MonadEffect for HKT5. Refactored code based. Added HKT type extension for Vec.
- *(deep_causality_haft)* Added HKT Trait for Arity 4 together with Monad and MonadEffect for HKT4

### Other

- Merge remote-tracking branch 'origin/main'
- *(deep_causality_haft)* Added extensive documentation.
- *(deep_causality_haft)* Increased test coverage.
