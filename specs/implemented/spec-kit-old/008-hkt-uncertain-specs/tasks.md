# Tasks for HKT Integration for Uncertain and MaybeUncertain Types

This document outlines the tasks required to implement HKT integration for `Uncertain<T>` and `MaybeUncertain<T>` types in the `deep_causality_uncertain` crate, leveraging `ConstTree` and the `ProbabilisticType` trait.

## Task Generation Strategy

Tasks are generated based on the feature specification, data model, quickstart scenarios, and the detailed architectural rationale. They are ordered by dependencies, with foundational changes first, followed by core trait implementations, and finally testing and polish tasks.

## Ordering Strategy

-   Setup tasks first.
-   New trait and core data structure definitions before their implementations.
-   Core `Uncertain<T>` refactoring before HKT trait implementations.
-   Tests before implementation (TDD where applicable).
-   Tasks marked with `[P]` can be executed in parallel.

## Tasks

### Setup and Foundational Changes

-   **T001**: Ensure `deep_causality_uncertain` crate is set up for development.
    -   **Description**: Verify that the `deep_causality_uncertain` crate can be built and tested independently.
    -   **File**: `deep_causality_uncertain/Cargo.toml`

-   **T002**: Add `deep_causality_ast` as a dependency to `deep_causality_uncertain`.
    -   **Description**: Update `Cargo.toml` to include `deep_causality_ast` as a dependency.
    -   **File**: `deep_causality_uncertain/Cargo.toml`

-   **T003 [P]**: Define `ProbabilisticType` trait.
    -   **Description**: Create the `ProbabilisticType` trait with `to_sampled_value`, `from_sampled_value`, and `default_value` methods.
    -   **File**: `deep_causality_uncertain/src/traits/probabilistic_type.rs`

-   **T004 [P]**: Implement `ProbabilisticType` for `f64`.
    -   **Description**: Provide the necessary implementations for `f64` to conform to `ProbabilisticType`.
    -   **File**: `deep_causality_uncertain/src/types/f64_probabilistic_type.rs` (new file, or integrate into existing f64 related files)

-   **T005 [P]**: Implement `ProbabilisticType` for `bool`.
    -   **Description**: Provide the necessary implementations for `bool` to conform to `ProbabilisticType`.
    -   **File**: `deep_causality_uncertain/src/types/bool_probabilistic_type.rs` (new file, or integrate into existing bool related files)

-   **T006**: Define `UncertainNodeContent<T: ProbabilisticType>` enum.
    -   **Description**: Create the `UncertainNodeContent` enum to represent symbolic operations and values within the `ConstTree`, generic over `T: ProbabilisticType`. This will replace the functionality of `ComputationNode`.
    -   **File**: `deep_causality_uncertain/src/types/computation/uncertain_node_content.rs`

-   **T007**: Update `Uncertain<T>` struct definition.
    -   **Description**: Modify `Uncertain<T>` to be generic over `T: ProbabilisticType` and replace `root_node: Arc<ComputationNode>` with `root_node: Arc<ConstTree<UncertainNodeContent<T>>>`.
    -   **File**: `deep_causality_uncertain/src/types/uncertain/mod.rs`

-   **T008**: Deprecate `ComputationNode` and related files.
    -   **Description**: Mark `ComputationNode` and its associated implementation files as deprecated, or remove them if no longer referenced.
    -   **File**: `deep_causality_uncertain/src/types/computation/node/computation_node.rs` and related files.

### Rewrite of `Uncertain<T>` Constructors and Operators

-   **T009**: Rewrite `Uncertain<T>::point` constructor.
    -   **Description**: Update `Uncertain<T>::point` to construct a `ConstTree<UncertainNodeContent<T>>` representing a point distribution.
    -   **File**: `deep_causality_uncertain/src/types/uncertain/mod.rs` (or `uncertain_f64.rs`, `uncertain_bool.rs`)

-   **T010**: Rewrite `Uncertain<f64>::normal` and `Uncertain<f64>::uniform` constructors.
    -   **Description**: Update these constructors to build `ConstTree<UncertainNodeContent<f64>>` instances for normal and uniform distributions.
    -   **File**: `deep_causality_uncertain/src/types/uncertain/uncertain_f64.rs`

-   **T011**: Rewrite `Uncertain<bool>::bernoulli` constructor.
    -   **Description**: Update `Uncertain<bool>::bernoulli` to build a `ConstTree<UncertainNodeContent<bool>>` instance for a Bernoulli distribution.
    -   **File**: `deep_causality_uncertain/src/types/uncertain/uncertain_bool.rs`

-   **T012**: Rewrite `Uncertain<f64>` arithmetic operator overloads (`Add`, `Sub`, `Mul`, `Div`, `Neg`).
    -   **Description**: Update these operators to construct new `ConstTree<UncertainNodeContent<f64>>` instances representing the arithmetic operations.
    -   **File**: `deep_causality_uncertain/src/types/uncertain/uncertain_op_arithmetic.rs`

-   **T013**: Rewrite `Uncertain<f64>` comparison methods (`greater_than`, `less_than`, `equals`, etc.).
    -   **Description**: Update these methods to construct new `ConstTree<UncertainNodeContent<bool>>` instances representing the comparison operations.
    -   **File**: `deep_causality_uncertain/src/types/uncertain/uncertain_op_comparison.rs`

-   **T014**: Rewrite `Uncertain<bool>` logical operator overloads (`BitAnd`, `BitOr`, `Not`, `BitXor`).
    -   **Description**: Update these operators to construct new `ConstTree<UncertainNodeContent<bool>>` instances representing the logical operations.
    -   **File**: `deep_causality_uncertain/src/types/uncertain/uncertain_op_logic.rs`

-   **T015**: Rewrite `Uncertain<f64>::map` and `Uncertain<f64>::map_to_bool`.
    -   **Description**: Update these methods to store the provided functions symbolically within `ConstTree<UncertainNodeContent<T>>`.
    -   **File**: `deep_causality_uncertain/src/types/uncertain/uncertain_f64.rs`

-   **T016**: Rewrite `Uncertain<bool>::to_bool` and `Uncertain<bool>::probability_exceeds`.
    -   **Description**: Update these methods to evaluate the `ConstTree<UncertainNodeContent<bool>>` via sampling and perform statistical hypothesis testing.
    -   **File**: `deep_causality_uncertain/src/types/uncertain/uncertain_bool.rs`

### Rewrite of Sampling Engine

-   **T017**: Rewrite `SequentialSampler::evaluate_node`.
    -   **Description**: Update `evaluate_node` to traverse and interpret `ConstTree<UncertainNodeContent<T>>`, applying symbolic functions and operations during sampling.
    -   **File**: `deep_causality_uncertain/src/types/sampler/sequential_sampler.rs`

### HKT Trait Implementations

-   **T018 [P]**: Implement `UncertainWitness` struct.
    -   **Description**: Create the zero-sized `UncertainWitness` struct.
    -   **File**: `deep_causality_uncertain/src/extensions/uncertain_witness.rs`

-   **T019 [P]**: Implement `MaybeUncertainWitness` struct.
    -   **Description**: Create the zero-sized `MaybeUncertainWitness` struct.
    -   **File**: `deep_causality_uncertain/src/extensions/maybe_uncertain_witness.rs`

-   **T020 [P]**: Implement `HKT` trait for `UncertainWitness`.
    -   **Description**: Implement the `HKT` trait from `deep_causality_haft` for `UncertainWitness`, ensuring `Type<T>` resolves to `Uncertain<T: ProbabilisticType>`.
    -   **File**: `deep_causality_uncertain/src/extensions/uncertain_hkt.rs`

-   **T021 [P]**: Implement `HKT` trait for `MaybeUncertainWitness`.
    -   **Description**: Implement the `HKT` trait from `deep_causality_haft` for `MaybeUncertainWitness`, ensuring `Type<T>` resolves to `MaybeUncertain<T: ProbabilisticType>`.
    -   **File**: `deep_causality_uncertain/src/extensions/maybe_uncertain_hkt.rs`

-   **T022**: Implement `Functor` trait for `UncertainWitness`.
    -   **Description**: Implement `fmap` for `UncertainWitness`, ensuring `A` and `B` are `ProbabilisticType` and functions are symbolically embedded in `ConstTree`.
    -   **File**: `deep_causality_uncertain/src/extensions/uncertain_functor.rs`

-   **T023**: Implement `Functor` trait for `MaybeUncertainWitness`.
    -   **Description**: Implement `fmap` for `MaybeUncertainWitness`, leveraging `Uncertain<T>`'s Functor implementation.
    -   **File**: `deep_causality_uncertain/src/extensions/maybe_uncertain_functor.rs`

-   **T024**: Implement `Applicative` trait for `UncertainWitness`.
    -   **Description**: Implement `pure` and `apply` for `UncertainWitness`, ensuring `A` and `B` are `ProbabilisticType` and operations are symbolically embedded.
    -   **File**: `deep_causality_uncertain/src/extensions/uncertain_applicative.rs`

-   **T025**: Implement `Applicative` trait for `MaybeUncertainWitness`.
    -   **Description**: Implement `pure` and `apply` for `MaybeUncertainWitness`, leveraging `Uncertain<T>`'s Applicative implementation.
    -   **File**: `deep_causality_uncertain/src/extensions/maybe_uncertain_applicative.rs`

-   **T026**: Implement `Monad` trait for `UncertainWitness`.
    -   **Description**: Implement `bind` for `UncertainWitness`, ensuring `A` and `B` are `ProbabilisticType` and monadic functions are symbolically embedded and flattened.
    -   **File**: `deep_causality_uncertain/src/extensions/uncertain_monad.rs`

-   **T027**: Implement `Monad` trait for `MaybeUncertainWitness`.
    -   **Description**: Implement `bind` for `MaybeUncertainWitness`, leveraging `Uncertain<T>`'s Monad implementation.
    -   **File**: `deep_causality_uncertain/src/extensions/maybe_uncertain_monad.rs`

-   **T028 [P]**: Ensure `UncertainWitness` and `MaybeUncertainWitness` do NOT implement `Foldable` trait.
    -   **Description**: Verify that `Foldable` trait is not implemented for `UncertainWitness` and `MaybeUncertainWitness`.
    -   **File**: `deep_causality_uncertain/src/extensions/mod.rs` (and `maybe_uncertain/mod.rs`)

### Testing and Validation

-   **T029 [P]**: Write unit tests for `ProbabilisticType` implementations (`f64`, `bool`).
    -   **Description**: Verify correct conversion to/from `SampledValue` and `default_value` behavior.
    -   **File**: `deep_causality_uncertain/tests/probabilistic_type_tests.rs`

-   **T030 [P]**: Write unit tests for `UncertainNodeContent` enum.
    -   **Description**: Verify correct construction, cloning, and representation of various node types.
    -   **File**: `deep_causality_uncertain/tests/uncertain_node_content_tests.rs`

-   **T031**: Write integration tests for `Uncertain<T>` constructors and operators.
    -   **Description**: Verify that rewritten constructors and operators produce correct `ConstTree` structures and maintain original behavior.
    -   **File**: `deep_causality_uncertain/tests/uncertain_core_tests.rs`

-   **T032**: Write integration tests for `SequentialSampler` with `ConstTree`.
    -   **Description**: Verify that the rewritten `evaluate_node` correctly samples from `ConstTree`-based computation graphs.
    -   **File**: `deep_causality_uncertain/tests/sequential_sampler_tests.rs`

-   **T033**: Write integration test for Functor Transformation scenario.
    -   **Description**: Create an integration test to verify the `fmap` operation for `Uncertain<T>` and `MaybeUncertain<T>` as per `quickstart.md`.
    -   **File**: `deep_causality_uncertain/tests/hkt_integration_tests.rs`

-   **T034**: Write integration test for Applicative Combination scenario.
    -   **Description**: Create an integration test to verify the `apply` operation for `Uncertain<T>` and `MaybeUncertain<T>` as per `quickstart.md`.
    -   **File**: `deep_causality_uncertain/tests/hkt_integration_tests.rs`

-   **T035**: Write integration test for Monadic Chaining scenario.
    -   **Description**: Create an integration test to verify the `bind` operation for `Uncertain<T>` and `MaybeUncertain<T>` as per `quickstart.md`.
    -   **File**: `deep_causality_uncertain/tests/hkt_integration_tests.rs`

-   **T036 [P]**: Write integration test for Foldable Restriction scenario.
    -   **Description**: Create an integration test to verify that `Foldable` operations are not available for `Uncertain<T>` and `MaybeUncertain<T>`.
    -   **File**: `deep_causality_uncertain/tests/hkt_integration_tests.rs`

-   **T037**: Write integration tests for `deep_causality` crate compatibility (`_evaluate_uncertain_logic`).
    -   **Description**: Verify that the `_evaluate_uncertain_logic` function in `deep_causality` continues to function correctly with the refactored `Uncertain<T>`.
    -   **File**: `deep_causality_uncertain/tests/deep_causality_compat_tests.rs`

-   **T038**: Write integration tests for `deep_causality` crate compatibility (`EffectEthos::evaluate_action`).
    -   **Description**: Verify that `EffectEthos::evaluate_action` (especially for uncertain norms) functions correctly with the refactored `Uncertain<T>`.
    -   **File**: `deep_causality_uncertain/tests/deep_causality_compat_tests.rs`

### Polish

-   **T039**: Run `make format && make fix`.
    -   **Description**: Format the code and fix any linting issues across the monorepo.
    -   **Command**: `make format && make fix`

-   **T040**: Run `make test`.
    -   **Description**: Execute the entire test suite for the monorepo to ensure no regressions.
    -   **Command**: `make test`