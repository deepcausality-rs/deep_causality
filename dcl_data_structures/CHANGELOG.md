# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.8.0](https://github.com/deepcausality-rs/deep_causality/compare/dcl_data_structures-v0.7.0...dcl_data_structures-v0.8.0) - 2025-03-19

### Other

- Applied a ton of lints.
- Fixed tests, benchmarks, and fixed some lints
- Bump rand from 0.8.5 to 0.9.0
- Code formatting and linting
- Added or updated tests for all atomic sequence implementations.
- Code formatting
- Fixed tests and benches
- made get_min_cursor_sequence generic
- made get_min_cursor_sequence generic
- Added Atomic sequence trait
- Added uniform batch sizes to all ring_buffer benchmarks

## [0.7.0](https://github.com/deepcausality-rs/deep_causality/compare/dcl_data_structures-v0.6.1...dcl_data_structures-v0.7.0) - 2024-11-26

### Other

- Fixed import in tests
- Fixed ArrayGrid tests and benchmark.
- Fixed module declaration
- Added test coverage for point_index
- Merge branch 'deepcausality-rs:main' into main
- Added or updated example code for dcl_data_structures
- Fixed rewind bug in unsafe_storage_array.
- Added more tests to window type
- Fixed rewind bug in ArrayStorage

## [0.6.1](https://github.com/deepcausality-rs/deep_causality/compare/dcl_data_structures-v0.6.0...dcl_data_structures-v0.6.1) - 2024-11-25

### Other

- code formatting,
- Updated RingBuffer example code documentation and Readme.
- Added examples for the ring_buffer in different configurations.
- Merged tests into one file
- Updated README.md

## [0.6.0](https://github.com/deepcausality-rs/deep_causality/compare/dcl_data_structures-v0.5.0...dcl_data_structures-v0.6.0) - 2024-11-24

### Other

- Updated README.md
- Applied various code lints.
- Added more tests to DSL.
- Added test
- Improved test coverage for BatchEventProcessor
- Fixed bench_dcl_data_structures.rs
- Added test coverage of DSL in RingBuffer
- Improved test coverage of RingBuffer type
- Improved test coverage of ArrayGrid type
- Rolled back breaking changes
- Update dcl_data_structures/tests/ring_buffer/wait_strategy/spinloop_wait_strategy_tests.rs
- Update dcl_data_structures/tests/ring_buffer/producer/multi_producer_tests.rs
- Update dcl_data_structures/src/ring_buffer/utils/bit_map.rs
- Code linting
- Code format
- Updated README.md
- Documented DSL in RingBuffer
- Added DSl to RinglBuffer
- Added documentation to RingBuffer
- Added benchmark and various tweaks to ring buffer
- Added documentation to single and multi producer.
- Added single and multi producer with tests
- Added documentation and tests to consumer / BatchEventProcessor
- Added consumer to ring buffer
- Added documentation, example, and tests to executor in ring buffer
- Added executor to ring buffer
- Added tests for wait strategy in ring_buffer
- Added documentation to sequence in ring_buffer
- Code forma!ing
- Added tests to barrier in ring_buffer
- Added documentation to barrier in ring_buffer
- Added wait strategy to rin_buffer
- Fixed missing import
- Code linting and formatting
- Added benchmark and tests for atomic_sequence.
- Added tests for atomic_sequence
- Added tests for the custom rin_buffer
- Added initial work on a custom ring_buffer implementation
- Code linting & formatting
- Bumped up minimum rust version to 1.80.
- Code formatting
- Updated Readme in data structure crate with the latest benchmark results.
- Improve performance of unsafe ArrayGrid further
- # Optimize Unsafe ArrayGrid Implementation
- Code formatted
- Updated README files in dcl_data_structures crate
- Added more tests to  unsafe ArrayGrid.
- Added unsafe ArrayGrid Type that performs up tp 50% faster than the safe counter part
- Added some more tests
- Code formatting

## [0.5.0](https://github.com/deepcausality-rs/deep_causality/compare/dcl_data_structures-v0.4.7...dcl_data_structures-v0.5.0) - 2024-11-21

### Other

- Increased test coverage of UnsafeArrayStorage
- Code formatting
- Increased test coverage for UnsafeVectorStorage and fixed a but on rewind.
- Increased test coverage of UnsafeArrayStorage
- Increased test coverage of UnsafeArrayStorage
- Increased test coverage of UnsafeArrayStorage
- Increased test coverage of VectorStorage
- Fixed failing benches
- Increases test coverage
- Increases test coverage
- Increases test coverage
- Fixed missing import in bench
- Fixed CI workflow
- Bunch of code lints
- Enabled unsafe feature on CI
- Applied code lints
- Code formatting
- For new_with_unsafe_vector_storage:
- Added proper feature gating to disable unsafe impl for the dcl_data_structures crate
- Updated docs for dcl_data_structures
- Updated docs for dcl_data_structures
- Optimizes UnsafeVectorStorage.
- Added initial, non-opt, UnsafeVectorStorage implementation for sliding window.
- Added documentation to UnsafeArrayStorage
- Opimized UnsafeArrayStorage futher.
- Added benchmark for UnsafeArrayStorage
- Added initial support for unsafe sliding window storage implementation.
- Improved the performance of the sliding window implementation in dcl_data_structures.

## [0.4.7](https://github.com/deepcausality-rs/deep_causality/compare/dcl_data_structures-v0.4.6...dcl_data_structures-v0.4.7) - 2024-01-14

### Other
- Reduced multiplier in vec push benchmark to address long running benchmark for DCL data structures
- Fixed linting and formatting in tests.

## [0.4.6](https://github.com/deepcausality-rs/deep_causality/compare/dcl_data_structures-v0.4.5...dcl_data_structures-v0.4.6) - 2023-08-30

### Other
- Formatted entire code base with rustfmt.
- Updated Readme in data structure crate.
- Updated README.md with links to sub-crates.
- Updated copyright in all source and bash script files.
- Updated copyright in all licence files.
