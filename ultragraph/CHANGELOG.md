# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.8.2](https://github.com/deepcausality-rs/deep_causality/compare/ultragraph-v0.8.1...ultragraph-v0.8.2) - 2025-07-14

### Other

- Updated UltraGraph README.md
- Improved docstring

## [0.8.1](https://github.com/marvin-hansen/deep_causality/compare/ultragraph-v0.8.0...ultragraph-v0.8.1) - 2025-07-10

### Other

- Improved perf of deduplication in brandes_bfs_and_path_counting
- Improved test coverage
- Fixed breaking API by restring previous public re-export.
- Added test coverage for betweenness_centrality algo.
- Implemented Brandes' algorithm
- Refactored ultragraph's trait system and algo implementations for better maintainability.
- Updated README.md
- Formatting and linting.
- Improved test coverage.
- Added Tarjan's algorithm to UltraGraph.
- Improved test coverage for Dijkstra's algorithm. Improved docstring.
- Removed problematic check in Dijkstra's algorithm
- Added Dijkstra's algorithm to UltraGraph.
- Fixed a memory leak in the ultragraph benchmark.

## [0.5.6](https://github.com/marvin-hansen/deep_causality/compare/ultragraph-v0.5.5...ultragraph-v0.5.6) - 2025-06-19

### Other

- Merge pull request #201 from deepcausality-rs/release-plz-2025-04-03T05-42-30Z
- Bump criterion from 0.5 to 0.6.0

## [0.5.5](https://github.com/deepcausality-rs/deep_causality/compare/ultragraph-v0.5.4...ultragraph-v0.5.5) - 2025-05-16

### Other

- Fixed tests, benchmarks, and fixed some lints
- Bump rand from 0.8.5 to 0.9.0
- Bump petgraph from 0.6.6 to 0.7.0

## [0.5.4](https://github.com/deepcausality-rs/deep_causality/compare/ultragraph-v0.5.3...ultragraph-v0.5.4) - 2024-11-24

### Other

- Bumped up minimum rust version to 1.80.

## [0.5.3](https://github.com/deepcausality-rs/deep_causality/compare/ultragraph-v0.5.2...ultragraph-v0.5.3) - 2024-11-21

### Other

- update Cargo.lock dependencies

## [0.5.2](https://github.com/deepcausality-rs/deep_causality/compare/ultragraph-v0.5.1...ultragraph-v0.5.2) - 2024-01-14

### Other
- Updated dependencies in Ultragraph.
- Updated dependencies in ultragraph crate.

## [0.5.1](https://github.com/deepcausality-rs/deep_causality/compare/ultragraph-v0.5.0...ultragraph-v0.5.1) - 2023-09-06

### Other
- Fixed various linting issues.

## [0.5.0](https://github.com/deepcausality-rs/deep_causality/compare/ultragraph-v0.4.6...ultragraph-v0.5.0) - 2023-08-30

### Other
- Switched to shorter version numbers in Cargo.toml
- Fixed linter issue
- Formatted entire code base with rustfmt.
- Separate ultragraph type implementation imn multiple files.
- Added convince constructor to error
- Fixed broke test  for node remove error.
- Added missing none checks to graph_root_tests.rs
- Fixed fallibility in shortest path.
- Separate ultragraph tests into multiple files.
- Renamed example code files to prevent output file name collision.
- Updated copyright in all source and bash script files.
- Updated copyright in all licence files.

## [0.4.6](https://github.com/deepcausality-rs/deep_causality/compare/ultragraph-v0.4.5...ultragraph-v0.4.6) - 2023-08-17

### Other

- Added remaining tests to ultragraph
- updated tests in ultragraph.
- Reduced benchmark graph size to decrease CI runtime.
