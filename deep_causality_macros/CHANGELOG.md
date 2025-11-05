# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.8.9](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_macros-v0.8.8...deep_causality_macros-v0.8.9) - 2025-11-05

### Other

- Updated SBOM for all crates.

## [0.8.8](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_macros-v0.8.7...deep_causality_macros-v0.8.8) - 2025-09-25

### Other

- Updated SBOM for all crates.

## [0.8.6](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_macros-v0.8.5...deep_causality_macros-v0.8.6) - 2025-09-22

### Fixed

- *(deep_causality)* Removed last internal macros and removed dependency on deep_causality_macro crate.

### Other

- Updated SBOM script to generate hash signature together with the SBOM.

## [0.8.5](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_macros-v0.8.4...deep_causality_macros-v0.8.5) - 2025-09-21

### Added

- *(deep_causality_rand)* added README.md

### Other

- Updated Cargo configuration and feature propagation across the entire repo.
- Lots of lints and formats. Updated MSRV to 1.89 and edition 2024 across the entire repo.
- *(deep_causality_rand)* Increased test coverage.

## [0.8.4](https://github.com/marvin-hansen/deep_causality/compare/deep_causality_macros-v0.8.3...deep_causality_macros-v0.8.4) - 2025-09-11

### Added

- *(deep_causality_macros)* Increased test coverage.
- *(deep_causality_macros)* removed getter macro. Added coord_match! macro. Rewrote all macros in the macro_rules syntax.

### Other

- trying to fix #315

## [0.8.3](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_macros-v0.8.2...deep_causality_macros-v0.8.3) - 2025-09-08

### Added

- *(ultragraph)* removed unused dependencies. ultragraph has no zero build dependencies.
- *(deep_causality_macros)* removed overly complex constructure macro.

## [0.8.2](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_macros-v0.8.1...deep_causality_macros-v0.8.2) - 2025-08-27

### Other

- update Cargo.toml dependencies

## [0.8.1](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_macros-v0.8.0...deep_causality_macros-v0.8.1) - 2025-08-08

### Other

- Updated copyright in Cargo.toml fils
- Updated CausableReasoning trait to handle RelayTo variant to dispatch to a different causaloid.

## [0.4.11](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_macros-v0.4.10...deep_causality_macros-v0.4.11) - 2025-05-16

### Other

- Applied a ton of lints.

## [0.4.10](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_macros-v0.4.9...deep_causality_macros-v0.4.10) - 2024-11-24

### Other

- Code linting & formatting
- Bumped up minimum rust version to 1.80.
- Increased test coverage of constructor macro
- Increased test coverage of getter macro
- Added tests to macros

## [0.4.9](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_macros-v0.4.8...deep_causality_macros-v0.4.9) - 2024-11-21

### Other

- update Cargo.lock dependencies

## [0.4.8](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_macros-v0.4.7...deep_causality_macros-v0.4.8) - 2023-09-19

### Other
- Added custom is_empty implementation to test if codecov recolonize it.
- Restored macro tests
- [no ci]

## [0.4.7](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_macros-v0.4.6...deep_causality_macros-v0.4.7) - 2023-09-06

### Other
- Updated Cargo.toml for macros

## [0.4.6](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_macros-v0.4.5...deep_causality_macros-v0.4.6) - 2023-08-30

### Other
- Formatted entire code base with rustfmt.
- Working on adjustable types.
- Code formatting.
- Code formatting.
- Added more tests to macros.
- Fixed failing test.
- Added more tests to macros.
- Code formatting.
- Added more tests to macros.
- Fixed a number of linting issues.
- Renamed example code files to prevent output file name collision.
- Updated adjustable types in deep causality to use macros to generate constructor and getters.
- Added proper Readme to macro crate.
- Added example code for new macros.
- Added tests for new macros.
- Moved previous macros to separate file. Updated lib.
- Added new constructor generator macro.
- Added new getter generator macro.
- Misc minor changes.
- Code formatting of macros
- Updated copyright in all source and bash script files.
- Updated copyright in all licence files.

## [0.4.5](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_macros-v0.4.4...deep_causality_macros-v0.4.5) - 2023-08-17

### Other

- Updated SPDX-License-Identifier to GFM comment to prevent rendering meta data as table.
- Updated copyright with SPDX-License code.
- Added SPDX-License-Identifier to all docs
- Removed make_main macro with time_execution util function.
