# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.5](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_quantum-v0.2.0...deep_causality_quantum-v0.2.5) - 2026-09-08

### Added

- *(deep_causality_linear)* move the fixed-size dense forms in, and close the C4 adoption
- *(deep_causality_num)* add the lift module and retire the inherent Float106::from_f64
- *(deep_causality_quantum)* add the three QCL consumer examples and close out add-qcl
- *(deep_causality_quantum)* add the QCL pipeline, from one config origin through validate, Screened and control
- *(deep_causality_quantum)* add Hypothesis with the mechanism-level intervention, embedded prediction and warrant-gated marginalisation
- *(deep_causality_quantum)* add the typed carriers, the shot budget and the default-build Born sampler
- *(deep_causality_quantum)* [**breaking**] finish class invariance with the normalizer check, the Clifford tableau and the tuple cap

### Fixed

- *(deep_causality_quantum)* Fixed broken version number that failed auto-release on CI.
- *(deep_causality_unified_math)* close audited correctness holes across the unified_math stack
- *(Bazel)* align crate features with cargo's resolution and recover 161 unrun tests
- *(deep_causality_linear)* scale the Euclidean norms and adopt the crate across consumers
- *(deep_causality_quantum)* resolve the QCL review findings, with the Float106 defects beneath them
- *(deep_causality_quantum,openspec)* correct C₃ to Definition 3.1 and apply the QCL corrections register

### Other

- *(deep_causality_stats)* [**breaking**] centralize workspace statistics in the stats crate
- *(Cargo)* Updated metadata in all crates.
- Fixed PR review issues.
- *(deep_causality_quantum)* add design as an exact pair cover and adjudicate under the verdict law
- Updated Rust depencies to the latest version.
