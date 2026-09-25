# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.3](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_stats-v0.2.2...deep_causality_stats-v0.2.3) - 2026-09-25

### Other

- Merge remote-tracking branch 'origin/main'
- Commit message for the exclude change:

## [0.2.2](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_stats-v0.2.1...deep_causality_stats-v0.2.2) - 2026-09-23

### Other

- align READMEs, rustdoc and example comments with the code
- edit all READMEs for clarity, concision and correctness
- *(deep_causality_unified_math)* regenerated SBOM.

## [0.2.1](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_stats-v0.2.0...deep_causality_stats-v0.2.1) - 2026-09-15

### Fixed

- *(CI)* Fixing release-plz auto-release.

## [0.2.0](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_stats-v0.1.0...deep_causality_stats-v0.2.0) - 2026-09-15

### Added

- *(deep_causality_stats)* [**breaking**] seven distributions, generic in the scalar

### Fixed

- *(stats)* sum the multi-accumulator reductions as trees too, and give them a suite that bites
- *(stats)* sum as a balanced tree, and stop calling the arrangement's limit the type's
- *(unified_math)* [**breaking**] replace From<f64> bounds with FromPrimitive, add software-scalar tensor ops

### Other

- *(unified_math)* [**breaking**] precision as a parameter in the sampling layer, for real
- *(unified_math)* add the Ising ensemble example and record the split
- *(unified_math)* [**breaking**] finish the sampling split at the consumers
- *(unified_math)* [**breaking**] precision as a parameter in the sampling layer
- *(unified_math)* [**breaking**] move the distributions from rand to stats
- *(deep_causality_rand)* [**breaking**] split StandardUniform by what it samples

## [0.1.0](https://github.com/marvin-hansen/deep_causality/releases/tag/deep_causality_stats-v0.1.0) - 2026-09-08

### Added

- *(deep_causality_stats)* Added SBOM and CHANGELOG.md
- *(deep_causality_stats)* add descriptive and information statistics at tier 4

### Fixed

- *(deep_causality_unified_math)* close audited correctness holes across the unified_math stack
- *(Bazel)* align crate features with cargo's resolution and recover 161 unrun tests

### Other

- *(openspec)* Archived specs and notes for unified_math_next.
- *(deep_causality_stats)* [**breaking**] centralize workspace statistics in the stats crate
- *(deep_causality_stats)* improved some corner case testing.
