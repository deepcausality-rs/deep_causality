# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.1](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_uncertain-v0.3.0...deep_causality_uncertain-v0.3.1) - 2025-10-02

### Added

- *(deep_causality_discovery)* working on data cleaning step during CDL

### Other

- *(deep_causality_uncertain)* removed flaky test assertion.

## [0.3.0](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_uncertain-v0.2.3...deep_causality_uncertain-v0.3.0) - 2025-09-25

### Added

- *(deep_causality_uncertain)* Updated Readme and examples.
- *(deep_causality_uncertain)* introduce MaybeUncertain type for probabilistic presence

### Other

- Updated SBOM for all crates.
- Minor lints and fixes. Trying cargo cache on build and test.
- Updated SBOM for all crates.

## [0.2.3](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_uncertain-v0.2.2...deep_causality_uncertain-v0.2.3) - 2025-09-22

### Fixed

- *(deep_causality)* Removed last internal macros and removed dependency on deep_causality_macro crate.

### Other

- Updated SBOM script to generate hash signature together with the SBOM.

## [0.2.2](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_uncertain-v0.2.1...deep_causality_uncertain-v0.2.2) - 2025-09-21

### Added

- *(deep_causality_num)* Initial implementation. Update of all downstream crates.
- *(deep_causality_rand)* Fixed issue with std pseudo number generator. Updated upstream crate deep_causality_uncertain
- *(deep_causality_uncertain)* Migrated random number generation to deep_causality_rand crate

### Fixed

- *(deep_causality_uncertain)* Minor test fix.
- *(deep_causality_uncertain)* Minor linting and formatting

### Other

- Updated Cargo configuration and feature propagation across the entire repo.
- Reworked and updated Bazel test config across all crates.
- Lots of lints and formats. Updated MSRV to 1.89 and edition 2024 across the entire repo.
- *(deep_causality_rand)* Increased test coverage.
- Added or updated Bazel config for the newly added crates.
- Updated build scripts

## [0.2.1](https://github.com/marvin-hansen/deep_causality/compare/deep_causality_uncertain-v0.2.0...deep_causality_uncertain-v0.2.1) - 2025-09-11

### Other

- trying to fix #315
- trying to fix #315
- trying to fix #315
- trying to fix #315
- trying to fix #315
- trying to fix #315
- trying to fix #315
- Fixed dependencies in the Uncertain crate to resolve #315
- Fixed global Cargo.toml to resolve #315
- Fixed dependencies in the Uncertain crate to resolve #315
- Updated project wide Bazel config, removed Bazel aliases, and updated project wide macro imports from the deep_causality_macro crate.

## [0.2.0](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_uncertain-v0.1.0...deep_causality_uncertain-v0.2.0) - 2025-09-08

### Added

- *(deep_causality_uncertain)* Increased test coverage.
- *(deep_causality_uncertain)* Increased test coverage.
- *(deep_causality_uncertain)* Improved uncertain_f64 with estimate_probability_exceeds and uncertain_bool with to_bool
- *(deep_causality_uncertain)* implemented PartEq for ComputationalNode. Added identity checks to ComputationalNode. Re-organized code and imports.
- *(deep_causality)* Add uncertain data types and tests to DeepCausality
- *(deep_causality_uncertain)* Implemented Debug for Uncertain and ComputationNode type.

### Fixed

- *(deep_causality_uncertain)* Fixed flaky test

### Other

- Increased test coverage across all crates.
- Increased test coverage across all crates.
- Updated project wide Bazel config.
- Merge remote-tracking branch 'origin/main'
# Changelog
