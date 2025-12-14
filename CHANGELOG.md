# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.12.2](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality-v0.12.1...deep_causality-v0.12.2) - 2025-12-14

### Other

- *(deep_causality)* Increased test coverage.

## [0.12.1](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality-v0.12.0...deep_causality-v0.12.1) - 2025-12-12

### Other

- updated the following local packages: deep_causality_core

## [0.12.0](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality-v0.11.11...deep_causality-v0.12.0) - 2025-12-12

### Added

- *(deep_causality)* Updated extension tests to new API.
- *(deep_causality)* Updated extension tests to new API.
- *(deep_causality)* Updated Error tests to new API.
- *(deep_causality_core)* Removed unrelated types.
- *(deep_causality)* Removed leftover legacy types that were replaced by new core crate
- *(deep_causality)* Updated benchmarks for using functional core.
- *(deep_causality)* Initial re-write using deep_causality_core crate for functional core.

### Fixed

- fixed a number of Bazel config files.
- *(deep_causality)* Simplified type system across the entire crate further.
- *(deep_causality)* Refactored Causaloid and replaced the previous complex implicit context with an explicit context passed through the EffectPropagtingProcess. Reduced generic type parameters from 9 to 4.
- *(deep_causality)* Restored proper fn pointers in CausalFn and ContextualCausalFn.

### Other

- Added a few more medical examples.
- Added or updated documentation.
- Updated Project README.md
- Updated criterion across the repo.
- Reorganized and updated repo wide examples.
- *(deep_causality)* Lints, fixes and code improvements.
- *(deep_causality)* Improved test coverage.
- *(deep_causality)* Improved test coverage.
- *(deep_causality)* Improved test coverage.
- *(deep_causality)* Improved test coverage.
- *(deep_causality)* Lints, fixes and code improvements.
- *(deep_causality)* Updated Bazel test config
- *(deep_causality)* Updated more tests to new API.
- *(deep_causality)* Updated more tests to new API.

## [0.11.11](https://github.com/marvin-hansen/deep_causality/compare/deep_causality-v0.11.10...deep_causality-v0.11.11) - 2025-12-03

### Added

- *(deep_causality_sparse)* Fixing auto-release
- *(deep_causality_sparse)* Implemented initial CsrMatrix types.
- *(deep_causality)* Moved LogAppend trait into haft crate.

### Other

- Regenerated SBOM.
- Updated dev dependencies across the repo.
- Updated Dev dependencies.
- Merge branch 'deepcausality-rs:main' into main
- Merge remote-tracking branch 'origin/main'
- Restored manually generated SBOM to restore Dependency and licence scan.

## [0.11.10](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality-v0.11.9...deep_causality-v0.11.10) - 2025-11-23

### Other

- Merge branch 'deepcausality-rs:main' into main

### Removed

- removed all manually generated SBOM files

## [0.11.9](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality-v0.11.8...deep_causality-v0.11.9) - 2025-11-23

### Added

- *(deep_causality)* Complete HKT generative system with automatic audit logging.
- *(deep_causality_multivector)* Added implementation of multivector crate.
- *(deep_causality)* Added new example of Structural Causal Model with Monadic Composition
- *(deep_causality)* Added new intervenable trait and a default impl for CausalMonad.
- *(deep_causality)* Refactored all monadic types into a dedicated module
- *(deep_causality)* Updated Bazel test config deps.
- *(deep_causality)* Updated EPP context example to new API.
- *(deep_causality)* Updated RCM example to new API.
- *(deep_causality)* Renamed EffectValue::Deterministic to EffectValue::Boolean
- *(deep_causality)* Updated Tokio example to new API.
- *(deep_causality)* refactored EffectValue
- *(deep_causality)* Minor refactoring
- *(deep_causality)* Simplified causable implementation for Causaloid.
- *(deep_causality)* Removed Registry as it was a design mistake that prevented HKT's over collections and introduced dynamic trait objects.
- *(deep_causality)* Updated Causable implementation.
- *(deep_causality)* Simplified causaloid from_causal_collection API. Updated relared tests and benchmarks.
- *(deep_causality)* Added structured causal function logging.
- *(deep_causality)* Introduce fluent monadic API and guarantee lo
- *(deep_causality)* Added External variant to the EffectValue to support external types as propagating value.
- *(deep_causality)* Updated ContextualLink variant of EffectValue
- *(deep_causality)* renamed files
- *(deep_causality)* Partial rewrite towards type based effect programming.
- *(deep_causality)* Updated Causaloid to better handle default reasoning of collections.
- *(deep_causality)* Code cleanup.
- *(deep_causality)* Migrated causal collection reasoning towards monadic composition.
- *(deep_causality)* Prepared migation of causal collection reasoning.
- *(deep_causality)* Prepared migation of causal collection reasoning.
- *(deep_causality)* Updated MonadicCausableGraphReasoning to use proper causal reasoning. Removed mock implementation. Updated Causable impl for Causaloid to use monadic bind.
- *(deep_causality)* made PropagatingEffect use CausalMonad pure.
- *(deep_causality)* reworking causal reasoning.
- *(deep_causality)* Refactor PropagatingEffect and Causaloid.
- *(deep_causality)* Prepared HKT migration
- *(deep_causality)* Removed effect field from Causaloid
- *(deep_causality)* Implemented MonadicCausable for Causaloid
- *(deep_causality)* Added explain to new PropagatingEffect.
- *(deep_causality)* Added MonadicCausable trait and new function aliases
- *(deep_causality)* Added CausalMonad, CausalEffectSystem types.
- *(deep_causality)* Added CausalMonad, CausalEffectSystem types.
- *(deep_causality)* Added new types: causal_value,  effect_log, and numeric_value.

### Other

- Updated pre-specs and updated Bazel config
- Updted README.md
- *(deep_causality)* Increased test coverage and applied lints.
- *(deep_causality)* Increased test coverage.
- *(deep_causality)* Increased test coverage.
- *(deep_causality)* Increased test coverage.
- *(deep_causality)* Minor lints and code tweaks.
- *(deep_causality)* Increased test coverage.
- *(deep_causality)* Increased test coverage.
- *(deep_causality)* Lints and formatting.
- *(deep_causality)* Lints and formatting. Improved error handling.
- *(deep_causality)* Added benchmarks for Causal Monad.
- *(deep_causality)* Added tests for Causal Monad.
- *(deep_causality)* Added tests for Causal Monad.
- *(deep_causality)* Minor lint.
- *(deep_causality)* Increased test coverage. Fixed tests affected by updated EffectValue.
- *(deep_causality)* Minor lint.
- *(deep_causality)* Minor lint.
- *(deep_causality)* Minor lints.
- *(deep_causality)* Minor lint.
- *(deep_causality)* Increased test coverage. Minor fixes and lints.
- Code formatting.
- *(deep_causality)* Increased test coverage.
- *(deep_causality)* Increased test coverage.
- Code formatting.
- *(deep_causality)* Increased test coverage.
- *(deep_causality)* Increased test coverage.
- *(deep_causality)* Increased test coverage.
- Code formatting.
- Merge branch 'deepcausality-rs:main' into main
- *(deep_causality)* Removed generative tests for the time being. Updated Bazel test config.
- *(deep_causality)* Updated more tests to new API and re-write missing tests.
- *(deep_causality)* Updated more tests to new API and re-write missing tests.
- *(deep_causality)* Updated more tests to new API and re-write missing tests.
- *(deep_causality)* Updated more tests to new API and re-write missing tests.
- *(deep_causality)* Added or updated documentation for new monadic types.
- *(deep_causality)* Updated more tests to new API and re-write missing tests.
- *(deep_causality)* Updated more tests to new API and re-write missing tests.
- *(deep_causality)* Updated more tests to new API and re-write missing tests.
- *(deep_causality)* Updated more tests to new API and re-write missing tests. Re-organized test utils.
- *(deep_causality)* Updated more tests to new API and re-write missing tests. Re-organized test utils.
- *(deep_causality)* Updated more tests to new API and re-write missing tests. Re-organized test utils.
- *(deep_causality)* Updated more tests to new API and re-write missing tests.
- Added new pre-specs for Multivector and Octonion
- *(deep_causality)* Updated more tests to new API and re-write missing tests.
- *(deep_causality)* Updated more tests to new API and re-write missing tests.
- *(deep_causality)* Updated more tests to new API and re-write missing tests.
- *(deep_causality)* Updated more tests to new API and re-write missing tests.
- *(deep_causality)* Updated more tests to new API and re-write missing tests.
- *(deep_causality)* Updated more tests to new API and re-write missing tests.
- *(deep_causality)* Updated more tests to new API and re-write missing tests.
- *(deep_causality)* Updated more tests to new API and re-write missing tests.
- *(deep_causality)* Updated more tests to new API.
- *(deep_causality)* Updated more tests to new API.
- *(deep_causality)* Updated causaloids tests to the new API
- *(deep_causality)* Updated causaloids tests to the new API
- *(deep_causality)* Updated utils tests to the new API
- *(deep_causality)* Updated more tests to the new API
- *(deep_causality)* Updated extension test to the new API
- *(deep_causality)* Updated the first extension test to the new API
- *(deep_causality)* Updated docstring and logs
- *(deep_causality)* Updated test utils and benchmark to new API.
- *(deep_causality)* Migrated tests of causal collection reasoning to use new API.
- *(deep_causality)* Improved docstring
- Migrated graph reasoning methods to monadic trait.
- Prepared migration of graph causal reasoning to monadic equivalent.
- *(deep_causality)* working on restoring tests for causal collection extensions.
- Added PhantomData to Causaloid
- updated re-export via lib.rs

## [0.11.8](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality-v0.11.7...deep_causality-v0.11.8) - 2025-11-05

### Added

- *(deep_causality_uncertain)* Migrated internal compute graph to ConsTree from deep_causality_ast crate.
- *(ast)* Add deep_causality_ast crate with persistent tree

### Other

- Updated SBOM for all crates.
- Merge branch 'deepcausality-rs:main' into 008-hkt-uncertain-specs
- updated AGENTS.md and README.md

## [0.11.7](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality-v0.11.6...deep_causality-v0.11.7) - 2025-10-31

### Added

- *(deep_causality)* Added ComplexTensor to PropagatingEffect.
- *(deep_causality_tensor)* Added TensorProduct
- *(deep_causality)* Reorganized aliases. Improved public re-export. Fixed imports.
- *(deep_causality)* Added constructors to propagating_effect. Added tests.
- *(deep_causality)* Added MaybeUncertain and CausalTensor to PropagatingEffect. Updated tests.

### Other

- Updates examples.
- Refactored Causaloid context into Arc<RwLock>
- *(deep_causality_num)* minor fixes.
- *(deep_causality_num)* minor fixes.
- *(deep_causality_num)* increased test coverage.
- *(deep_causality_uncertain)* Increased test coverage.
- *(deep_causality)* Increased test coverage.
- Added and updated type aliases.

## [0.11.6](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality-v0.11.5...deep_causality-v0.11.6) - 2025-10-19

### Other

- Updated project README.md
- Added FOSA batches to README.md

## [0.11.5](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality-v0.11.4...deep_causality-v0.11.5) - 2025-09-25

### Other

- Updated SBOM for all crates.
- Updated Project README.md
- Updated SBOM for all crates.

## [0.11.4](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality-v0.11.3...deep_causality-v0.11.4) - 2025-09-22

### Fixed

- *(deep_causality)* Fixed incorrect trait bound for BTreeMap
- *(deep_causality)* Fixed incorrect test logic.
- *(deep_causality)* Fixed historic bug in  type extensions.
- *(deep_causality)* Removed last internal macros and removed dependency on deep_causality_macro crate.

### Other

- *(deep_causality)* Improved test coverage for extensions.
- *(deep_causality)* Improved test coverage for extensions.
- *(deep_causality)* Improved test coverage for extensions.
- *(deep_causality)* Improved test coverage for extensions.
- *(deep_causality)* Improved test coverage for extensions.
- *(deep_causality)* Improved test coverage for extensions.
- *(deep_causality)* Added more tests for extensions.
- Updated SBOM script to generate hash signature together with the SBOM.

## [0.11.3](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality-v0.11.2...deep_causality-v0.11.3) - 2025-09-21

### Added

- *(deep_causality_tensor)* Initial setup. Moved CausalTensor type from the data_structure crate into dedicated deep_causality_tensor crate.
- *(deep_causality_num)* Initial implementation. Update of all downstream crates.
- *(deep_causality_algorithms)* Implement mRMR (FCQ variant) feature selector

### Other

- Updated Project README.md
- Updated Project README.md
- Updated Cargo configuration and feature propagation across the entire repo.
- Lots of lints and formats. Updated MSRV to 1.89 and edition 2024 across the entire repo.
- *(deep_causality_rand)* Increased test coverage.

## [0.11.2](https://github.com/marvin-hansen/deep_causality/compare/deep_causality-v0.11.1...deep_causality-v0.11.2) - 2025-09-15

### Added

- *(deep_causality_data_structures)* Added identity types. Removed num_traits dependency.

### Other

- *(deep_causality_algorithms)* Added test coverage for SURD algo.

## [0.11.1](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality-v0.11.0...deep_causality-v0.11.1) - 2025-09-11

### Added

- *(deep_causality)* fixed wrong README

### Other

- trying to fix #315
- trying to fix #315
- trying to fix #315
- trying to fix #315

## [0.10.0](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality-v0.9.1...deep_causality-v0.10.0) - 2025-09-08

### Added

- *(deep_causality_macros)* removed overly complex constructure macro.
- *(deep_causality)* Integrated Uncertain<T> into CSM (Causal State Machine)
- *(deep_causality)* refactored CSM types and removed unnecessary macro usage.
- *(deep_causality)* integrated Uncertain<T> into Deontic Reasoning and EffectEthos
- *(deep_causality)* Updated PartialEq for PropagatingEffect
- *(deep_causality)* Added reasoning over uncertainty i.e. probability distributions to causal collections.
- *(deep_causality)* Add uncertain data types and tests to DeepCausality

### Other

- Increased test coverage across all crates.
- Increased test coverage across all crates.
- Updated project wide Bazel config.
- Fixed formatting.
- *(deep_causality)* Added tests for new variants of PropagatingEffect.

## [0.9.0](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality-v0.8.2...deep_causality-v0.9.0) - 2025-08-27

### Added

- *(deep_causality)* Added new example for Effect Ethos with CSM.
- *(deep_causality)* Integrated Effect Ethos with CSM
- *(deep_causality)* Added explanation of the verdict to the effect ethos.
- *(deep_causality)* Implement core Telos framework for deontic inference

### Fixed

- *(deep_causality)* fixed mapping maintained by EffectEthos
- *(deep_causality)* fixed incorrect test.

### Other

- *(deep_causality)* updated TagIndex to use a HashSet to prevent duplicate tags.
- *(deep_causality)* promoted CSM modules from files into directories.
- *(deep_causality)* Improved test coverage
- *(deep_causality)* Improved test coverage
- *(deep_causality)* Improved test coverage
- *(deep_causality)* Decluttered tests for EffectEthos
- *(deep_causality)* Improved test coverage
- Updated Bazel config.
- Updated Docstring.
- Formats and lints.
- *(deep_causality)* Added more tests to test new EffectEthos.
- Updated Effect Ethos type and added a new high level API
- *(deep_causality)* Added tests for new effect ethos and teloid types.
- Reworked Teloid, TeloidStore, and ProposedAction types
- Implemented ActionParameterValue and ProposedAction type
- Removed duplicate copyright note
- Added TeloidStore
- Added telos types tag_index and teloid_modal. Added unit tests. Updated primitive aliases. Updated Bazel test config.

## [0.8.2](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality-v0.8.1...deep_causality-v0.8.2) - 2025-08-08

### Added

- *(deep_causality)* Implemented Adaptive Reasoning #282
- *(deep_causality)* Added Configurable Reasoning Modalities to Causal Collectins.
- *(deep_causality)* Added Programmatic Verification of Model Assumptions #275
- *(deep_causality)* Unified Evidence and PropagatingEffect into a Single Typ #273

### Other

- Rolled back manual version setting.
- Bumped up deep_causality version for auto-release
- Updated copyright in Cargo.toml fils
- Added unit tests for graph indexable data in context graph.
- added data indexable trait and default implementation for context graph.
- Fixed failing test
- Added code example for Rubin Causal Model.
- Removed overly strict trait constraints from Dataoid/ Data context type.
- Enhanced graph reasoning to handle `RelayTo` target validation. Updated error handling for non-existent nodes and added corresponding unit test.
- Fixed typo
- Added tests for adaptive reasoning.
- Removed duplicate license header
- Updated test utils to enhance error handling for non-deterministic effects; added unit test for erroneous singleton evaluation.
- Added unit tests for deterministic propagation in empty causable collections, including explain method validation.
- Updated tests for deterministic input-output causaloids. Updated test utils. Improved causality graph evaluation logical clarity.
- Enhanced graph reasoning: introduced proper effect propagation logic and improved documentation clarity in causable graph methods.
- Simplified evaluate in causable graph
- Limited access scope for private helper method in causable collection
- Refactored module names in causable_collection and causable_graph traits for improved clarity and consistency.
- Removed dead code
- Refactored CausableReasoning into modular CausableCollection traits. Enhanced structure and accessibility for causable collections.
- Removed unused files
- Improved code organization for CausableReasoning trait. Secured private impl's by limiting scope.
- Improved test coverage; renamed for simpler name convention.
- Added or improved test coverage for causable reasoning for all causable collections.
- reworked evaluate_probabilistic_propagation and evaluate_mixed_propagation in causable_reasoning.rs
- Working on evaluate_mixed_propagation
- Removed halting variant from PropagatingEffect
- Simplified causable_reasoning
- Separated debug and display trait impl for Assumption type.
- Finalized Programmatic Verification of Model Assumptions #275
- Restored previous implementation of causable_reasoning.rs
- Added empty test to Assumption vector.
- Improved test coverage for Assumption.
- Improved test coverage for Model.
- Added tests for AggregateLogic
- Improved AssumptionError and its testing.
- Working on  Configurable Reasoning Modalitie #274
- Updated CausableReasoning trait to handle RelayTo variant to dispatch to a different causaloid.
- Increased test coverage of PropagatingEffect
- Lints and formats
- Increased test coverage of PropagatingEffect
- Increased test coverage of PropagatingEffect
- Updated benchmarks to use new PropagatingEffect.
- Removed unused Evidence type.
- Format and lints
- Derived Default of PropagatingEffect instead of custom impl.
- Bump criterion from 0.6.0 to 0.7.0
- Linting and formatting.
- Added black_box to evaluate_single_cause to ensure no fluke can ever happen.
- Updated benchmark code with minor fix to evaluate_single_cause
- Improved benchmark code.
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.8.0](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality-v0.7.5...deep_causality-v0.8.0) - 2025-07-08

### Added

- *(generative)* Implement full lifecycle management for contexts
- *(generative)* implement full CRUD for context and contextoids
- Implement and refine ModelBuildError tests

### Fixed

- fix(deep_causality) fixed benchmarks.
- *(deep_causality)* Improve NedSpace tests
- *(deep_causality)* Improve NedSpace tests
- *(ecef)* Commit remaining EcefSpace test changes
- *(euclidean)* Improve EuclideanSpace tests
- *(ecef)* Improve EcefSpace tests
- *(spacetime)* Correct TangentSpacetime position method and tests
- *(tests)* Correct Lorentzian spacetime interval tests

### Other

- Updated test util
- Updated test util
- Updated test util
- Linting
- Tested test utils...
- Linting
- Fix and linting
- Updated Bazel test config
- Re-organized context tests
- Re-organized test utils
- Re-organized trait tests
- Increased test coverage
- Increased test coverage
- Increased test coverage
- Increased test coverage
- Linting
- Increased test coverage
- Increased test coverage
- Increased test coverage
- Increased test coverage
- Removed active state from Causaloid and downstream usage.
- Lints
- Increased test coverage
- Increased test coverage
- Fixed failing test
- Increased test coverage
- Increased test coverage
- Fixed all remaining tests
- Fixed up all causality graph tests
- Fixed Causaloid tests
- Added or updated tests for reasoning types. Applied minor fixes to debug and display in Evidence type.
- Fixed tests for GenerativeProcessor
- Updated all tests for the collection type extensions
- Added or updated error tests
- Fixed benchmarks to use new API
- Updated CSM implementation to restore fully deterministic behavior for triggering actions.
- Implemented first version of unified reasoning.
- Removed prelude
- Added specs for unified reasoning
- Added specs for unified reasoning
- Fixed line inconsistencies in Cargo.toml
- Preparing for ultragraph release
- Removed benchmark badge from Readme
- Switched to central workspace dependencies.
- Set the version number of dependent internal crates to match deep_causality for simpler version management.
- Improve test coverage
- Improved test coverage and improved context API.
- Update the DeepCausality crate to use the refactored UltraGraph AP
- Adding more tests for coverage
- Adding more tests for coverage
- Adding more tests for coverage
- Working on tests
- Working on tests
- Working on tests
- Working on tests
- Removed assert comments
- Working on test coverage
- Added or updated documentation
- Increased test coverage for GenerativeProcessor
- Updated or added trait documentation.
- Increased test coverage of ExtendableContextuableGraph
- Improved test coverage of test utils
- Increased test coverage across the repo
- Deleted dead code
- Improved test coverage and fixed some lints.
- Re-organized model type tests
- Added tests for new model error types.
- Added tests for GenerativeOutput
- Fixed a gazillion Broken Links Locations in Rust Docs.
- Linting and formatting
- Added tests to  reasoning types
- Fixed module doc
- Moved script folder into build folder
- Updated README.md and Bazel.md docs
- Fixed up the Bazel config.
- Updated README.md
- Marked Bazel files as excluded from Cargo release to ensure these crates vendor well when used with Bazel.
- Added Bazel config for build and test
- Added first draft of Generative Function Traits, its implementation, and addition to the model type.
- Lints and formats..
- Working on generative function.
- Reworked ReasoningMode, Evidence and PropagatingEffect
- Added tests to GenerativeTrigger
- Merge remote-tracking branch 'origin/main'
- Updated prelude.rs
- Corrected brittle `test_display_trait` assertions in both
- Added documentation to alias module
- Fixed circular dependency issue in alias types.
- Added tests for SymbolicKind type.
- Added new Uniform alias types. Re-organized alias type package. Updated source files to import all aliases from prelude.
- Added Tokio example for async / background inference processing
- Code formatting.
- Added documentation to PreviousTimeIndex trait methods.
- Removed Redundant Braces in TimeIndexable impl for Context
- Even more lints and formatting
- Applied more lints & autofixed
- Added time index to generic context. Resolves  https://github.com/deepcausality-rs/deep_causality/issues/239
- Increased test coverage on adjustable implementations.
- Added  RwLock poisoning handling in CSM type
- Fixed remaining lints
- Increased test coverage
- Improved error message in Causable for Causaloid
- Improved lock error handling specificity in CSM type
- Removed redundant NaN validation in Adjustable for EuclideanTime
- Fixed a few more lints
- Fixed formatting and lints
- Fixed failing tests for updated Display trait impl.
- Fixed error message in Adjustable<f64> for QuaternionSpace
- Replaced panic with proper error handling in Causable for Causaloid
- Added Eq and PartialEq derives to Contextoid
- Added Handling of lock poisoning gracefully in CSM type.
- Fixed blanket trait implementations fo adjustable data and replaced it type bound impl.
- Added more tests for more corner cases in non-Euclidean geometries
- Fixed code formatting
- Added tests for reasoning types
- Added tests for ContextoidType
- Added tests to increase coverage
- Fixed invalid overflow check
- Fix quaternion component assignment bug
- Fixed another failing test
- Fixed failing test
- Fixed more lints and checks
- Removed remaining lifetime annotations in tests and bench utils.
- Updated copyright across the entire repo
- Minor lints
- merged regular and adjustable types. Made adjustable opt in via adjustable trait and type extension.
- Added tests for adjustable space types
- Added tests for time types
- Added tests for symbolic spacetime
- Added tests for spacetime types
- Fixed up remaining tests
- Code formatting.
- Fixed up a lot of tests
- Fixed up test utils and some tests
- Fixed up benchmarks
- Linting and formatting
- Removed lifetime annotation from all types.
- Added adjustable time types.
- Fixed minor lints
- Added multiple temporal types, just in case.
- Fixed complex generic issue. Updated all downstream types.
- Added initial support for non-Euclidean geometries in the context.
- Merge remote-tracking branch 'origin/main'
- Signed commit with gpg key.
- Removed comma after link
- Working on restoring link on Logo on README.md
- Working on restoring Logo on README.md
- Working on restoring Logo on README.md
- Working on restoring Logo on README.md
- Update README.md
- Update README.md
- Update README.md with new Discord link that never expires

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