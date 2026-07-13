# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.1](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_discovery-v0.5.0...deep_causality_discovery-v0.5.1) - 2026-07-13

### Added

- *(deep_causality_haft)* add Category + Kleisli (named category, compose = bind) — H2

## [0.5.0](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_discovery-v0.4.1...deep_causality_discovery-v0.5.0) - 2026-07-08

### Added

- *(deep_causality_discovery)* learn-once, rank-many CPDAG cache for BRCD

### Fixed

- *(deep_causality_discovery)* version-tag CPDAG cache key; correct Precision doc
- *(brcd,discovery)* address QA findings (32-bit shift, DRY, Precision bound)
- fixed sone doctest warnings
- *(deep_causality_discovery)* fixed bazel test config.

### Other

- *(num)* split deep_causality_num into num-core + algebra + complex + dual
- *(bazel)* register all missing test suites; add Dual Default; move iso test utils to src/utils_tests
- *(deep_causality_algorithms)* parallelize BRCD across candidates; add BRCD eval harnesses + companion papers
- raise test coverage across 8 crates.
- Generated new SBOM for all crates.
- Merge branch 'deepcausality-rs:main' into main
- Updated README file across multiple crates to meet project standard.

## [0.4.1](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_discovery-v0.4.0...deep_causality_discovery-v0.4.1) - 2026-06-12

### Other

- Merge branch 'deepcausality-rs:main' into main
- *(deps)* Bump parquet from 58.3.0 to 59.0.0

## [0.4.0](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_discovery-v0.3.7...deep_causality_discovery-v0.4.0) - 2026-06-09

### Fixed

- fix(deep_causality_discovery):
- *(deep_causality_discovery,deep_causality_tensor)* make f64→T casts honest; sync docs to generic T

### Other

- *(website)* Added a new blog post that introduces the new CDL;
- *(deep_causality_discovery)* Improved test coverage.
- *(deep_causality_discovery)* Updated README and reference documentation
- *(deep_causality_discovery)* dual-algorithm CDL (SURD + BRCD) with config-builder surface
- *(deep_causality)* fix rustdoc intra-doc link warnings
- *(deep_causality_discovery)* split cdl_effect into focused modules; extract CdlBuilder
- *(discovery)* generify the CDL pipeline over RealField precision
- disable Miri isolation for deep_causality_core; gate discovery MRMR test
- enforce repo-wide `unsafe_code = "forbid"`; remove avoidable unsafe

## [0.3.7](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_discovery-v0.3.6...deep_causality_discovery-v0.3.7) - 2026-05-26

### Other

- Updated example Readme.
- *(examples)* consolidate algorithms/discovery examples into causal_discovery_examples
- Updated the example Readme.

## [0.3.6](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_discovery-v0.3.5...deep_causality_discovery-v0.3.6) - 2026-05-03

### Other

- Applied minor lints.

## [0.3.5](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_discovery-v0.3.4...deep_causality_discovery-v0.3.5) - 2026-03-12

### Other

- Updated all SBOMS to reflect lates depdency versions.
- Updated all Cargo deps to latest version.

## [0.3.4](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_discovery-v0.3.3...deep_causality_discovery-v0.3.4) - 2026-01-22

### Other

- *(deep_causality_discovery)* Fixed numerous bugs.
- Updated SBOM of and applied docstring fixes.

## [0.3.3](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_discovery-v0.3.2...deep_causality_discovery-v0.3.3) - 2026-01-09

### Added

- *(deep_causality_num)* Fixed downstrem type annotation issues.
- *(deep_causality_sparse)* Finalized HKT extension to use new GAT bounded HKT.

### Other

- Updated depenencies and vendored dependencies.
- updated project wide SBOM files.
- updated project wide copyright note.
- Migrated to dedicted pure HKT trait.

## [0.3.2](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_discovery-v0.3.1...deep_causality_discovery-v0.3.2) - 2025-12-31

### Other

- Updated SBOMs to trigger release.

## [0.3.1](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_discovery-v0.3.0...deep_causality_discovery-v0.3.1) - 2025-12-18

### Fixed

- *(deep_causality_data_structures)* Fixed a number of bugs. Updated tests for verification.
- *(deep_causality_discovery)* Fixed a number of bugs. Updated tests for verification.

## [0.3.0](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_discovery-v0.2.6...deep_causality_discovery-v0.3.0) - 2025-12-12

### Added

- *(deep_causality_discovery)* Added option to start with no data as to accomodate custom data loaders.
- *(deep_causality_discovery)* Added cohort filtering.
- *(deep_causality_discovery)* Improved Apply implementation by removing clone overhead.
- *(deep_causality_discovery)* Improved API with new load_data_with_config constructor.
- *(deep_causality_discovery)* Improved test coverage.
- *(deep_causality_discovery)* Re-implemented CDL with HKT from haft crate. Updated tests, examples and README.md

### Fixed

- *(deep_causality_discovery)* Fixed SurdResultAnalyzer and related tests.
- fixed a number of Bazel config files.

### Other

- *(deep_causality_discovery)* Increased test coverage.
- Linting.
- code formatting and linting
- Added or updated documentation.
- Added a lot more examples across physics and medicine.

## [0.2.6](https://github.com/marvin-hansen/deep_causality/compare/deep_causality_discovery-v0.2.5...deep_causality_discovery-v0.2.6) - 2025-12-03

### Added

- *(deep_causality_core)* Added test coverage

### Other

- Regenerated SBOM.
- Updated external deps in discovery crate and ICU example.
- Merge branch 'deepcausality-rs:main' into main
- Merge remote-tracking branch 'origin/main'
- Restored manually generated SBOM to restore Dependency and licence scan.

## [0.2.5](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_discovery-v0.2.4...deep_causality_discovery-v0.2.5) - 2025-11-23

### Other

- Merge branch 'deepcausality-rs:main' into main

### Removed

- removed all manually generated SBOM files

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
