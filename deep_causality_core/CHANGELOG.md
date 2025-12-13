# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.2](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_core-v0.0.1...deep_causality_core-v0.0.2) - 2025-12-12

### Other

- *(deep_causality_core)* release v0.0.1

## [0.0.1](https://github.com/deepcausality-rs/deep_causality/releases/tag/deep_causality_core-v0.0.1) - 2025-12-12

### Added

- *(deep_causality_core)* enabled relase of crate.
- *(deep_causality_core)* Added bind_or_error to CausalEffectPropagationProcess
- *(deep_causality)* Updated extension tests to new API.
- *(deep_causality_core)* Removed unrelated types.
- *(deep_causality_core)* Re-implemented intervenable trait. Added new tests. Linting and code formatting.
- *(deep_causality)* Initial re-write using deep_causality_core crate for functional core.
- *(deep_causality_core)* Added test coverage
- *(deep_causality_num)* Added algebraic trait bounds.
- *(deep_causality_core)* Updated ControlFlowBuilder, Added new strict_zst examples, and updated CausalityError to be zero allocation.
- *(deep_causality_core)* Added ControlFlowBuilder, examples, and a README.md
- *(deep_causality_core)* First draft of new core crate

### Fixed

- fixed a number of Bazel config files.
- *(deep_causality)* Restored proper fn pointers in CausalFn and ContextualCausalFn.

### Other

- *(deep_causality_physics)* Code formatting and linting.
- *(deep_causality_physics)* Improved test coverage.
- *(deep_causality_physics)* Added more tests.
- *(deep_causality_physics)* Added more tests.
- Reorganized and updated repo wide examples.
- *(deep_causality_core)* Improved test coverage
- Regenerated SBOM.
- Fixed Bazel build config.
- *(deep_causality_core)* Improved test coverage
- *(deep_causality_core)* Improved test coverage
- *(deep_causality_core)* Improved test coverage
- Working on Bazel build config
- *(deep_causality_multivector)* Improved test coverage
- *(deep_causality_core)* Improved test coverage
- *(deep_causality_core)* Improved test coverage
- Updated Dev dependencies.
- Updated note on core type system design.
- Added note on hte preliminary design of the core crate.
- *(deep_causality_core)* Added Bazel configuration and some initial tests.
- *(deep_causality_core)* Added License and SBOM.
- *(deep_causality_core)* Separated CausalEffectPropagationProcess as a dedicated shared type for arity-3  PropagatingEffect and arity-5 PropagatingProcess.
- *(deep_causality_core)* Restructured code organization.
- *(deep_causality_core)* Lints and formatting.
- *(deep_causality_core)* Reworked Effect Log.
- *(deep_causality_core)* Fixed doctests.
