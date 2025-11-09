# Pre-Specification: Type-Safe Causal Functions and Effects

**Version**: 0.1
**Author**: Gemini
**Date**: 2025-11-09
**Status**: DRAFT

## 1. Overview

This document outlines a major architectural refactoring of the `deep_causality` library's core reasoning engine. The goal is to move from a runtime-polymorphic `EffectValue` enum to a compile-time, generic-based approach for causal functions and causaloids.

This change addresses several critical design issues:
1.  **Silent Failures**: Causal functions currently return a `PropagatingEffect`, which can silently contain an error, making it easy to miss.
2.  **Fallible Conversions**: The use of `match` on the `EffectValue` enum within causal functions is error-prone and not checked at compile time. A causaloid expecting a `Probabilistic` value might receive a `Deterministic` one, leading to a runtime panic or error.
3.  **Lack of Type Safety**: The `CausalFn` signature is not type-safe regarding its input and output values, pushing the burden of type checking onto the developer at runtime.
4.  **Optional Logging**: Logging is manual and not enforced, making it easy to omit crucial diagnostic information.

The proposed solution is to leverage Rust's trait and generic systems to enforce type safety, make failures explicit, and automate logging.

## 2. Proposed Refactoring Plan

The refactoring will be executed in four phases to ensure a structured and manageable transition.

### Phase 1: Introduce Core Traits for Type-Safe Conversions

This phase establishes the foundation for bridging the generic (compile-time) and enum-based (runtime) worlds.

1.  **Create `PropagatingValue` Trait**:
    *   **File**: `src/traits/propagating_value.rs`
    *   **Definition**: A new marker trait to signify that a type can be used as a value within the causal system.
    *   **Signature**: `pub trait PropagatingValue: Debug {}`

2.  **Create `PropagatingEffect` Trait**:
    *   **File**: `src/traits/propagating_effect.rs`
    *   **Definition**: This trait will define the contract for any type that can be losslessly converted to and from the `EffectValue` enum. This is the core mechanism for safe type conversion.
    *   **Signature**:
        ```rust
        pub trait PropagatingEffect: PropagatingValue {
            fn into_effect_value(self) -> EffectValue;
            fn try_from_effect_value(ev: EffectValue) -> Result<Self, CausalityError>;
        }
        ```

3.  **Implement `PropagatingEffect` for Core Types**:
    *   Implement the `PropagatingEffect` trait for all relevant primitive types that correspond to `EffectValue` variants (e.g., `bool`, `f64`, `NumericValue`, `UncertainBool`, `CausalTensor<f64>`, etc.).
    *   This will centralize the fallible conversion logic, replacing all scattered `match` statements with a single, reusable `try_from_effect_value` call.

### Phase 2: Redefine `Causaloid` and `CausalFn` with Generics

This phase applies the new traits to the core components of the reasoning engine.

1.  **Make `CausalFn` and `ContextualCausalFn` Generic**:
    *   **File**: `src/alias/alias_function.rs`
    *   **Change**: The function type aliases will be made generic over an input type `I` and an output type `O`, where both must implement the `Effect` trait. The signature will also be changed to return a `Result`.
    *   **New Signatures**:
        ```rust
            where I: PropagatingEffect, O: PropagatingEffect;

        pub type ContextualCausalFn<I, O, D, S, T, ST, SYM, VS, VT> = fn(
            value: I,
            context: &Arc<RwLock<Context<D, S, T, ST, SYM, VS, VT>>>,
        ) -> Result<O, CausalityError>
            where I: PropagatingEffect, O: PropagatingEffect;
        ```

2.  **Make `Causaloid` Generic**:
    *   **File**: `src/types/causal_types/causaloid/mod.rs`
    *   **Change**: The `Causaloid` struct will be made generic over its input `I` and output `O` types.
    *   **New Signature**: `pub struct Causaloid<I, O, D, S, T, ST, SYM, VS, VT> where I: PropagatingEffect, O: PropagatingEffect, ...`
    *   The `causal_fn` and `context_causal_fn` fields will be updated to use the new generic `CausalFn` and `ContextualCausalFn` types.
    *   Constructors like `new` and `new_with_context` will be updated to accept the new generic function types.

### Phase 3: Centralized Causaloid Management with `CausaloidRegistry`

This phase introduces a centralized registry for managing all `Causaloid` instances, enabling both flexible composition and optimized evaluation. This approach integrates with the existing `Causable` and `MonadicCausable` traits.

1.  **Create `CausaloidRegistry`**:
    *   **File**: `src/types/causal_types/causaloid_registry.rs`
    *   **Definition**: A centralized, type-erased storage for all `Causaloid` instances. It uses `TypeId` to store homogeneous collections of `Causaloid<I, O>` and provides a hybrid static/dynamic dispatch mechanism.
    *   **Key Components**:
        *   `storage: HashMap<TypeId, Box<dyn Any>>`: Stores `Vec<Causaloid<I, O>>` for different `I, O` types.
        *   `lookup: HashMap<CausaloidId, (TypeId, usize)>`: Maps a stable `CausaloidId` (u64) to its `TypeId` and index within its typed vector.
        *   `next_id: CausaloidId`: Counter for generating unique IDs.
    *   **Core Methods**:
        *   `register<T: MonadicCausable<P> + 'static>(&mut self, causaloid: T) -> CausaloidId`: Adds a `Causaloid` to its appropriate typed vector and returns a unique ID.
        *   `evaluate(&self, id: CausaloidId, effect: &PropagatingEffect) -> PropagatingEffect`: The core dispatch method. It performs a `TypeId` lookup and then either:
            *   **Static Dispatch**: If the `TypeId` matches a known, built-in `Causaloid<I, O>` type, it downcasts to the concrete `Vec<Causaloid<I, O>>` and calls the `evaluate` method directly.
            *   **Dynamic Dispatch**: If the `TypeId` is unknown (e.g., a user-defined custom type), it falls back to calling `evaluate` on a `Box<dyn MonadicCausable<P>>` trait object.

2.  **Update Composite `Causaloid`s and `CausaloidGraph`**:
    *   The `Causaloid` struct fields `causal_coll` and `causal_graph` will be updated to hold collections of `CausaloidId`s instead of actual `Causaloid` instances.
    *   **New Field Types**:
        *   `causal_coll: Option<Arc<Vec<CausaloidId>>>`
        *   `causal_graph: Option<Arc<CausaloidGraph<CausaloidId>>>`
    *   This makes the graph and collection structures lightweight and independent of the concrete `Causaloid` types.

### Phase 4: Update Evaluation Logic and Automate Logging

This phase ties everything together by updating the evaluation logic to use the new typed structures and enforce logging, leveraging the `CausaloidRegistry` for dispatch.

1.  **Update `MonadicCausable` Implementation for `Causaloid`**:
    *   **File**: `src/types/causal_types/causaloid/causable.rs`
    *   **Change**: The `evaluate` method for `Causaloid<I, O, ...>` will be rewritten. This `evaluate` method is the strongly-typed internal evaluation.
    *   **New Logic (Internal `Causaloid<I, O>` evaluation)**:
        1.  It receives a strongly-typed input `I`.
        2.  It calls the strongly-typed internal `causal_fn(I) -> Result<O, _>`.
        3.  It takes the `Result<O, _>` and converts the `O` back to a `PropagatingEffect` using `O::into_effect_value()`.
        4.  **Automated Logging**: At this step, it will automatically create a log entry detailing the causaloid ID, the input value, and the output value or error. This log is pushed into the `logs` field of the returned `PropagatingEffect`.
        5.  The final `PropagatingEffect` (with value/error and logs) is returned.

2.  **Update `MonadicCausable` Implementation for `CausalMonad` (or similar orchestrator)**:
    *   **File**: `src/types/causal_types/causal_monad/mod.rs` (or relevant orchestrator)
    *   **Change**: The `bind` or `evaluate` method of the orchestrator will be updated to interact with the `CausaloidRegistry`.
    *   **New Logic**:
        1.  It receives a `CausaloidId` and an input `PropagatingEffect`.
        2.  It calls `CausaloidRegistry::evaluate(id, input_effect)`.
        3.  The `CausaloidRegistry` handles the hybrid static/dynamic dispatch to the correct `Causaloid<I, O>` instance's internal `evaluate` method.
        4.  The `PropagatingEffect` (containing the result and logs) is returned by the registry.

## 3. Impact Analysis

*   **Breaking Changes**: This is a significant breaking change. All code that constructs or directly interacts with `Causaloid` and `CausalFn` will need to be updated.
*   **Benefits**:
    *   **Compile-Time Safety**: Errors from incorrect effect value types will be caught at compile time instead of runtime.
    *   **Explicit Failures**: The `Result`-based function signatures make all potential failures explicit and force the caller to handle them.
    *   **Improved Developer Experience**: Creating new causaloids will be less error-prone and more intuitive.
    *   **Guaranteed Logging**: Core evaluation steps will be automatically logged, improving debuggability and traceability.
*   **Affected Modules**: The changes will primarily impact `deep_causality/src/types`, `deep_causality/src/traits`, and `deep_causality/src/alias`. All tests and examples using these types will also require updates.
*   **"No Dyn" Constraint**: The project's constraint of avoiding `dyn Trait` in performance-critical hot paths is now respected. Dynamic dispatch is confined to the `CausaloidRegistry`'s fallback mechanism for user-defined types, while core library types benefit from static dispatch. This provides an optimal balance between performance and extensibility.

This refactoring will establish a more robust, safe, and maintainable foundation for the `deep_causality` library.
