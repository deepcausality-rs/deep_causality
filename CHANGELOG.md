# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.7.5](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality-v0.7.4...deep_causality-v0.7.5) - 2025-06-19

### Added

- *(deep_causality)* Updated README.md

### Other

- Merge remote-tracking branch 'origin/main'
- Removed legacy docs folder.
- Moved examples into root folder.
- Updated Readme with Discord and various other project links.
- Fixed missing badges in README.md
- Bump criterion from 0.5 to 0.6.0
- Bump parquet to 55.1.0
- Bump parquet to 55.1.0
- Updated scale in linear graph bench util
- Add comprehensive documentation to CSM CausalAction type and CausalState type
- Merge remote-tracking branch 'origin/main'
- Add comprehensive documentation to CSM struct

## [0.7.5](https://github.com/marvin-hansen/deep_causality/compare/deep_causality-v0.7.4...deep_causality-v0.7.5) - 2025-06-19

### Added

- *(deep_causality)* Updated README.md

### Other

- Merge remote-tracking branch 'origin/main'
- Removed legacy docs folder.
- Moved examples into root folder.
- Updated Readme with Discord and various other project links.
- Fixed missing badges in README.md
- Bump criterion from 0.5 to 0.6.0
- Bump parquet to 55.1.0
- Bump parquet to 55.1.0
- Updated scale in linear graph bench util
- Add comprehensive documentation to CSM CausalAction type and CausalState type
- Merge remote-tracking branch 'origin/main'
- Add comprehensive documentation to CSM struct

## [0.7.4](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality-v0.7.3...deep_causality-v0.7.4) - 2025-05-16

### Other

- Bump parquet from 54.2.1 to 54.3.1
- Applied a ton of lints.
- Fixed tests, benchmarks, and fixed some lints

## [0.7.3](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality-v0.7.2...deep_causality-v0.7.3) - 2024-11-26

### Other

- updated the following local packages: dcl_data_structures

## [0.7.2](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality-v0.7.1...deep_causality-v0.7.2) - 2024-11-24

### Other

- Added initial work on a custom ring_buffer implementation
- Updated alias types
- Code linting & formatting
- Removed lifetimes from context types.
- Bumped up minimum rust version to 1.80.
- Code formatting
- Update error tests
- Refactored Error module in deep_causality and added test coverage.

### Removed

- removed phantom marker

## [0.7.1](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality-v0.7.0...deep_causality-v0.7.1) - 2024-11-21

### Other

- updated the following local packages: dcl_data_structures

## [0.7.0](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality-v0.6.5...deep_causality-v0.7.0) - 2024-01-25

### Other
- Replaced Cell types with Arc/RwLock to make interior mutability thread safe.

## [0.6.5](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality-v0.6.4...deep_causality-v0.6.5) - 2024-01-25

### Other
- Narrowed down overly general type constrains on generic type parameters.

## [0.6.4](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality-v0.6.3...deep_causality-v0.6.4) - 2024-01-25

### Other
- Merge branch 'deepcausality-rs:main' into main
- Documented all protocols in deep causality.
- Code formatting & linting.
- Implemented Indexable trait for Context
- Added Indexable protocol to deep causality crate

## [0.6.3](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality-v0.6.2...deep_causality-v0.6.3) - 2024-01-14

### Other
- Moved example folder back to deep_causality folder due to cargo config errors.
- Flattened folder structure.
- Updated dependencies to latest version.
- Update parquet requirement from 48 to 49
- Restricted ctx example dependency to mitigate yanked sub sub dependency warning.
- Removed pointless tests that only threw clippy linting errors.
- Added generic sum util with tests.
- Disabled unused re-exports in prelude.
- Updated examples to latest DC version.

## [0.6.2](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality-v0.6.1...deep_causality-v0.6.2) - 2023-09-19

### Fixed
- fixed failing test.

### Other
- Updated starter example.
- Added starter code example
- Removed unused import
- Added missing error message
- Created new starter example
- Restored macros in causable extension
- Fixed missing test for Identifiable in AdjustableSpace.
- Added error case tests to adjustable space time tests
- Added custom is_empty implementation to test if codecov recolonize it.
- Added more tests for adjustable types.
- Added more tests for adjustable types.
- Added more tests for adjustable types.
- Added more error tests to drive up code coverage.
- Merge remote-tracking branch 'origin/main'
- Trying phylum gh action again.

## [0.6.1](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality-v0.6.0...deep_causality-v0.6.1) - 2023-09-08

### Other
- Added unset method to extendable_contextuable_graph.rs

## [0.6.0](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality-v0.5.0...deep_causality-v0.6.0) - 2023-09-06

### Other
- Tested multiple contexts implementation.
- Finalized multiple contexts implementation.
- Implemented multiple contexts.
- Working on multiple contexts.
- Implemented initial support for multiple contexts.
- Added field extra_context to Context.
- Removed PhantomData marker in Context type since all generic parameters are bound.
- Renamed type alias.
- Renamed node structs to something more sensible and intuitive.
- Moved slides into main doc folder.
- Fixed various linting issues.
- Removed old swift notebook.
- Moved all documentation to project website.
- Fixed a test

## [0.5.0](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality-v0.4.0...deep_causality-v0.5.0) - 2023-08-30

### Other
- Fixed type aliases in Causaloid getters.
- Simplified complex types with type alias.
- Fixed tests to run with latest commit in main.
- Fixed type signatures in benchmarks to run with latest commit in main.
- Made Spatial and Temporable trait generic. Resolves issue 42.
- Switched to shorter version numbers in Cargo.toml
- Merge remote-tracking branch 'origin/main'
- Update parquet requirement from 45.0 to 46.0
- Added more corner case testing causal graph explaining.
- Added more corner case testing causal graph reasoning.
- Added tests for error handing in graph reasoning.
- Updated graph reasoning protocol with more error handling and removed dead code.
- Added missing error handing tests to causal graph reasoning.
- Fixed several broken links ik documentation.
- Code formatting.
- Added tests to adjustable protocol.
- Added default constructor to custom errors.
- Code formatting.
- Code formatting.
- Update tests to match generated getters.
- Updated multiple types to use macros to generate constructors and getters.
- Updated adjustable types in deep causality to use macros to generate constructor and getters.
- Added more tests to CSM types.
- Moved all test utils into test folder. Updated import path in affected tests.
- Moved all benchmark utils into benchmark folder.
- Removed benchmark and test utils from src/utils folder.
- Reformatted Adjustable protocol.
- Update AdjustableTime
- Added tests for adjustable time type.
- Added AdjustableTime type.
- Updated prelude to export new AdjustableData type.
- Misc minor changes.
- Removed PropagationError together with propagation method in adjustable protocol.
- Updated adjustable protocol for generic usage and uniform signature.
- Reorganized context tests to mirror folder structure in src folder.
- Added tests for adjustable data type
- Added adjustable data type that implements adjustable protocol.
- Updated copyright in all source and bash script files.
- Updated copyright in all licence files.
- Update mod.rs
- Uncomment adjustable trait with a notice that it needs review and an actual implementation with tests.
- Updated deep causality code examples to use latest version.

## [0.4.0](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality-v0.3.1...deep_causality-v0.4.0) - 2023-08-17

### Other

- Fixed broken benchmark.
- Separated context graph type into multiple files.
- Moved contextoid_type into contextoid folder.
- Moved root type into node types folder
- Separated Contextoid type into multiple files.
- Update mode files and prelude
- Separated Observation type into multiple files.
- Separated inference type into multiple files.
- Separated causaloid graph type into multiple files.
- Separated causaloid type into multiple files.
- Separated assumption type into multiple files.
- Removed unnecessary trait bounds in context type.
- updated tests in ultragraph.
- Reduced benchmark graph size to decrease CI runtime.
- Updated causal and context graph to use new ultragraph type alias.
- Merge branch 'deepcausality-rs:main' into main
- Fixed typo in referenced author's name.
- Added recent presentation files to docs.
- Fixed broken import.
- Moved reasoning utils to protocol.
- Limited visibility of internal type aliases to pub(crate)
- renamed some files.
- Moved remaining reasoning methods from CausaloidGraph into the default implementation in causable_graph_explaining
  protocol
- Moved remaining explain methods from CausaloidGraph into the default implementation in causable_graph_explaining
  protocol
- Added test for get_graph. Reorganized graph reasoning tests.
- Updated code documentation of CausaloidGraph
- Fixed a bunch of linter errors, re-added default implementation to CausaloidGraph and updated tests.
- Merge branch 'main' into main
- Updated protocol documentation.
- Updated imports, paths, and tests.
- Moved type aliases into causable graph protocol and made them public
- refactored reasoning utils into a shared module
- Added default implementation to CausableGraph and CausableGraphReasoning traits.
- Updated imports and prelude
- Moved traits CausableGraph and CausableGraphReasoning into two seperate
- code formatting
- Made CausableGraphReasoning trait a sub-trait of CausableGraph
- Moved utils into seperate causaloid utils file to declutter causaloid graph implementation.
- Updated documentation
- Updated documentation
- Updated documentation
- Code formatting of protocols.
- Code formatting of protocols.
- Added documentation to type extensions.
- Added tests for causable vec deque
- Reorganizing causable extension type tests.
- Added tests for causable Btree map.
- Updated causable protocol and type extension
- Reorganizing causable extension type tests.
- Added tests for inferable VecDeque.
- Added tests for inferable VecDeque.
- Added inferable type extension for BTreeMap and VecDeque
- Reorganizing inferable tests.
- Added tests for VecDeque observable.
- Added bree map tests to observable.
- Reorganizing observable tests.
- Removed Clone trait requirement from Assumable
- Code formatting
- Added assumable tests for VecDeque
- Updated assumable tests for BTreeMap
- Updated assumable tests for array and hashmap
- Added assumable type extension for BTreeMap, HashSet, and BTreeSet
- Reorganizing assumable tests.
- Added Readme to DTX example.
- Updated dependencies in Cargo.toml for DTX example
- Updated run method in DTX example
- Added utils to DTX example
- Added file reader to DTX example
- Added data types to DTX example
- Added config types to DTX example
- Added data to dtx example
- Added run method to dtx example
- Started working on new example dtx: Dynamic Context.
- Added min rust version to examples
- Updated ctx example dependencies in Cargo.toml
- Renamed file
- Updated SPDX-License-Identifier to GFM comment to prevent rendering meta data as table.
- Updated copyright with SPDX-License code.
- Added SPDX-License-Identifier to all docs
  [//]: # (---)
  [//]: # (SPDX-License-Identifier: MIT)
  [//]: # (---)

# What's New

## [0.2.4](https://github.com/deepcausality-rs/deep_causality/releases/tag/0.2.4) (2023-07-10)

> Description

Feature update

### Upgrade Steps

* Updated version in Cargo.toml

### Breaking Changes

* None

### New Features

* Added Causal State Machine (CSM)
* Updated code example & tests

### Bug Fixes

* None

### Performance Improvements

* None

### Other Changes

* Added tests for Causal State Machine