# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.0](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality-v0.4.0...deep_causality-v0.5.0) - 2023-08-17

### Other
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
- Moved remaining reasoning methods from CausaloidGraph into the default implementation in causable_graph_explaining protocol
- Moved remaining explain methods from CausaloidGraph into the default implementation in causable_graph_explaining protocol
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