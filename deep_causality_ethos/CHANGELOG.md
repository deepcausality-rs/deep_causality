# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.10](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_ethos-v0.2.9...deep_causality_ethos-v0.2.10) - 2026-07-24

### Other

- *(website)* update deps and add the CFD site to the README

## [0.2.9](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_ethos-v0.2.8...deep_causality_ethos-v0.2.9) - 2026-07-14

### Added

- *(deep_causality_haft)* add Category + Kleisli (named category, compose = bind) — H2

### Other

- add user-facing SKILLS.md agent file for building with deep_causality

## [0.2.8](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_ethos-v0.2.7...deep_causality_ethos-v0.2.8) - 2026-07-08

### Other

- *(num)* split deep_causality_num into num-core + algebra + complex + dual
- *(bazel)* register all missing test suites; add Dual Default; move iso test utils to src/utils_tests
- *(readme)* use absolute raw URLs for logo images
- Restructured the avionics example folder.
- Generated new SBOM for all crates.
- *(papers)* Reorganized publication by moving each paper into the crate where it is actually implemented.
- Updated README file across multiple crates to meet project standard.

## [0.2.7](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_ethos-v0.2.6...deep_causality_ethos-v0.2.7) - 2026-06-09

### Other

- *(website)* Added a new blog post that introduces the CosaFlow DSL.
- Added an official statement on the usage of AI coding assistance and added it to the README under the contributing section.
- Added Miri badge to README.md
- enforce repo-wide `unsafe_code = "forbid"`; remove avoidable unsafe

## [0.2.6](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_ethos-v0.2.5...deep_causality_ethos-v0.2.6) - 2026-05-29

### Other

- Updated the project README.md
- Updated logo on the main README.md

## [0.2.5](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_ethos-v0.2.4...deep_causality_ethos-v0.2.5) - 2026-05-26

### Added

- *(deep_causality_topology)* cubical Regge core R1 — cell volumes
- *(deep_causality_num)* add Tier 2 witness-typed Iso traits + StandardIso (iso-traits Stage B)

### Other

- *(deep_causality_physics)* promote CauchyStress to dedicated ViscousStress / ReynoldsStress newtypes
- Fixed a typo.
- Updated example Readme.
- *(deep_causality_effects)* retire crate
- *(README)* Fixed broken link in the project README.md
- Removed FOSSA from Readme
- Added a README.md to the deep_causality_ethos crate
- Updated the README file.
- Updated project README.md
- Updated example. Read me to add the latest example and add a more sensible ordering of all existing examples.
- Edited. Readme again.
- Added writing guides and updated the README.md

## [0.2.4](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_ethos-v0.2.3...deep_causality_ethos-v0.2.4) - 2026-03-12

### Other

- Updated all SBOMS to reflect lates depdency versions.

## [0.2.3](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_ethos-v0.2.2...deep_causality_ethos-v0.2.3) - 2026-01-22

### Fixed

- fixed typo in README.md

### Other

- *(deep_causality_ethos)* Improved existing tests.

## [0.2.2](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_ethos-v0.2.1...deep_causality_ethos-v0.2.2) - 2026-01-09

### Other

- updated project wide SBOM files.
- updated project wide copyright note.

## [0.2.1](https://github.com/marvin-hansen/deep_causality/compare/deep_causality_ethos-v0.2.0...deep_causality_ethos-v0.2.1) - 2025-12-31

### Other

- Minor lints.

## [0.2.0](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_ethos-v0.1.1...deep_causality_ethos-v0.2.0) - 2025-12-18

### Fixed

- *(deep_causality_effects)* Format and linting
- *(deep_causality_effects)* Fixed a number of bugs. Updated tests for verification.

## [0.1.1](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_ethos-v0.1.0...deep_causality_ethos-v0.1.1) - 2025-12-12

### Fixed

- fixed a number of Bazel config files.

### Other

- Added a few more medical examples.
- Added or updated documentation.
- Updated Project README.md
