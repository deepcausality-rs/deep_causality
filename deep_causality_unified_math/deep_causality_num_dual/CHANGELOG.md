# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_num_dual-v0.1.6...deep_causality_num_dual-v0.2.0) - 2026-09-01

### Added

- *(num_dual)* close E2 — drop the Dual struct bound, add DualWitness

### Fixed

- *(unified_math)* [**breaking**] act on a review of the HKT law and constraint work

### Other

- *(cargo)* hoist every dependency into [workspace.dependencies]
- *(bazel)* merge test BUILD files and derive deps from Cargo
- Applied lints and fixes.
- *(deep_causality_haft)* [**breaking**] remove the HKT Constraint system and the Satisfies marker
- *(openspec)* Updated paths in docs of relocated crates across the repo
- *(openspec)* Updated inbound links to archived notes
- *(openspec)* Archived specs and notes on lax monoidal applicative
- *(haft)* record why VecWitness has no Traversable, with the measurement
- consolidate the mathematics crates under deep_causality_unified_math/

## [0.1.4](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_num_dual-v0.1.3...deep_causality_num_dual-v0.1.4) - 2026-07-14

### Other

- *(miri)* ignore compute-heavy FFT/multivector tests; fix nextest period
- drop Aeneas / L4 from the verification program (non-goal)
- *(num)* add crate-local Lean verification status notes for the numeric tower
- *(openspec)* close out formalize-main-crate — main-crate Lean status note, sync + archive

## [0.1.3](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_num_dual-v0.1.2...deep_causality_num_dual-v0.1.3) - 2026-07-08

### Other

- release

## [0.1.2](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_num_dual-v0.1.0...deep_causality_num_dual-v0.1.2) - 2026-07-08

### Other

- Bumping up versions of dual and complex crate.
- release

## [0.1.0](https://github.com/deepcausality-rs/deep_causality/releases/tag/deep_causality_num_dual-v0.1.0) - 2026-07-08

### Added

- *(lean)* formalize the extracted num/algebra/complex/dual crates (L1)

### Other

- Update Cargo.toml
- Fixed dependencies version in various crate to fix CI auto release.
- *(num)* split deep_causality_num into num-core + algebra + complex + dual
