# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/deepcausality-rs/deep_causality/releases/tag/deep_causality_algebra-v0.1.0) - 2026-07-08

### Added

- *(lean)* formalize the extracted num/algebra/complex/dual crates (L1)

### Fixed

- address code-review findings (graph-reasoning docs/test, stale doc example, clippy)

### Other

- Fixed dependencies version in various crate to fix CI auto release.
- code formatting and linting.
- rename the Lean layer Num → Algebra to mirror the new crate.
- *(num)* split deep_causality_num into num-core + algebra + complex + dual
