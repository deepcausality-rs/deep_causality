# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.3](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_file-v0.1.2...deep_causality_file-v0.1.3) - 2026-07-13

### Added

- *(deep_causality_haft)* add Category + Kleisli (named category, compose = bind) — H2

### Other

- Improved test coverage.

## [0.1.2](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_file-v0.1.1...deep_causality_file-v0.1.2) - 2026-07-08

### Other

- release

## [0.1.1](https://github.com/deepcausality-rs/deep_causality/releases/tag/deep_causality_file-v0.1.1) - 2026-07-08

### Added

- *(deep_causality_cfd)* Added a group 1 of the cfd flow DSL rework
- *(cfd,physics,file,examples)* study DSL (sweep, Gates, run_owned, duct_march) + three gated examples
- *(file,cfd)* CFD file IO seams — typed tables, sensor traces, snapshot/resume
- *(deep_causality_file)* initial version of the file crate for handling data loading and processing

### Fixed

- *(deep_causality_file)* Set publish to false to fix CI auto-release for the time being.
- *(deep_causality_file)* Fixed dependencies versioning issue in Cargo.toml that prevented auto-release via CI.

### Other

- Releasig new file crate
- release
- *(num)* split deep_causality_num into num-core + algebra + complex + dual
- *(physics,tensor,file)* skip Miri-incompatible tests
- code formatting and linting.
- code formatting and linting.
- code formatting and linting.
- *(deep_causality_cfd)* Increased test coverage
- *(deep_causality_cfd)* Increased test coverage
- *(cfd)* compile_fail doctests for the study phase discipline (DSL rework group 5.4)
- *(deep_causality_file)* full test suite for loaders, types, errors
