# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.5](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_num_dual-v0.1.4...deep_causality_num_dual-v0.1.5) - 2026-08-25

### Added

- *(algebra)* [**breaking**] state each algebraic law about the operation it governs

### Fixed

- *(release)* bump algebra to 0.3.0 so the new traits are actually published
- *(deep_causality_algebra)* [**breaking**] stop handing out algebraic laws by inference

### Other

- General QA: Bug fixes and code improvements.
- *(deep_causality_num_dual)* Updated SBOM.
- Updated version requirmentes
- bring the algebra reference current and correct the CI claim

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
