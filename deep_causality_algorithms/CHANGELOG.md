# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.4](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_algorithms-v0.1.3...deep_causality_algorithms-v0.1.4) - 2025-09-25

### Other

- Updated SBOM for all crates.

## [0.1.2](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_algorithms-v0.1.1...deep_causality_algorithms-v0.1.2) - 2025-09-22

### Fixed

- *(deep_causality)* Removed last internal macros and removed dependency on deep_causality_macro crate.

### Other

- Updated SBOM script to generate hash signature together with the SBOM.

## [0.1.1](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_algorithms-v0.1.0...deep_causality_algorithms-v0.1.1) - 2025-09-21

### Added

- *(deep_causality_tensor)* Initial setup. Moved CausalTensor type from the data_structure crate into dedicated deep_causality_tensor crate.
- *(deep_causality_num)* Initial implementation. Update of all downstream crates.
- *(deep_causality_algorithms)* Implement mRMR (FCQ variant) feature selector

### Fixed

- *(deep_causality_algorithms)* Fixed Bazel build.

### Other

- Updated Cargo configuration and feature propagation across the entire repo.
- Reworked and updated Bazel test config across all crates.
- Merge branch '002-replace-rng-currently'
- Lots of lints and formats. Updated MSRV to 1.89 and edition 2024 across the entire repo.
- *(deep_causality_rand)* Increased test coverage.

## [0.1.0](https://github.com/marvin-hansen/deep_causality/releases/tag/deep_causality_algorithms-v0.1.0) - 2025-09-15

### Added

- *(deep_causality_data_structures)* bumped up version number
- *(deep_causality_algorithms)* Updated Bazel build and test config.
- *(deep_causality_data_structures)* Added identity types. Removed num_traits dependency.
- *(deep_causality_algorithms)* Added a README.md and LICENSE file.
- *(deep_causality_algorithms)* Added a README.md to the example_surd in the example folder.
- *(deep_causality_algorithms)* Implemented SURD-State algorithm described in the paper "Observational causality by states and interaction type for scientific discovery"
- *(deep_causality_algorithms)* Improved  SURD-Stat algorithms with separation of causal and non-causal state dependent maps. Improved documentation of the algo.
- *(deep_causality_algorithms)* Initial implementation of the SURD-State algorithms.

### Fixed

- *(deep_causality_algorithms)* Fixed multiple division by zero issue and replaced unsafe tensor division with save_div.

### Other

- Restored version number...
- Removed version number form internal deps.
- *(deep_causality_algorithms)* linting and formatting
- *(deep_causality_data_structures)* Improved test coverage.
- *(deep_causality_algorithms)* Improved test coverage.
- *(deep_causality_data_structures)* Improved test coverage.
- *(deep_causality_algorithms)* Improved test coverage.
- *(deep_causality_algorithms)* Improved test coverage.
- Code formating and linting across the repo. Moved old and empty dcl crate into yanked folder.
- *(deep_causality_data_structures)* Increased test coverage for CausalTensorMathExt
- *(deep_causality_algorithms)* Added test coverage for SURD algo.
- *(deep_causality_algorithms)* Added test coverage for SURD algo.
