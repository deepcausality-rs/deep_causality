# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.1](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_algorithms-v0.4.0...deep_causality_algorithms-v0.4.1) - 2026-07-13

### Fixed

- *(deep_causality_physics)* Fixing 10MB max upload limit on crates.io

### Other

- Improved test coverage.

## [0.4.0](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_algorithms-v0.3.0...deep_causality_algorithms-v0.4.0) - 2026-07-08

### Added

- *(deep_causality_algorithms)* opt-in O(du) MAP-config pruning for BRCD (near-linear path)
- *(deep_causality_algorithms)* add dag_sampling uniform MEC DAG sampler
- *(deep_causality_algorithms)* add dag_sampling — polynomial-time Clique-Picking AMO counter

### Fixed

- *(deep_causality_algorithms)* Fixed miri test config
- *(deep_causality_algorithms)* Fixed miri test config
- *(deep_causality_algorithms,deep_causality_discovery)* cache version tag, docs,
- *(deep_causality_algorithms)* Removed dead code
- *(deep_causality_algorithms)* remove latent panic in Clique-Picking AMO counter
- *(brcd,discovery)* address QA findings (32-bit shift, DRY, Precision bound)
- *(deep_causality_algorithms)* Resolved sorting order issues reported in https://github.com/deepcausality-rs/deep_causality/issues/641
- *(deep_causality_algorithms)* correct verification data path after brcd-paper move

### Other

- *(num)* split deep_causality_num into num-core + algebra + complex + dual
- strengthen physics quantity assertions and fix review nits
- *(deep_causality_algorithms)* close remaining coverage gaps with targeted branch tests
- *(deep_causality_algorithms)* memoize invalid MapPrune orientations
- *(deep_causality_algorithms)* updated
- *(deep_causality_algorithms)* parallelize BRCD across candidates; add BRCD eval harnesses + companion papers
- *(deep_causality_algorithms)* remove experimental BRCD thesis probes from verification
- *(deep_causality_algorithms)* wire BRCD to the polynomial dag_sampling counter + sampler
- Merge branch 'brcd-paper'
- Generated new SBOM for all crates.
- *(papers)* Reorganized publication by moving each paper into the crate where it is actually implemented.
- Updated README file across multiple crates to meet project standard.

## [0.3.0](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_algorithms-v0.2.14...deep_causality_algorithms-v0.3.0) - 2026-06-09

### Added

- *(deep_causality_algorithms)* BOSS bootstrap CPDAG-uncertainty variant for BRCD
- *(deep_causality_algorithms)* wire BOSS into brcd_run via an optional CPDAG (BREAKING)
- *(deep_causality_algorithms)* BOSS structure-learning core for BRCD (score, GST, order search)
- *(deep_causality_algorithms)* Updated SBOM
- *(deep_causality_algorithms)* BRCD posterior assembly + driver (runs end-to-end)
- *(deep_causality_algorithms)* BRCD F-node augmentation + cut-config enumeration + cache
- *(deep_causality_algorithms)* BRCD F-integration (mixture of experts) + Dirichlet
- *(deep_causality_algorithms)* BRCD ridge-Gaussian family estimator + transform ladder
- *(deep_causality_algorithms)* BRCD logistic-regression gate (IRLS) + shared SPD solve
- *(deep_causality_algorithms)* BRCD MEC engine — exact AMO enumeration over MixedGraph
- *(deep_causality_algorithms)* BRCD causal-graph ops — Meek, validity, MEC over MixedGraph

### Fixed

- *(deep_causality_algorithms)* Fixed path to data in verification examples.

### Other

- *(openspec)* retarget the calculus change to deep_causality_calculus
- *(deep_causality_algorithms)* supplied-vs-BOSS CPDAG comparison examples
- *(deep_causality_algorithms)* BOSS structure-learning verification example
- Merge remote-tracking branch 'origin/main'
- *(deep_causality_algorithms)* Added benchmark for the BRCD algorithm. Implemented multiple performance improvements while keeping the verification results identical. Added parallel version for larger graphs.
- *(deep_causality_algorithms)* Added benchmark for the BRCD algorithm.
- *(deep_causality)* fix rustdoc intra-doc link warnings
- *(deep_causality_algorithms)* improve test coverage.
- *(deep_causality_algorithms)* Updated README.md
- *(deep_causality_algorithms)* Updated README.md
- *(algorithms)* Verify the BRCD algorithm against the authoritative Python reference on
- *(num,tensor,algorithms)* generify the numeric + SURD stack over RealField
- stabilize two more tests against Miri soft-float drift
- enforce repo-wide `unsafe_code = "forbid"`; remove avoidable unsafe

## [0.2.14](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_algorithms-v0.2.13...deep_causality_algorithms-v0.2.14) - 2026-05-26

### Other

- Updated example Readme.
- *(examples)* consolidate algorithms/discovery examples into causal_discovery_examples

## [0.2.13](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_algorithms-v0.2.12...deep_causality_algorithms-v0.2.13) - 2026-05-03

### Other

- Updated Cargo deps.

## [0.2.12](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_algorithms-v0.2.11...deep_causality_algorithms-v0.2.12) - 2026-03-12

### Other

- Updated all SBOMS to reflect lates depdency versions.

## [0.2.11](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_algorithms-v0.2.10...deep_causality_algorithms-v0.2.11) - 2026-02-09

### Other

- updated all cargo dependencies to the latest version.

## [0.2.10](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_algorithms-v0.2.9...deep_causality_algorithms-v0.2.10) - 2026-01-22

### Other

- *(deep_causality_algorithms)* Fixed consitency bug.
- Updated SBOM of and applied docstring fixes.

## [0.2.9](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_algorithms-v0.2.8...deep_causality_algorithms-v0.2.9) - 2026-01-09

### Other

- updated project wide SBOM files.
- updated project wide copyright note.

## [0.2.8](https://github.com/marvin-hansen/deep_causality/compare/deep_causality_algorithms-v0.2.7...deep_causality_algorithms-v0.2.8) - 2025-12-31

### Added

- *(deep_causality_tensor)* Finalized new backend agnostic CPU impl.

## [0.2.7](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_algorithms-v0.2.6...deep_causality_algorithms-v0.2.7) - 2025-12-18

### Other

- updated the following local packages: deep_causality_num, deep_causality_tensor

## [0.2.6](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_algorithms-v0.2.5...deep_causality_algorithms-v0.2.6) - 2025-12-12

### Added

- *(deep_causality_algorithms)* Added dedicated MrmrResult type.  Updated all downstream usage, examples and added relevant tests.

### Fixed

- fixed a number of Bazel config files.

### Other

- Updated criterion across the repo.

## [0.2.5](https://github.com/marvin-hansen/deep_causality/compare/deep_causality_algorithms-v0.2.4...deep_causality_algorithms-v0.2.5) - 2025-12-03

### Added

- *(deep_causality_tensor)* Added full set of new algebraic trait system to CausalTensor.

### Other

- Regenerated SBOM.
- Updated dev dependencies across the repo.
- Updated Dev dependencies.
- Merge branch 'deepcausality-rs:main' into main
- Merge remote-tracking branch 'origin/main'
- Restored manually generated SBOM to restore Dependency and licence scan.

## [0.2.4](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_algorithms-v0.2.3...deep_causality_algorithms-v0.2.4) - 2025-11-23

### Other

- Merge branch 'deepcausality-rs:main' into main

### Removed

- removed all manually generated SBOM files

## [0.2.3](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_algorithms-v0.2.2...deep_causality_algorithms-v0.2.3) - 2025-11-23

### Added

- *(deep_causality_tensor)* refactored public Tensor API as a trait.

## [0.2.2](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_algorithms-v0.2.1...deep_causality_algorithms-v0.2.2) - 2025-11-05

### Other

- Updated SBOM for all crates.

## [0.2.1](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_algorithms-v0.2.0...deep_causality_algorithms-v0.2.1) - 2025-10-31

### Added

- *(deep_causality_tensor)* Moved TensorProduct implementation from an extension into the CausalTensor  type.

## [0.2.0](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_algorithms-v0.1.4...deep_causality_algorithms-v0.2.0) - 2025-10-02

### Added

- *(deep_causality_algorithms)* Generic MRMR
- *(deep_causality_algorithms)* Parallelize mRMR feature selection algo.
- *(deep_causality_algorithms)* Added new example.
- *(deep_causality_algorithms)* Added mrmr score to feature selection. documented in specs/006-mrmr-feature-score.
- *(deep_causality_algorithms)* Updated MRMR Error with new variant for score calculation.
- *(deep_causality_algorithms)* Added CDL variant of SURD state algo to handle None / NaN values in data
- *(deep_causality_algorithms)* Added a CDL variant of the MRMR algorithm that uses CausalTensor<Option<f64>> with pairwise value selection to deal with missing data without introducing bias.
- *(deep_causality_algorithms)* updated MRMRM Error

### Fixed

- *(deep_causality_algorithms)* renamed mrmr algo

### Other

- *(deep_causality_algorithms)* removed dead code and unreachable errors to increased test coverage.
- *(deep_causality_algorithms)* increased test coverage.
- Added specs for parallel implementation of mrmr algo
- Added stage 2 to ICU sepsis case study
- *(deep_causality_algorithms)* Updated README.md
- *(deep_causality_algorithms)* Added test coverage for CDL SURD variant.

## [0.1.4](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_algorithms-v0.1.3...deep_causality_algorithms-v0.1.4) - 2025-09-25

### Other

- Updated SBOM for all crates.

## [0.1.2](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_algorithms-v0.1.1...deep_causality_algorithms-v0.1.2) - 2025-09-22

### Fixed

- *(deep_causality)* Removed last internal macros and removed dependency on deep_causality_macro crate.

### Other

- Updated SBOM script to generate hash signature together with the SBOM.

## [0.1.1](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_algorithms-v0.1.0...deep_causality_algorithms-v0.1.1) - 2025-09-21

### Added

- *(deep_causality_tensor)* Initial setup. Moved CausalTensor type from the data_structure crate into dedicated deep_causality_tensor crate.
- *(deep_causality_num)* Initial implementation. Update of all downstream crates.
- *(deep_causality_algorithms)* Implement mRMR (FCQ variant) feature selector

### Fixed

- *(deep_causality_algorithms)* Fixed Bazel build.

### Other

- Updated Cargo configuration and feature propagation across the entire repo.
- Reworked and updated Bazel test config across all crates.
- Merge branch '002-replace-rng-currently'
- Lots of lints and formats. Updated MSRV to 1.89 and edition 2024 across the entire repo.
- *(deep_causality_rand)* Increased test coverage.

## [0.1.0](https://github.com/marvin-hansen/deep_causality/releases/tag/deep_causality_algorithms-v0.1.0) - 2025-09-15

### Added

- *(deep_causality_data_structures)* bumped up version number
- *(deep_causality_algorithms)* Updated Bazel build and test config.
- *(deep_causality_data_structures)* Added identity types. Removed num_traits dependency.
- *(deep_causality_algorithms)* Added a README.md and LICENSE file.
- *(deep_causality_algorithms)* Added a README.md to the example_surd in the example folder.
- *(deep_causality_algorithms)* Implemented SURD-State algorithm described in the paper "Observational causality by states and interaction type for scientific discovery"
- *(deep_causality_algorithms)* Improved  SURD-Stat algorithms with separation of causal and non-causal state dependent maps. Improved documentation of the algo.
- *(deep_causality_algorithms)* Initial implementation of the SURD-State algorithms.

### Fixed

- *(deep_causality_algorithms)* Fixed multiple division by zero issue and replaced unsafe tensor division with save_div.

### Other

- Restored version number...
- Removed version number form internal deps.
- *(deep_causality_algorithms)* linting and formatting
- *(deep_causality_data_structures)* Improved test coverage.
- *(deep_causality_algorithms)* Improved test coverage.
- *(deep_causality_data_structures)* Improved test coverage.
- *(deep_causality_algorithms)* Improved test coverage.
- *(deep_causality_algorithms)* Improved test coverage.
- Code formating and linting across the repo. Moved old and empty dcl crate into yanked folder.
- *(deep_causality_data_structures)* Increased test coverage for CausalTensorMathExt
- *(deep_causality_algorithms)* Added test coverage for SURD algo.
- *(deep_causality_algorithms)* Added test coverage for SURD algo.
