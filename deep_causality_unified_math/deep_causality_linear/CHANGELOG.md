# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.3](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_linear-v0.2.2...deep_causality_linear-v0.2.3) - 2026-09-25

### Other

- Merge remote-tracking branch 'origin/main'
- Commit message for the exclude change:

## [0.2.2](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_linear-v0.2.1...deep_causality_linear-v0.2.2) - 2026-09-23

### Other

- edit all READMEs for clarity, concision and correctness
- *(deep_causality_unified_math)* regenerated SBOM.

## [0.2.1](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_linear-v0.2.0...deep_causality_linear-v0.2.1) - 2026-09-15

### Fixed

- *(CI)* Fixing release-plz auto-release.

## [0.2.0](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_linear-v0.1.4...deep_causality_linear-v0.2.0) - 2026-09-15

### Added

- *(haft)* add the Collectable capability and implement it for the rank-1 witnesses
- *(deep_causality_haft)* add the diagonal traversal for zip witnesses
- *(deep_causality_topology)* add the CochainWitness HKT witness
- *(deep_causality_linear)* implement Traversable for DenseVectorWitness

### Fixed

- *(unified_math)* [**breaking**] replace From<f64> bounds with FromPrimitive, add software-scalar tensor ops

## [0.1.4](https://github.com/marvin-hansen/deep_causality/compare/deep_causality_linear-v0.1.3...deep_causality_linear-v0.1.4) - 2026-09-08

### Added

- *(deep_causality_linear)* move the fixed-size dense forms in, and close the C4 adoption

### Fixed

- *(deep_causality_unified_math)* close audited correctness holes across the unified_math stack
- *(Bazel)* align crate features with cargo's resolution and recover 161 unrun tests
- *(deep_causality_linear)* scale the Euclidean norms and adopt the crate across consumers

### Other

- *(Cargo)* Updated metadata in all crates.
