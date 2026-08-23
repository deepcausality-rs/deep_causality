# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_algebra-v0.2.0...deep_causality_algebra-v0.3.0) - 2026-08-23

### Added

- *(lean)* formalize the Euclidean domain and Q
- *(deep_causality_algebra)* add the semiring level so N has an algebraic slot
- *(deep_causality_algebra)* [**breaking**] admit the integers into the algebra tower

### Fixed

- *(deep_causality_algebra)* [**breaking**] stop handing out algebraic laws by inference
- *(deep_causality_algebra)* [**breaking**] make the Euclidean contract honest at the edges
- *(deep_causality_algebra)* [**breaking**] AddGroup requires Neg, not merely Sub
- *(deep_causality_algebra)* [**breaking**] normalize the Euclidean gcd and stop lcm overflowing

### Other

- More fixes and lints.
- Improved test coverage
- bring the algebra reference current and correct the CI claim

## [0.2.0](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_algebra-v0.1.1...deep_causality_algebra-v0.2.0) - 2026-07-14

### Added

- *(deep_causality)* the graph algebra — schedule-invariant ∇∘(Λ⊗Λ) joins + freeze checks (roadmap Stage 4)
- *(deep_causality_haft)* add Category + Kleisli (named category, compose = bind) — H2

### Fixed

- *(deep_causality_algebra)* CommutativeMonoid requires the Commutative marker

### Other

- *(miri)* ignore compute-heavy FFT/multivector tests; fix nextest period
- Improved test coverage.
- *(openspec)* close out formalize-main-crate — main-crate Lean status note, sync + archive

## [0.1.1](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_algebra-v0.1.0...deep_causality_algebra-v0.1.1) - 2026-07-08

### Other

- updated the following local packages: deep_causality_num
