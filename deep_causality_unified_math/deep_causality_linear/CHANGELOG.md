# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_linear-v0.1.2...deep_causality_linear-v0.2.0) - 2026-09-01

### Fixed

- *(unified_math)* [**breaking**] act on a review of the HKT law and constraint work
- *(unified_math)* [**breaking**] rectify the HKT law tests and correct the defects they exposed
- *(deep_causality_unified_math)* Applied a number of fixes and lint corrections.

### Other

- *(bazel)* compile the 71 tests Cargo ran and Bazel never saw
- *(cargo)* hoist every dependency into [workspace.dependencies]
- *(bazel)* merge test BUILD files and derive deps from Cargo
- *(deep_causality_haft)* [**breaking**] remove the HKT Constraint system and the Satisfies marker
- *(openspec)* Updated paths in docs of relocated crates across the repo
- *(openspec)* Updated inbound links to archived notes
