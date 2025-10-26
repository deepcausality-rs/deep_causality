# Tasks for HKT Integration for Uncertain and MaybeUncertain Types

This document outlines the tasks required to implement HKT integration for `Uncertain<T>` and `MaybeUncertain<T>` types in the `deep_causality_uncertain` crate.

## Task Generation Strategy

Tasks are generated based on the feature specification, data model, and quickstart scenarios. They are ordered by dependencies, with setup tasks first, followed by core trait implementations, and finally testing and polish tasks.

## Ordering Strategy

- Setup tasks first.
- Core trait implementations (models) before tests.
- Tests before final polish.
- Tasks marked with `[P]` can be executed in parallel.

## Tasks

### Setup

- **T001**: Ensure `deep_causality_uncertain` crate is set up for development.
  - **Description**: Verify that the `deep_causality_uncertain` crate can be built and tested independently.
  - **File**: `deep_causality_uncertain/Cargo.toml`

### Core Trait Implementations

- **T002 [P]**: Implement `UncertainWitness` struct.
  - **Description**: Create the zero-sized `UncertainWitness` struct. Refer to `data-model.md` for entity definition.
  - **File**: `deep_causality_uncertain/src/extensions/uncertain_witness.rs`

- **T003 [P]**: Implement `MaybeUncertainWitness` struct.
  - **Description**: Create the zero-sized `MaybeUncertainWitness` struct. Refer to `data-model.md` for entity definition.
  - **File**: `deep_causality_uncertain/src/extensions/maybe_uncertain_witness.rs`

- **T004 [P]**: Implement `HKT` trait for `UncertainWitness`.
  - **Description**: Implement the `HKT` trait from `deep_causality_haft` for `UncertainWitness`. Refer to `data-model.md` for trait definition.
  - **File**: `deep_causality_uncertain/src/extensions/uncertain_hkt.rs`

- **T005 [P]**: Implement `HKT` trait for `MaybeUncertainWitness`.
  - **Description**: Implement the `HKT` trait from `deep_causality_haft` for `MaybeUncertainWitness`. Refer to `data-model.md` for trait definition.
  - **File**: `deep_causality_uncertain/src/extensions/maybe_uncertain_hkt.rs`

- **T006 [P]**: Implement `Functor` trait for `UncertainWitness`.
  - **Description**: Implement the `Functor` trait from `deep_causality_haft` for `UncertainWitness`, including the `fmap` operation. Refer to `data-model.md` for trait definition and `quickstart.md` for Functor Transformation scenario.
  - **File**: `deep_causality_uncertain/src/extensions/uncertain_functor.rs`

- **T007 [P]**: Implement `Functor` trait for `MaybeUncertainWitness`.
  - **Description**: Implement the `Functor` trait from `deep_causality_haft` for `MaybeUncertainWitness`, including the `fmap` operation. Refer to `data-model.md` for trait definition and `quickstart.md` for Functor Transformation scenario.
  - **File**: `deep_causality_uncertain/src/extensions/maybe_uncertain_functor.rs`

- **T008 [P]**: Implement `Applicative` trait for `UncertainWitness`.
  - **Description**: Implement the `Applicative` trait from `deep_causality_haft` for `UncertainWitness`, including `pure` and `apply` operations. Refer to `data-model.md` for trait definition and `quickstart.md` for Applicative Combination scenario.
  - **File**: `deep_causality_uncertain/src/extensions/uncertain_applicative.rs`

- **T009 [P]**: Implement `Applicative` trait for `MaybeUncertainWitness`.
  - **Description**: Implement the `Applicative` trait from `deep_causality_haft` for `MaybeUncertainWitness`, including `pure` and `apply` operations. Refer to `data-model.md` for trait definition and `quickstart.md` for Applicative Combination scenario.
  - **File**: `deep_causality_uncertain/src/extensions/maybe_uncertain_applicative.rs`

- **T010 [P]**: Implement `Monad` trait for `UncertainWitness`.
  - **Description**: Implement the `Monad` trait from `deep_causality_haft` for `UncertainWitness`, including the `bind` operation. Refer to `data-model.md` for trait definition and `quickstart.md` for Monadic Chaining scenario.
  - **File**: `deep_causality_uncertain/src/extensions/uncertain_monad.rs`

- **T011 [P]**: Implement `Monad` trait for `MaybeUncertainWitness`.
  - **Description**: Implement the `Monad` trait from `deep_causality_haft` for `MaybeUncertainWitness`, including the `bind` operation. Refer to `data-model.md` for trait definition and `quickstart.md` for Monadic Chaining scenario.
  - **File**: `deep_causality_uncertain/src/extensions/maybe_uncertain_monad.rs`

- **T012 [P]**: Ensure `UncertainWitness` and `MaybeUncertainWitness` do NOT implement `Foldable` trait.
  - **Description**: Verify that `Foldable` trait is not implemented for `UncertainWitness` and `MaybeUncertainWitness`. Refer to `data-model.md` for trait definition and `quickstart.md` for Foldable Restriction scenario.
  - **File**: `deep_causality_uncertain/src/extensions/mod.rs` (and `maybe_uncertain/mod.rs`)

### Test Implementations

- **T013 [P]**: Write integration test for Functor Transformation scenario.
  - **Description**: Create an integration test to verify the `fmap` operation for `Uncertain<T>` and `MaybeUncertain<T>`. Refer to `quickstart.md` for the scenario details.
  - **File**: `deep_causality_uncertain/tests/hkt_integration_tests.rs`

- **T014 [P]**: Write integration test for Applicative Combination scenario.
  - **Description**: Create an integration test to verify the `apply` operation for `Uncertain<T>` and `MaybeUncertain<T>`. Refer to `quickstart.md` for the scenario details.
  - **File**: `deep_causality_uncertain/tests/hkt_integration_tests.rs`

- **T015 [P]**: Write integration test for Monadic Chaining scenario.
  - **Description**: Create an integration test to verify the `bind` operation for `Uncertain<T>` and `MaybeUncertain<T>`. Refer to `quickstart.md` for the scenario details.
  - **File**: `deep_causality_uncertain/tests/hkt_integration_tests.rs`

- **T016 [P]**: Write integration test for Foldable Restriction scenario.
  - **Description**: Create an integration test to verify that `Foldable` operations are not available for `Uncertain<T>` and `MaybeUncertain<T>`. Refer to `quickstart.md` for the scenario details.
  - **File**: `deep_causality_uncertain/tests/hkt_integration_tests.rs`

### Polish

- **T017**: Run `make format && make fix`.
  - **Description**: Format the code and fix any linting issues across the monorepo.
  - **Command**: `make format && make fix`

- **T018**: Run `make test`.
  - **Description**: Execute the entire test suite for the monorepo to ensure no regressions.
  - **Command**: `make test`
