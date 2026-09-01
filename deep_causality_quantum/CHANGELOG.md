# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_quantum-v0.1.2...deep_causality_quantum-v0.2.0) - 2026-09-01

### Added

- *(deep_causality_quantum)* decide class invariance over the code space
- *(homology,topology,quantum)* [**breaking**] close the last five QCL gaps
- *(deep_causality_quantum)* [**breaking**] retype the Haruna gate layer onto Gf2Chain and close four QCL gaps
- *(quantum,num,topology)* close the six low-effort QCL gaps

### Fixed

- *(CI)* fixed broken lean theorem.

### Other

- *(cargo)* hoist every dependency into [workspace.dependencies]
- *(bazel)* merge test BUILD files and derive deps from Cargo
- consolidate the mathematics crates under deep_causality_unified_math/
