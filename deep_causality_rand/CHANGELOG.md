# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/deepcausality-rs/deep_causality/releases/tag/deep_causality_rand-v0.1.0) - 2025-09-19

### Added

- *(deep_causality_tensor)* Initial setup. Moved CausalTensor type from the data_structure crate into dedicated deep_causality_tensor crate.
- *(deep_causality_rand)* added README.md
- *(deep_causality_num)* Initial implementation. Update of all downstream crates.
- *(deep_causality_rand)* Fixed issue with std pseudo number generator. Updated upstream crate deep_causality_uncertain
- *(deep_causality_uncertain)* Migrated random number generation to deep_causality_rand crate
- *(deep_causality_rand)* Base implementation for rand.

### Other

- *(deep_causality_rand)* increased test coverage
- *(deep_causality_rand)* increased test coverage
- *(deep_causality_rand)* increased test coverage
- *(deep_causality_rand)* increased test coverage
- *(deep_causality_rand)* increased test coverage
- *(deep_causality_num)* increased test coverage
- test(deep_causality_rand) Increased test coverage.
- Reworked and updated Bazel test config across all crates.
- Lots of lints and formats. Updated MSRV to 1.89 and edition 2024 across the entire repo.
- *(deep_causality_rand)* Increased test coverage.
- *(deep_causality_rand)* increased test coverage
- *(deep_causality_num)* increased test coverage
- Added or updated Bazel config for the newly added crates.
