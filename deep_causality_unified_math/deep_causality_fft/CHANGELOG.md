# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.5](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_fft-v0.1.4...deep_causality_fft-v0.1.5) - 2026-09-08

### Fixed

- *(Bazel)* align crate features with cargo's resolution and recover 161 unrun tests

### Other

- *(Cargo)* Updated metadata in all crates.

## [0.1.2](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_fft-v0.1.1...deep_causality_fft-v0.1.2) - 2026-07-14

### Added

- *(deep_causality_haft)* add Category + Kleisli (named category, compose = bind) — H2

### Other

- *(miri)* Gated two more FFI tests to skip MIRI.
- *(miri)* ignore compute-heavy FFT/multivector tests; fix nextest period

## [0.1.1](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_fft-v0.1.0...deep_causality_fft-v0.1.1) - 2026-07-08

### Other

- *(num)* split deep_causality_num into num-core + algebra + complex + dual
- *(bazel)* register all missing test suites; add Dual Default; move iso test utils to src/utils_tests
- Generated new SBOM for all crates.
