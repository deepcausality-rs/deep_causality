# Pre-Specification: Monadic Causal Reasoning Migration

## 1. Objective

This document outlines the pre-specification for migrating the default causal reasoning implementations currently found in the `CausableGraphReasoning` trait to a monadic style leveraging the `MonadicCausableGraphReasoning` trait and the `CausalMonad`.
 The goal is to encapsulate error handling, logging, and value propagation within the `PropagatingEffect` monad, providing a more consistent and composable way to perform causal reasoning over graphs.

## 2. Current State: `CausableGraphReasoning`

The `deep_causality/src/traits/causable_graph/graph_reasoning/mod.rs` file defines the `CausableGraphReasoning` trait with the following key methods, all returning `Result<PropagatingEffect, CausalityError>`:

*   `evaluate_single_cause(index: usize, effect: &PropagatingEffect) -> Result<PropagatingEffect, CausalityError>`
    *   Evaluates a single causaloid, returning its effect or an error.
*   `evaluate_subgraph_from_cause(start_index: usize, initial_effect: &PropagatingEffect) -> Result<PropagatingEffect, CausalityError>`
    *   Performs a BFS traversal, propagates effects, and handles `RelayTo` for adaptive reasoning.
*   `evaluate_shortest_path_between_causes(start_index: usize, stop_index: usize, initial_effect: &PropagatingEffect) -> Result<PropagatingEffect, CausalityError>`
    *   Evaluates nodes along the shortest path, propagating effects and immediately returning `RelayTo` effects.

The current implementations rely heavily on Rust's `Result` type for error propagation, and logging is not explicitly integrated into the return type.

## 3. Target State: `MonadicCausableGraphReasoning` and `CausalMonad`

The `deep_causality/src/traits/causable_graph/monadic_graph_reasoning/mod.rs` file introduces the `MonadicCausableGraphReasoning` trait, which defines the `evaluate_graph` method:

*   `evaluate_graph(&self, incoming_effect: PropagatingEffect) -> PropagatingEffect`

This trait expects `T: MonadicCausable<CausalMonad> + Causable + Identifiable + PartialEq + Clone`. The `CausalMonad` (defined in `deep_causality/src/types/reasoning_types/causal_monad/mod.rs`) provides `pure` and `bind` operations for `CausalPropagatingEffect`. The `PropagatingEffect` type now encapsulates `value`, `error: Option<CausalityError>`, and `logs: Vec<CausalEffectLog>`, allowing errors and logs to be carried along with the computational value in a single monadic structure.

## 4. High-Level Migration Strategy

1.  **Deprecate/Re-implement `CausableGraphReasoning` methods:** The existing `CausableGraphReasoning` trait's default implementations will be re-implemented to internally call the new monadic versions. The `Result` return type will be removed, and only `PropagatingEffect` will be returned, as it now carries error and log information.
2.  **Implement corresponding monadic methods as default implementations:** The monadic equivalents of `evaluate_single_cause`, `evaluate_subgraph_from_cause`, and `evaluate_shortest_path_between_causes` will be added as default implementations to the `MonadicCausableGraphReasoning` trait. These methods will strictly use `PropagatingEffect` as their input and output.
3.  **Refactor internal logic:** The logic within these methods will be refactored to use monadic composition where possible, replacing explicit `match Result` statements and `?` operators with patterns operating directly on `PropagatingEffect`.

## 5. Detailed Migration Steps for Each Method

This section details how the logic of each original `CausableGraphReasoning` method will be translated into a monadic pattern, assuming a `monadic_evaluate` method exists on `MonadicCausable` that returns a `PropagatingEffect`.

### 5.1. `evaluate_single_cause` (Monadic Equivalent)

**Objective:** Evaluate a single causaloid, with errors and logs handled by `PropagatingEffect`. This method will be a default implementation in `MonadicCausableGraphReasoning`.

**Changes:**

*   **Return Type:** Switch from `Result<PropagatingEffect, CausalityError>` to `PropagatingEffect`.
*   **Logic:**
    1.  Attempt to retrieve the causaloid using `self.get_causaloid(index)`.
    2.  If the causaloid is not found, return `PropagatingEffect::from_error(CausalityError(...))`. This is a specialized constructor that effectively lifts an error into a `PropagatingEffect` (conceptually similar to `CausalMonad::pure` for error values).
    3.  If found, call `causaloid.evaluate_monadic(effect)`. This method is expected to return a `PropagatingEffect` directly, which handles any internal errors or logs from the causaloid's evaluation.

### 5.2. `evaluate_subgraph_from_cause` (Monadic Equivalent)

**Objective:** Perform BFS, propagating monadic effects and handling `RelayTo` within the monadic structure. This method will be a default implementation in `MonadicCausableGraphReasoning`.

**Changes:**

*   **Return Type:** Switch from `Result<PropagatingEffect, CausalityError>` to `PropagatingEffect`.
*   **Logic:**
    1.  **Initial Validations:** Check `!self.is_frozen()` and `!self.contains_causaloid(start_index)`. If any fail, immediately return `PropagatingEffect::from_error(CausalityError(...))`.
    2.  **BFS Loop Adaptation:** The core BFS logic will remain similar (queue, visited set).
    3.  **Node Evaluation:** Inside the loop, for each `(current_index, incoming_effect)`:
        *   Retrieve the causaloid. If not found, return `PropagatingEffect::from_error(CausalityError(...))` (this will effectively stop the traversal as an error state is propagated).
        *   Evaluate: `let result_effect = causaloid.evaluate_monadic(&incoming_effect);`.
        *   **Monadic Sequencing (Bind):** The iterative update of `last_propagated_effect` and `current_effect` within the loop, combined with `causaloid.evaluate_monadic` (which itself returns a `PropagatingEffect` and handles its internal errors/logs), effectively implements the `CausalMonad::bind` behavior. Each step takes the `PropagatingEffect` from the previous step, processes its value, and produces a new `PropagatingEffect`, propagating errors and accumulating logs.
        *   **Error Propagation:** If `result_effect.is_err()`, the BFS should effectively stop for this path, and this erroring `PropagatingEffect` will be the final result of the function unless another branch succeeded and overwrote it.
        *   **Log Accumulation:** `Causaloid::evaluate_monadic` and the implicit monadic sequencing are responsible for accumulating logs within `result_effect`.
        *   **`RelayTo` Handling:** If `result_effect.value` matches `EffectValue::RelayTo(target_index, inner_effect)`:
            *   **Validation:** Check if `target_index` exists. If not, return `PropagatingEffect::from_error(CausalityError(...))`.
            *   **Dynamic Jump:** Clear the current queue, add `(target_index, *inner_effect)` to the queue, and mark `target_index` as visited. The `last_propagated_effect` should be updated to `result_effect`.
        *   **Normal Propagation:** If `result_effect.value` is not `RelayTo` and `result_effect.error` is `None`:
            *   Get children of `current_index`.
            *   For each unvisited child, add `(child_index, result_effect.clone())` to the queue.
    4.  **Final Result:** The `last_propagated_effect` (which is continually updated by `Causaloid::evaluate_monadic` calls) will be the final returned `PropagatingEffect`.

### 5.3. `evaluate_shortest_path_between_causes` (Monadic Equivalent)

**Objective:** Evaluate a shortest path, propagating monadic effects, and immediately returning `RelayTo` or errors. This method will be a default implementation in `MonadicCausableGraphReasoning`.

**Changes:**

*   **Return Type:** Switch from `Result<PropagatingEffect, CausalityError>` to `PropagatingEffect`.
*   **Logic:**
    1.  **Initial Validations:** Check `!self.is_frozen()`. If it fails, return `PropagatingEffect::from_error(CausalityError(...))`.
    2.  **Single-Node Case:** Handle `start_index == stop_index` by retrieving the causaloid and calling `causaloid.evaluate_monadic(initial_effect)`.
    3.  **Path Retrieval:** Call `self.get_shortest_path(start_index, stop_index)`. If a path is not found, return `PropagatingEffect::from_error(CausalityError(...))`.
    4.  **Path Iteration:** Initialize `let mut current_effect = initial_effect.clone();`
    5.  For each `index` in the `path`:
        *   Retrieve the causaloid. If not found, update `current_effect` to `PropagatingEffect::from_error(CausalityError(...))` and break.
        *   Evaluate: `current_effect = causaloid.evaluate_monadic(&current_effect);`.
        *   **Monadic Sequencing (Bind):** Similar to `evaluate_subgraph_from_cause`, the sequential update of `current_effect` by calling `causaloid.evaluate_monadic` on the previous `current_effect` effectively implements the `CausalMonad::bind` operation for chaining computations.
        *   **Error Check:** If `current_effect.is_err()`, immediately break the loop and return `current_effect`.
        *   **`RelayTo` Check:** If `current_effect.value` matches `EffectValue::RelayTo(_, _)`, immediately return `current_effect`.
    6.  **Final Result:** Return the `current_effect` after the loop completes.

## 6. Usage of `CausalMonad::pure` and `CausalMonad::bind`

*   **`CausalMonad::pure`:** This method is used to lift a raw value `T` into a `CausalPropagatingEffect<T, Error, Log>`. In the context of this migration, `PropagatingEffect` is already the monadic type. Therefore, `CausalMonad::pure` would primarily be used if we had a raw `EffectValue` that needed to be introduced into the monadic flow as a `PropagatingEffect`. For instance, `PropagatingEffect::from_error` is a specialized constructor that effectively uses a similar principle to `pure` by creating a `PropagatingEffect` from an error value.

*   **`CausalMonad::bind`:** This is the core sequencing operation of the monad. It takes a monadic value (`CausalPropagatingEffect<T>`) and a function `f: T -> CausalPropagatingEffect<U>`, applies `f` to the *value contained within* the monad, and returns a new monadic value (`CausalPropagatingEffect<U>`). Crucially, `bind` handles error propagation (short-circuiting if an error is present) and log accumulation. In the detailed migration steps above, the iterative updates of `current_effect` (e.g., `current_effect = causaloid.evaluate_monadic(&current_effect);`) within the loops, combined with `causaloid.evaluate_monadic` returning a `PropagatingEffect`, collectively achieve the behavior of `CausalMonad::bind` for chaining monadic computations. Each step processes the result of the previous step, propagating errors and accumulating logs as defined by the `CausalMonad`'s `bind` implementation.

## 7. Assumptions and Pre-requisites

*   The `CausableGraph` trait (defined in `deep_causality/src/traits/causable_graph/graph/mod.rs`) is already trait-bound to `MonadicCausable<CausalMonad>`, meaning its generic type `T` (the causaloid) must implement `MonadicCausable<CausalMonad>`. This ensures that nodes within the graph can be evaluated monadically.
*   The `MonadicCausable` trait (defined in `deep_causality/src/traits/causable/mod.rs`) provides an `evaluate_monadic` method that returns a `PropagatingEffect`.
*   The `CausalMonad`'s `bind` implementation (in `deep_causality/src/types/reasoning_types/causal_monad/mod.rs`) correctly handles error propagation (short-circuiting on `Some(error)`) and log accumulation, ensuring the monadic properties are maintained.

## 8. Future Considerations

*   **Testing:** All existing tests and sample code will be updated to reflect the new `PropagatingEffect` return type and the monadic approach.
