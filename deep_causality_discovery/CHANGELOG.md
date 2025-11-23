# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.4](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_discovery-v0.2.3...deep_causality_discovery-v0.2.4) - 2025-11-23

### Other

- Updated CDL HKT pre-specs

## [0.2.3](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_discovery-v0.2.2...deep_causality_discovery-v0.2.3) - 2025-11-05

### Other

- Updated SBOM for all crates.

## [0.2.2](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_discovery-v0.2.1...deep_causality_discovery-v0.2.2) - 2025-10-31

### Other

- Updated parquet to latest version in  deep_causality_discovery

## [0.2.1](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_discovery-v0.2.0...deep_causality_discovery-v0.2.1) - 2025-10-19

### Other

- Updated Cargo deps.

## [0.2.0](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_discovery-v0.1.1...deep_causality_discovery-v0.2.0) - 2025-10-02

### Added

- *(deep_causality_algorithms)* Generic MRMR
- *(deep_causality_algorithms)* Added mrmr score to feature selection. documented in specs/006-mrmr-feature-score.
- *(deep_causality_discovery)* working on data cleaning step during CDL

### Fixed

- *(deep_causality_discovery)* refactored CDL module for improved modularity
- *(deep_causality_algorithms)* renamed mrmr algo

### Other

- *(deep_causality_algorithms)* increased test coverage.
- *(deep_causality_discovery)* Added or updates test coverage for DataCleaner and affected types.
- Preparing DataCleaning stage in CDL.

## [0.1.1](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_discovery-v0.1.0...deep_causality_discovery-v0.1.1) - 2025-09-25

### Fixed

- *(deep_causality_discovery)* bumped up versions of external deps.

### Other

- Updated SBOM for all crates.
- Added Bazel config for deep_causality_discovery.

## [0.1.0](https://github.com/marvin-hansen/deep_causality/releases/tag/deep_causality_discovery-v0.1.0) - 2025-09-23

### Added

- *(deep_causality_discovery)* Added LICENSE
- *(deep_causality_discovery)* Added README.md
- *(deep_causality_algorithms)* Added data pre-processor to the CDL pipeline
- *(deep_causality_discovery)* Updated tests and added example.
- *(deep_causality_discovery)* Updated CdlError
- *(deep_causality_discovery)* Added initial implementation of the CDL causal discovery DSL.

### Fixed

- *(deep_causality_discovery)* Minor lints.

### Other

- Updated SBOM for all crates.
- *(deep_causality_discovery)* Improved test coverage.
- *(deep_causality_discovery)* Improved test coverage.
- *(deep_causality_discovery)* Improved test coverage for types.
- *(deep_causality_discovery)* Improved test coverage for types.
- *(deep_causality_discovery)* Improved test coverage for types.
- *(deep_causality_discovery)* Improved test coverage for types.
- test(deep_causality_discovery: Increased test coverage.
- *(deep_causality_algorithms)* Added tests for error types
- *(deep_causality_algorithms)* Improved documentation of all types and process steps.
- *(deep_causality_algorithms)* Improved documentation of all traits
