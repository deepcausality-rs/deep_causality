# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/deepcausality-rs/deep_causality/releases/tag/deep_causality_algorithms-v0.1.0) - 2025-09-15

### Added

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
