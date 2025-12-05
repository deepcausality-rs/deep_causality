# Migration Guide: `deep_causality` to `deep_causality_core`

This guide details the process of migrating the `deep_causality` crate to use the foundational types provided by the `deep_causality_core` crate. The migration's primary goal is to adopt a more robust, flexible, and explicit type system for handling causal effects, state, and context.

The core of this change is the move from an arity-3 monadic system (Value, Error, Log) to a more powerful arity-5 system (Value, State, Context, Error, Log).

 

## 1. Core Concepts: The Arity-5 Monad

The new foundation in `deep_causality_core` is the `CausalEffectPropagationProcess<Value, State, Context, Error, Log>` struct. This is a generic container that represents the complete state of a causal computation at any given moment.

-   **`Value`**: The computation's result.
-   **`State`**: Mutable state that is carried through the computation chain. This is a new, explicit concept.
-   **`Context`**: Environmental or configuration data, usually read-only during a computation step.
-   **`Error`**: The error type for propagation.
-   **`Log`**: The log type for accumulating an audit trail.

To simplify its use, `deep_causality_core` provides two key type aliases:

-   **`PropagatingEffect<T>`**: A **stateless** and **contextless** specialization, ideal for simple functional transformations.
    -   `type PropagatingEffect<T> = CausalEffectPropagationProcess<T, (), (), CausalityError, EffectLog>;`
-   **`PropagatingProcess<T, S, C>`**: A **stateful** and **context-aware** specialization for complex, history-dependent computations.
    -   `type PropagatingProcess<T, S, C> = CausalEffectPropagationProcess<T, S, C, CausalityError, EffectLog>;`

 

## 2. Migration of `PropagatingEffect` and `EffectValue`

The most significant change is the replacement of the concrete `PropagatingEffect` struct and `EffectValue` enum with their new generic counterparts from the core crate.

### Before (`deep_causality`)

-   `PropagatingEffect`: A struct holding `value: EffectValue`, `error: Option<CausalityError>`, and `logs: CausalEffectLog`.
-   `EffectValue`: A large enum with many concrete variants like `Boolean(bool)`, `Numerical(f64)`, `Tensor(...)`, etc.
-   `IntoEffectValue` trait: Used to convert concrete types into an `EffectValue` variant.

```rust
// Old deep_causality
pub struct PropagatingEffect {
    pub value: EffectValue,
    // ...
}

pub enum EffectValue {
    None,
    Boolean(bool),
    Numerical(f64),
    // ... many other variants
}
```

### After (`deep_causality_core`)

-   `PropagatingEffect<T>`: A type alias for the arity-5 `CausalEffectPropagationProcess`. `T` is the type of the value.
-   `EffectValue<T>`: A generic enum that primarily wraps the successful result in a `Value(T)` variant.

```rust
// New deep_causality_core
pub type PropagatingEffect<T> = CausalEffectPropagationProcess<T, (), (), CausalityError, EffectLog>;

pub enum EffectValue<T> {
    None,
    Value(T),
    // ... other control-flow variants
}
```

### Migration Steps

1.  **Replace Struct with Alias**: Change all usages of `deep_causality::PropagatingEffect` to `deep_causality_core::PropagatingEffect<T>`, where `T` is the specific type of the value being propagated (e.g., `bool`, `f64`).

2.  **Update Value Construction**:
    -   `EffectValue::Boolean(true)` becomes `EffectValue::Value(true)`. The overall effect type becomes `PropagatingEffect<bool>`.
    -   `EffectValue::Numerical(0.5)` becomes `EffectValue::Value(0.5)`. The overall effect type becomes `PropagatingEffect<f64>`.
    -   `EffectValue::None` remains the same, but the effect type will be `PropagatingEffect<T>` where `T` has a `Default` implementation.

3.  **Remove the `IntoEffectValue` Trait**: The old `IntoEffectValue` trait is no longer needed. Instead of converting a type to a large enum variant, the type itself becomes the generic parameter `T` in `PropagatingEffect<T>`.

 

## 3. Migration of `Causaloid` and Causal Functions

The signature of the causal function (`causal_fn`) within a `Causaloid` must change to align with the new monadic system.

### Before (`deep_causality`)

The causal function returned a `Result` containing a `CausalFnOutput`, which bundled the output value and a log.

```rust
// Old Causaloid function signature
fn my_causal_fn(obs: f64) -> Result<CausalFnOutput<bool>, CausalityError> {
    let output = obs > 0.5;
    let log = CausalEffectLog::new();
    // ... add to log
    Ok(CausalFnOutput::new(output, log))
}
```

### After (`deep_causality_core`)

The causal function must now return a complete `PropagatingEffect<T>` (for stateless causaloids) or `PropagatingProcess<T, S, C>` (for stateful ones).

```rust
// New Causaloid function signature
use deep_causality_core::{CausalMonad, PropagatingEffect};
use deep_causality_haft::MonadEffect5;

fn my_new_causal_fn(obs: f64) -> PropagatingEffect<bool> {
    let mut effect = CausalMonad::pure(obs > 0.5);
    effect.logs.add_entry("Causal function executed");
    effect
}
```

### Migration Steps

1.  **Update `causal_fn` Signature**: Change the return type from `Result<CausalFnOutput<O>, CausalityError>` to `PropagatingEffect<O>`.
2.  **Use `CausalMonad::pure`**: Use `CausalMonad::pure(value)` to lift the output value into the new monadic context. This creates a successful effect with an empty log.
3.  **Logging**: Add log entries directly to the `logs` field of the returned effect.
4.  **Error Handling**: To return an error, create an effect using `PropagatingEffect::from_error(error)`.

 

## 4. Using `Context` and the New `State` Parameter 

The arity-5 monad introduces an explicit `State` parameter, formalizing what was previously implicit or manual state management. This is a significant architectural improvement that enhances clarity, testability, and type safety.

### Conceptual Shift

| | Before (`deep_causality`) | After (`deep_causality_core`) |
| :--- | :--- | :--- |
| **Context** | Passed as an explicit `&Arc<RwLock<T>>` argument to `context_causal_fn`. | Becomes a generic type parameter `C` within `PropagatingProcess<V, S, C>`. |
| **State** | No formal concept. State was managed implicitly, often via `RefCell` or by mutating data inside the shared `Context`. | Becomes a first-class generic type parameter `S` within `PropagatingProcess<V, S, C>`. |
| **Access** | Requires locking (`.read().unwrap()`) which can panic and introduces runtime overhead. | State `S` and Context `C` are passed by value into the causal function, avoiding locks. |

This change moves from a shared-memory concurrency model (`Arc<RwLock>`) to a message-passing model where state is explicitly threaded through the computation chain.

### Before: Manual Context Locking

The old pattern required manually locking the context, which was verbose and prone to runtime errors (e.g., lock poisoning).

```rust
// Old contextual function
fn old_context_fn(
    obs: f64,
    ctx: &Arc<RwLock<BaseContext>>
) -> Result<CausalFnOutput<bool>, CausalityError> {
    // 1. Manually lock the context to read from it
    let context = ctx.read().unwrap();
    let threshold = context.get_some_threshold(); // Hypothetical function

    // 2. Logic uses the context data
    let is_active = obs > threshold;
    
    // 3. To "update" state, one might have to acquire a write lock,
    // which is even more cumbersome and risky.
    // let mut context = ctx.write().unwrap();
    // context.set_some_value(...);

    Ok(CausalFnOutput::new(is_active, CausalEffectLog::new()))
}
```

### After: Explicit State and Context Passing

The new pattern uses the `PropagatingProcess` type, where `State` and `Context` are formal parameters. The `bind` method of `CausalEffectPropagationProcess` handles passing them between steps.

**Step 1: Define Your State and Context Structs**

These are plain data structures that hold the relevant information.

```rust
// In your application logic:

// Context: Read-only configuration for the process.
#[derive(Clone)]
pub struct CalculationContext {
    pub threshold: f64,
    pub description: String,
}

// State: Mutable data that evolves with each step.
#[derive(Clone, Default)]
pub struct AccumulatorState {
    pub total_sum: f64,
    pub observation_count: u32,
}
```

**Step 2: Update the Causal Function Signature**

The function now receives `state` and `context` directly and must return a new `PropagatingProcess`.

```rust
use deep_causality_core::{CausalEffectPropagationProcess, CausalMonad, PropagatingProcess};
use deep_causality_haft::MonadEffect5;

// The function signature now includes State and Context.
fn new_stateful_fn(
    obs: f64,
    mut state: AccumulatorState,
    context: Option<CalculationContext>,
) -> PropagatingProcess<bool, AccumulatorState, CalculationContext> {
    // 1. Context is passed as Option<C>, ready to use.
    let ctx = context.as_ref().expect("Context is required for this operation");

    let is_above_threshold = obs > ctx.threshold;

    // 2. Update the state based on the current step.
    state.total_sum += obs;
    state.observation_count += 1;

    // 3. Return a new process with the result and the *updated* state.
    // The core `CausalEffectPropagationProcess` is used to construct the return value.
    let mut process = CausalMonad::pure(is_above_threshold);
    process.state = state; // Set the new state
    process.context = context; // Pass the context along
    process.logs.add_entry(&format!("Observation: {}, Threshold: {}", obs, ctx.threshold));

    process
}
```

**Step 3: Chain Stateful Functions with `bind`**

The `Causaloid`'s `evaluate` method will now use `bind` to orchestrate the flow of `(value, state, context)`.

```rust
// Simplified Causaloid `evaluate` implementation
fn evaluate(&self, effect: &PropagatingEffect<f64>) -> PropagatingProcess<bool, _, _> {
    // Start with a pure effect and lift it into a stateful process
    let initial_process = PropagatingProcess::with_state(
        effect.clone(), // The initial stateless effect
        AccumulatorState::default(), // The initial state
        Some(CalculationContext { threshold: 0.5, description: "config".into() }) // The context
    );

    // Chain the computation. The bind method handles state/context passing.
    initial_process.bind(|value, state, context| {
        // `value` is f64 from initial_process
        self.causal_fn(value.into_value().unwrap(), state, context) // Call our stateful function
    })
}
```

This explicit model makes the data flow clear, eliminates locking, and makes individual causal functions much easier to test in isolation.


## 5. Impact on Hypergraph and Recursive Structures

A key feature of `deep_causality` is its use of isomorphic recursive data structures, where a `Causaloid` can contain other `Causaloid`s, forming either a `Collection` or a `Graph` (a hypergraph). The migration to `deep_causality_core` preserves this powerful feature while significantly enhancing its capabilities by introducing explicit state management into the evaluation process.

### The Improvement: Stateful Traversal

Previously, evaluating a `CausaloidGraph` or a `Causaloid` collection was primarily about aggregating the final results of the child nodes. State management across the traversal was either impossible or required cumbersome shared-memory patterns (e.g., mutating a shared context via `Arc<RwLock<...>>`).

With `deep_causality_core`, the evaluation of a `CausaloidGraph` or `Collection` now naturally produces a `PropagatingProcess`. This means a single, coherent `State` object can be threaded through the entire recursive evaluation, making the structures more powerful than ever.

### Impact on Causal Graphs

When a `Causaloid` of type `Graph` is evaluated, the traversal of the inner `CausaloidGraph` (e.g., via `evaluate_subgraph_from_cause`) becomes a stateful process.

**Benefit**: You can now model scenarios where the evaluation of one node in the graph directly influences the state for subsequent nodes in the traversal path.

**Example: Path and Evidence Accumulation**

Imagine you want the graph evaluation to not only determine an outcome but also to record the path taken and accumulate evidence along the way.

**1. Define a `State` for traversal:**

```rust
#[derive(Clone, Default, Debug)]
struct GraphTraversalState {
    path: Vec<u64>, // Causaloid IDs
    accumulated_evidence: f64,
}
```

**2. The `evaluate_subgraph_from_cause` method is updated:**

The internal logic for graph traversal (e.g., Breadth-First Search) will now use the stateful `bind` operation. At each step, the function applied by `bind`:
1.  Receives the current `GraphTraversalState`.
2.  Adds the current node's ID to `state.path`.
3.  Adds the node's output to `state.accumulated_evidence`.
4.  Returns a new `PropagatingProcess` with the updated state.

This allows the final `PropagatingProcess` returned by the graph evaluation to contain a complete record of its internal execution path and state changes, making it far more powerful for explanation and debugging.

### Impact on Causal Collections

Similarly, evaluating a `Causaloid` of type `Collection` becomes a sequential, stateful process. The `evaluate_collection` method, which previously just aggregated final values, can now be implemented as a monadic fold that threads state through the collection.

**Benefit**: The evaluation of an element `causaloid[i]` can produce a new state `S_i` that becomes the input state for the evaluation of `causaloid[i+1]`.

**Example: Running Average Calculation**

Consider a collection of causaloids, each providing a numerical observation. We want the final result to be the average of all observations.

**1. Define `State` for accumulation:**

```rust
#[derive(Clone, Default, Debug, PartialEq)]
struct RunningAverageState {
    sum: f64,
    count: u32,
}
```

**2. Implement `evaluate_collection` as a stateful fold:**

```rust
// Simplified logic for a collection of <f64, f64> causaloids
fn evaluate_collection(&self, effect: &PropagatingEffect<f64>) -> PropagatingProcess<f64, RunningAverageState, ()> {
    let initial_process = PropagatingProcess::with_state(
        effect.clone(),
        RunningAverageState::default(),
        None,
    );

    self.causal_coll.iter().fold(initial_process, |process, causaloid| {
        // Bind the evaluation of the next causaloid to the accumulated process
        process.bind(|val, state, ctx| {
            // Evaluate the child causaloid, which returns its own process
            let child_process = causaloid.evaluate(val.into_value().unwrap());
            
            // Extract the result and update the state
            let mut new_state = state;
            if let Some(child_val) = child_process.value.into_value() {
                new_state.sum += child_val;
                new_state.count += 1;
            }

            // Construct the next process in the chain
            let mut next_process = CausalMonad::pure(child_process.value.into_value().unwrap_or_default());
            next_process.state = new_state;
            next_process
        })
    })
}
```

This migration transforms the isomorphic data structures from simple containers for aggregation into powerful frameworks for modeling complex, state-dependent causal processes.


## 6. New API in Practice

The new core provides two primary monadic types for different use cases: `PropagatingEffect` for stateless computations and `PropagatingProcess` for stateful ones.

### 6.1 Stateless Propagation with `PropagatingEffect`

Use `PropagatingEffect<T>` when your causal logic is a series of pure transformations, and you only need to carry forward the resulting value, along with any errors or logs.

**Scenario**: A simple data validation and transformation pipeline.
1.  Check if a raw sensor reading (`f64`) is valid (i.e., not negative).
2.  If valid, convert it to an integer percentage.
3.  If it's an error at any point, the chain should stop and report the error.

```rust
use deep_causality_core::{CausalMonad, CausalityError, CausalityErrorEnum, EffectValue, PropagatingEffect};
use deep_causality_haft::{LogAddEntry, MonadEffect5};

// Function 1: Validate sensor reading
fn validate_reading(reading: f64) -> PropagatingEffect<f64> {
    if reading.is_sign_negative() {
        let mut effect = PropagatingEffect::from_error(
            CausalityError::new(CausalityErrorEnum::TypeConversionError)
        );
        effect.logs.add_entry("Validation failed: Negative reading.");
        effect
    } else {
        let mut effect = CausalMonad::pure(reading);
        effect.logs.add_entry("Validation successful.");
        effect
    }
}

// Function 2: Convert to percentage
fn convert_to_percentage(value: f64) -> PropagatingEffect<i32> {
    let pct = (value * 100.0).round() as i32;
    let mut effect = CausalMonad::pure(pct);
    effect.logs.add_entry(&format!("Converted {} to {}%", value, pct));
    effect
}

// ---

// Case 1: Successful execution
let initial_effect = CausalMonad::pure(0.78);
let final_effect = initial_effect
    .bind(|v| validate_reading(v.into_value().unwrap()))
    .bind(|v| convert_to_percentage(v.into_value().unwrap()));

assert!(final_effect.is_ok());
assert_eq!(final_effect.value, EffectValue::Value(78));
println!("---
Success Explanation  {}", final_effect.explain());

// Case 2: Error propagation
let error_effect = CausalMonad::pure(-0.5);
let final_error_effect = error_effect
    .bind(|v| validate_reading(v.into_value().unwrap()))
    .bind(|v| convert_to_percentage(v.into_value().unwrap())); // This step is skipped

assert!(final_error_effect.is_err());
assert_eq!(final_error_effect.value, EffectValue::None); // Value becomes None on error
println!("\n---
Error Explanation  {}", final_error_effect.explain());
```

### 6.2 Stateful Propagation with `PropagatingProcess`

Use `PropagatingProcess<T, S, C>` when you need to maintain and update a state `S` across multiple computation steps, optionally using a read-only context `C`.

**Scenario**: Accumulate a series of observations, but only if they exceed a threshold defined in the context.
-   **State**: `AccumulatorState` will store the `sum` and `count` of valid observations.
-   **Context**: `ProcessingContext` will provide the `threshold`.
-   **Value**: The value propagated at each step will be the observation that was just processed.

```rust
use deep_causality_core::{CausalEffectPropagationProcess, CausalMonad, EffectValue, PropagatingProcess};
use deep_causality_haft::{LogAddEntry, MonadEffect5};

// 1. Define State and Context
#[derive(Debug, Clone, Default, PartialEq)]
struct AccumulatorState {
    sum: f64,
    count: u32,
}

#[derive(Debug, Clone)]
struct ProcessingContext {
    threshold: f64,
}

// 2. Define a stateful causal function
fn process_observation(
    obs: f64,
    mut state: AccumulatorState,
    context: Option<ProcessingContext>,
) -> PropagatingProcess<f64, AccumulatorState, ProcessingContext> {
    let ctx = context.as_ref().unwrap();

    let mut process = CausalMonad::pure(obs); // The output value is the observation itself
    process.context = context;

    if obs > ctx.threshold {
        state.sum += obs;
        state.count += 1;
        process.logs.add_entry(&format!("Accepted observation: {}", obs));
    } else {
        process.logs.add_entry(&format!("Rejected observation: {}", obs));
    }

    process.state = state; // Update the state for the next step
    process
}

// ---

// 3. Set up the initial process
let initial_effect = CausalMonad::pure(0.6); // First observation
let initial_process = PropagatingProcess::with_state(
    initial_effect,
    AccumulatorState::default(),
    Some(ProcessingContext { threshold: 0.5 }),
);

// 4. Chain multiple stateful operations
let process_after_step1 = initial_process.bind(
    |val, state, ctx| process_observation(val.into_value().unwrap(), state, ctx)
);

let process_after_step2 = process_after_step1.bind(
    |_, state, ctx| process_observation(0.4, state, ctx) // Next observation is 0.4
);

let final_process = process_after_step2.bind(
    |_, state, ctx| process_observation(0.9, state, ctx) // Final observation is 0.9
);


// 5. Assert the final state
assert_eq!(final_process.state.count, 2); // 0.6 and 0.9 were accepted
assert!((final_process.state.sum - 1.5).abs() < 1e-9); // 0.6 + 0.9 = 1.5

println!("---
Stateful Process Explanation  {}", final_process.explain());
```

 
## 7. Comprehensive Testing and Validation

The migration to `deep_causality_core` makes causal logic more explicit and easier to test. A key goal of this migration should be to achieve comprehensive test coverage for all new and modified code, including all branches and error conditions.

### Unit Testing Causal Functions

The new monadic and functional approach simplifies unit testing significantly. Causal functions are now purer, receiving all dependencies as explicit inputs.

#### Testing Stateless Functions (`PropagatingEffect`)

Stateless functions are the easiest to test. You provide an input value and assert on the returned `value`, `error`, and `logs`.

```rust
#[test]
fn test_validate_reading_logic() {
    // Test the success path
    let success_result = validate_reading(0.8);
    assert!(success_result.is_ok());
    assert_eq!(success_result.value, EffectValue::Value(0.8));
    assert!(success_result.logs.to_string().contains("Validation successful"));

    // Test the error path (branch coverage)
    let error_result = validate_reading(-1.0);
    assert!(error_result.is_err());
    assert_eq!(error_result.value, EffectValue::None);
    assert!(error_result.logs.to_string().contains("Validation failed"));
    assert_eq!(error_result.error.unwrap().0, CausalityErrorEnum::TypeConversionError);
}
```

#### Testing Stateful Functions (`PropagatingProcess`)

Stateful functions are also straightforward to test because `State` and `Context` are no longer hidden behind shared pointers. You can construct them directly in your test.

```rust
#[test]
fn test_process_observation_logic() {
    // 1. Setup initial state and context for the test
    let initial_state = AccumulatorState { sum: 10.0, count: 1 };
    let context = Some(ProcessingContext { threshold: 0.5 });
    
    // 2. Test the "accepted" branch
    let accepted_obs = 0.9;
    let accepted_process = process_observation(accepted_obs, initial_state.clone(), context.clone());

    // Assert on the new state
    let expected_state_accepted = AccumulatorState { sum: 10.9, count: 2 };
    assert_eq!(accepted_process.state, expected_state_accepted);
    assert!(accepted_process.is_ok());
    assert!(accepted_process.logs.to_string().contains("Accepted observation"));

    // 3. Test the "rejected" branch
    let rejected_obs = 0.4;
    let rejected_process = process_observation(rejected_obs, initial_state.clone(), context.clone());

    // Assert that the state did NOT change
    assert_eq!(rejected_process.state, initial_state); // State remains unchanged
    assert!(rejected_process.is_ok());
    assert!(rejected_process.logs.to_string().contains("Rejected observation"));
}
```

### Mandate for Full Coverage

-   **All Branches**: Every `if/else` and `match` statement within a causal function must have dedicated tests for each branch.
-   **All Error Cases**: Any condition that can produce an error (`PropagatingEffect::from_error(...)`) must be explicitly tested.
-   **Integration Tests**: After unit testing individual functions, write integration tests for entire `Causaloid`s to ensure that `bind` chains and subgraph evaluations behave as expected. Test the full `evaluate` method of each refactored causaloid.

Adhering to these testing principles will ensure the migration is not only a structural improvement but also a significant step up in the system's overall robustness and reliability.

 
## 8. Summary of Key Changes

| Old Concept (`deep_causality`) | New Concept (`deep_causality_core`) | Key Action |
| :--- | :--- | :--- |
| `PropagatingEffect` (struct) | `PropagatingEffect<T>` (stateless alias) | Update type usages to be generic. |
| (Implicit State) | `PropagatingProcess<T, S, C>` (stateful alias) | Model state explicitly with a struct `S`. |
| `EffectValue` (large enum) | `EffectValue<T>` (generic enum) | Wrap concrete values in `EffectValue::Value(T)`. |
| `CausalMonad` (arity-3) | `CausalMonad<S, C>` (arity-5) | Use new monad for state/context-aware chains. |
| `causal_fn` returns `Result` | `causal_fn` returns `PropagatingEffect` | Functions must return the full monadic container. |
| `context` as `Arc<RwLock<T>>` | `context` as a type parameter `C` | Context is now a first-class citizen of the monad. |

This migration modernizes the `deep_causality` crate, aligning it with a more robust and extensible foundation for complex causal reasoning. By making state and context explicit parts of the computation, the new system enhances clarity, testability, and correctness.