# Summary
- **Context**: The `Observable` and `Inferable` traits provide reasoning methods for collections that calculate percentages and metrics over trait implementations.
- **Bug**: The `percent_observation` method in `ObservableReasoning` and three percentage methods in `InferableReasoning` (`percent_inferable`, `percent_inverse_inferable`, `percent_non_inferable`) along with `conjoint_delta` perform division by zero when called on empty collections.
- **Actual vs. expected**: When called on an empty collection, these methods return `NaN` (Not a Number) instead of handling the empty collection case gracefully like the `AssumableReasoning` trait does.
- **Impact**: This bug causes silent data corruption by producing `NaN` values that can propagate through calculations, potentially causing incorrect causal reasoning results or downstream failures in production systems using these percentage calculations.

# Code with bug

**Observable trait** (`deep_causality/src/traits/observable/mod.rs`):
```rust
fn percent_observation(
    &self,
    target_threshold: NumericalValue,
    target_effect: NumericalValue,
) -> NumericalValue {
    self.number_observation(target_threshold, target_effect) / self.len() as NumericalValue
    // * (100 as NumericalValue)  // <-- BUG 🔴 Division by zero when len() == 0
}
```

**Inferable trait** (`deep_causality/src/traits/inferable/mod.rs`):
```rust
fn percent_inferable(&self) -> NumericalValue {
    (self.number_inferable() / self.len() as NumericalValue) * (100 as NumericalValue)
    // <-- BUG 🔴 Division by zero when len() == 0
}

fn percent_inverse_inferable(&self) -> NumericalValue {
    (self.number_inverse_inferable() / self.len() as NumericalValue) * (100 as NumericalValue)
    // <-- BUG 🔴 Division by zero when len() == 0
}

fn percent_non_inferable(&self) -> NumericalValue {
    (self.number_non_inferable() / self.len() as NumericalValue) * (100 as NumericalValue)
    // <-- BUG 🔴 Division by zero when len() == 0
}

fn conjoint_delta(&self) -> NumericalValue {
    let one = 1.0;
    let total = self.len() as NumericalValue;
    let non_inferable = self.number_non_inferable();
    let cum_conjoint = total - non_inferable;

    abs_num(one - (cum_conjoint / total))  // <-- BUG 🔴 Division by zero when len() == 0
}
```

# Evidence

## Example

When a collection implementing `ObservableReasoning` has zero items:
- `self.len()` returns `0`
- `self.len() as NumericalValue` becomes `0.0` (f64)
- `self.number_observation(...)` returns `0.0`
- Division: `0.0 / 0.0` = `NaN`

The same pattern occurs for `InferableReasoning` methods.

## Inconsistency within the codebase

### Reference code
`deep_causality/src/traits/assumable/mod.rs`:
```rust
fn percent_assumption_valid(&self) -> Result<NumericalValue, AssumptionError> {
    if self.is_empty() {
        return Err(AssumptionError::EvaluationFailed(
            "Cannot calculate percentage with zero assumptions".to_string(),
        ));
    }
    let percentage = (self.number_assumption_valid() / self.len() as NumericalValue) * 100.0;
    Ok(percentage)
}
```

### Current code
`deep_causality/src/traits/observable/mod.rs`:
```rust
fn percent_observation(
    &self,
    target_threshold: NumericalValue,
    target_effect: NumericalValue,
) -> NumericalValue {
    self.number_observation(target_threshold, target_effect) / self.len() as NumericalValue
}
```

`deep_causality/src/traits/inferable/mod.rs`:
```rust
fn percent_inferable(&self) -> NumericalValue {
    (self.number_inferable() / self.len() as NumericalValue) * (100 as NumericalValue)
}

fn conjoint_delta(&self) -> NumericalValue {
    let one = 1.0;
    let total = self.len() as NumericalValue;
    let non_inferable = self.number_non_inferable();
    let cum_conjoint = total - non_inferable;

    abs_num(one - (cum_conjoint / total))
}
```

### Contradiction
The `AssumableReasoning` trait correctly handles empty collections by checking `is_empty()` and returning an error, preventing division by zero. However, `ObservableReasoning` and `InferableReasoning` do not implement this safety check, leading to `NaN` values. This inconsistency suggests the latter traits were implemented without considering the edge case that the `AssumableReasoning` trait properly handles.

## Failing test

### Test script
```rust
// Test to reproduce division by zero bug in Observable::percent_observation
use deep_causality::{Observable, ObservableReasoning, Identifiable, NumericalValue};
use std::fmt::Debug;

#[derive(Debug, Clone)]
struct TestObservable {
    id: u64,
    observation: f64,
    effect: f64,
}

impl Identifiable for TestObservable {
    fn id(&self) -> u64 {
        self.id
    }
}

impl Observable for TestObservable {
    fn observation(&self) -> NumericalValue {
        self.observation
    }

    fn observed_effect(&self) -> NumericalValue {
        self.effect
    }
}

struct TestCollection {
    items: Vec<TestObservable>,
}

impl ObservableReasoning<TestObservable> for TestCollection {
    fn len(&self) -> usize {
        self.items.len()
    }

    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    fn get_all_items(&self) -> Vec<&TestObservable> {
        self.items.iter().collect()
    }
}

fn main() {
    // Create empty collection
    let empty_collection = TestCollection {
        items: vec![],
    };

    println!("Testing percent_observation with empty collection...");
    println!("Collection length: {}", empty_collection.len());
    println!("Is empty: {}", empty_collection.is_empty());

    // This will cause division by zero
    let result = empty_collection.percent_observation(0.5, 1.0);

    println!("Result: {}", result);

    if result.is_infinite() || result.is_nan() {
        println!("BUG CONFIRMED: Division by zero produces invalid result!");
        println!("Result is infinite: {}", result.is_infinite());
        println!("Result is NaN: {}", result.is_nan());
    } else {
        println!("No bug detected (unexpected)");
    }
}
```

### Test output
```
Testing percent_observation with empty collection...
Collection length: 0
Is empty: true
Result: NaN
BUG CONFIRMED: Division by zero produces invalid result!
Result is infinite: false
Result is NaN: true
```

# Full context

The `Observable`, `Inferable`, and `Assumable` traits are part of DeepCausality's reasoning system for causal models. These traits provide collection-level reasoning methods that aggregate information from multiple trait implementations.

The `ObservableReasoning` trait is implemented by collections (Vec, arrays, maps) containing `Observable` items, which represent observations in a causal model. The `percent_observation` method calculates what percentage of observations meet certain thresholds and effects.

The `InferableReasoning` trait is implemented by collections of `Inferable` items, which represent inferences in causal reasoning. The percentage methods (`percent_inferable`, `percent_inverse_inferable`, `percent_non_inferable`) and `conjoint_delta` are used to analyze the inferability of causal relationships and estimate the effect of unobserved factors.

These traits are re-exported from the main library (`deep_causality/src/lib.rs`) and are part of the public API. They're used by the model types in `deep_causality/src/types/model_types/`, which implement causal reasoning and inference systems. The `Model` type in particular uses these traits to perform causal analysis on collections of observations, inferences, and assumptions.

When empty collections are passed to these percentage calculation methods, the resulting `NaN` values can:
1. Propagate through subsequent calculations in a causal model
2. Cause logical comparisons to fail unexpectedly (since `NaN != NaN`)
3. Lead to incorrect causal reasoning decisions
4. Make debugging difficult since `NaN` doesn't explicitly indicate the source of the error

# Why has this bug gone undetected?

This bug has gone undetected for several reasons:

1. **Test coverage gap**: All existing tests use non-empty collections. For example, in `deep_causality/tests/extensions/inferable/inferable_vec_tests.rs`, the `test_percent_inferable` function always operates on collections with at least 2 items created by `get_test_inf_vec()`. While tests do verify `is_empty()` functionality, they never call percentage methods on empty collections.

2. **Typical usage patterns**: In production use, these traits are typically used with models that have been populated with data. Empty collections are edge cases that may only occur during initialization, error conditions, or when filtering produces no results.

3. **Silent failure**: The bug produces `NaN` rather than panicking. In Rust, `f64` division by zero doesn't panic - it returns `NaN` for `0.0/0.0` or `Infinity` for `n/0.0`. This means the bug causes silent data corruption rather than an obvious crash, making it much harder to detect.

4. **Inconsistent implementation**: The `AssumableReasoning` trait correctly handles this case, but it returns a `Result` type, which may have led developers to think the simpler traits don't need error handling. This inconsistency suggests the bug was introduced when implementing the `Observable` and `Inferable` traits without referencing the safer pattern from `Assumable`.

5. **Type system doesn't prevent it**: Rust's type system allows division by zero for floating-point types, so there's no compile-time warning about this potential issue.

# Recommended fix

Add empty collection checks similar to the `AssumableReasoning` implementation. There are two possible approaches:

**Option 1: Return Result types** (Breaking change, but safer):
```rust
fn percent_observation(
    &self,
    target_threshold: NumericalValue,
    target_effect: NumericalValue,
) -> Result<NumericalValue, ObservableError> {  // <-- FIX 🟢
    if self.is_empty() {
        return Err(ObservableError::EvaluationFailed(
            "Cannot calculate percentage with zero observations".to_string(),
        ));
    }
    Ok(self.number_observation(target_threshold, target_effect) / self.len() as NumericalValue)
}
```

**Option 2: Return sensible default** (Non-breaking, pragmatic):
```rust
fn percent_observation(
    &self,
    target_threshold: NumericalValue,
    target_effect: NumericalValue,
) -> NumericalValue {
    if self.is_empty() {  // <-- FIX 🟢
        return 0.0;  // 0% when empty
    }
    self.number_observation(target_threshold, target_effect) / self.len() as NumericalValue
}
```

The same fix pattern should be applied to:
- `InferableReasoning::percent_inferable()`
- `InferableReasoning::percent_inverse_inferable()`
- `InferableReasoning::percent_non_inferable()`
- `InferableReasoning::conjoint_delta()`

For `conjoint_delta`, returning `1.0` (maximum delta) for empty collections makes semantic sense, as it indicates complete absence of observed factors.

# Related bugs

The same pattern of missing empty collection checks may exist in:
- `ObservableReasoning::number_non_observation()` - performs subtraction that could theoretically underflow, though less critical
- Any other methods in these traits that perform mathematical operations on `self.len()`

These should be reviewed for similar issues, though the division by zero bugs are the most critical.

# Summary
- **Context**: The `remove_single_state()` method in `deep_causality/src/types/csm_types/csm/state_remove.rs` is part of the CSM (Causal State Machine) API that allows dynamic state management after construction.
- **Bug**: The `id` parameter in `remove_single_state()` is used as the HashMap key for removal, which is inconsistent with how states are keyed during CSM construction via `CSM::new()`, where states are keyed by their internal `state.id()` value.
- **Actual vs. expected**: When states are added via `add_single_state(idx, ...)`, they are keyed by the provided `idx` parameter rather than the state's internal ID, but `CSM::new()` keys states by `state.id()`. This creates inconsistent behavior where the same state can be accessible by different IDs depending on how it was added.
- **Impact**: Users cannot reliably access states by their internal IDs when using mutation methods (`add_single_state`, `update_single_state`, `remove_single_state`, `eval_single_state`), leading to confusion and potential runtime errors.

# Code with bug

`deep_causality/src/types/csm_types/csm/state_remove.rs`:
```rust
pub fn remove_single_state(&self, id: usize) -> Result<(), UpdateError> {
    let mut binding = self.state_actions.write().unwrap();

    if binding.remove(&id).is_none() {  // <-- BUG 🔴 Uses `id` parameter as HashMap key
        return Err(UpdateError(format!(
            "State {id} does not exist and cannot be removed"
        )));
    }

    Ok(())
}
```

The inconsistency originates from the constructor in `deep_causality/src/types/csm_types/csm/mod.rs`:
```rust
pub fn new(state_actions: &[(&CausalState<I, O, C>, &CausalAction)]) -> Self {
    let mut map = CSMMap::with_capacity(state_actions.len());

    for (state, action) in state_actions {
        map.insert(state.id(), ((*state).clone(), (*action).clone()));  // <-- Uses state.id() as key
    }

    Self {
        state_actions: Arc::new(RwLock::new(map)),
    }
}
```

And is perpetuated in `deep_causality/src/types/csm_types/csm/state_add.rs`:
```rust
pub fn add_single_state(
    &self,
    idx: usize,  // <-- Uses arbitrary idx parameter as key
    state_action: StateAction<I, O, C>,
) -> Result<(), UpdateError> {
    if self.state_actions.read().unwrap().contains_key(&idx) {
        return Err(UpdateError(format!("State {idx} already exists.")));
    }

    self.state_actions
        .write()
        .unwrap()
        .insert(idx, state_action);  // <-- Uses idx, not state_action.0.id()

    Ok(())
}
```

# Evidence

## Failing test

### Test script
```rust
/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 *
 * Test to demonstrate ID mismatch bug in CSM
 */

use deep_causality::utils_test::test_utils;
use deep_causality::utils_test::test_utils_csm;
use deep_causality::{CSM, CausalState, PropagatingEffect};

/// This test demonstrates the core bug: when using CSM::new(), states are keyed by their
/// internal state.id(), but when using add_single_state/remove_single_state/eval_single_state,
/// an arbitrary index is used. This creates an inconsistency.
#[test]
fn test_eval_after_add_with_inconsistent_id() {
    // Create initial CSM with one state (ID=42, keyed by 42)
    let state_id = 42;
    let version = 1;
    let data = PropagatingEffect::from_value(0.23f64);
    let causaloid = test_utils::get_test_causaloid_deterministic(23);

    let cs = CausalState::new(state_id, version, data.clone(), causaloid.clone(), None);
    let ca = test_utils_csm::get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    // Create a new state with internal ID = 100
    let cs2 = CausalState::new(100, 2, data.clone(), causaloid, None);
    let ca2 = test_utils_csm::get_test_action();

    // BUG: Add it with map key = 200 (different from internal ID 100)
    let res = csm.add_single_state(200, (cs2, ca2));
    assert!(res.is_ok());
    assert_eq!(csm.len(), 2);

    // Now try to evaluate using the state's internal ID (100)
    let eval_data = PropagatingEffect::from_value(0.6f64);
    let res_by_internal_id = csm.eval_single_state(100, &eval_data);

    // Try to evaluate using the map key (200)
    let res_by_map_key = csm.eval_single_state(200, &eval_data);

    println!("Result by internal ID (100): {:?}", res_by_internal_id);
    println!("Result by map key (200): {:?}", res_by_map_key);

    // BUG MANIFESTATION: Using internal ID fails, but map key works
    // This is confusing and inconsistent with CSM::new() behavior
    assert!(
        res_by_internal_id.is_err(),
        "BUG CONFIRMED: Cannot evaluate state by its internal ID"
    );
    assert!(
        res_by_map_key.is_ok(),
        "Can only evaluate by the arbitrary index passed to add_single_state"
    );
}

/// This test shows that remove_single_state has the same inconsistency
#[test]
fn test_remove_with_inconsistent_id() {
    let state_id = 42;
    let version = 1;
    let data = PropagatingEffect::from_value(0.23f64);
    let causaloid = test_utils::get_test_causaloid_deterministic(23);

    let cs = CausalState::new(state_id, version, data.clone(), causaloid.clone(), None);
    let ca = test_utils_csm::get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    // Create a new state with internal ID = 100
    let cs2 = CausalState::new(100, 2, data, causaloid, None);
    let ca2 = test_utils_csm::get_test_action();

    // Add it with map key = 200 (different from internal ID 100)
    let res = csm.add_single_state(200, (cs2, ca2));
    assert!(res.is_ok());
    assert_eq!(csm.len(), 2);

    // Try to remove using the state's internal ID (100)
    let res_by_internal_id = csm.remove_single_state(100);

    println!("Remove by internal ID (100): {:?}", res_by_internal_id);

    // BUG: Cannot remove by internal ID
    assert!(
        res_by_internal_id.is_err(),
        "BUG CONFIRMED: Cannot remove state by its internal ID"
    );
    assert_eq!(csm.len(), 2, "State was not removed");

    // But can remove by map key
    let res_by_map_key = csm.remove_single_state(200);
    println!("Remove by map key (200): {:?}", res_by_map_key);

    assert!(
        res_by_map_key.is_ok(),
        "Can only remove by the arbitrary index passed to add_single_state"
    );
    assert_eq!(csm.len(), 1);
}

/// This test shows the inconsistency between CSM::new() and add_single_state
#[test]
fn test_new_vs_add_inconsistency() {
    let version = 1;
    let data = PropagatingEffect::from_value(0.23f64);
    let causaloid = test_utils::get_test_causaloid_deterministic(23);

    // Create state with internal ID = 100
    let cs1 = CausalState::new(100, version, data.clone(), causaloid.clone(), None);
    let ca1 = test_utils_csm::get_test_action();

    // Method 1: Add via CSM::new() - uses state.id() as key
    let csm1 = CSM::new(&[(&cs1, &ca1)]);
    let eval_data = PropagatingEffect::from_value(0.6f64);

    // Can evaluate by internal ID (100) because new() used state.id()
    let res = csm1.eval_single_state(100, &eval_data);
    println!("CSM::new - eval by internal ID (100): {:?}", res);
    assert!(res.is_ok(), "CSM::new uses state.id() as key");

    // Method 2: Add via add_single_state with different index
    let cs2 = CausalState::new(100, version, data.clone(), causaloid.clone(), None);
    let ca2 = test_utils_csm::get_test_action();

    let cs_initial = CausalState::new(42, version, data.clone(), causaloid.clone(), None);
    let ca_initial = test_utils_csm::get_test_action();

    let csm2 = CSM::new(&[(&cs_initial, &ca_initial)]);
    csm2.add_single_state(200, (cs2, ca2)).unwrap();

    // Cannot evaluate by internal ID (100) because add_single_state used 200
    let res = csm2.eval_single_state(100, &eval_data);
    println!("add_single_state - eval by internal ID (100): {:?}", res);
    assert!(
        res.is_err(),
        "add_single_state ignores state.id() and uses provided index"
    );

    // BUG: Same state ID (100), different behavior depending on how it was added!
}
```

### Test output
```
running 1 test
Result by internal ID (100): Err(Action(ActionError("State 100 does not exist. Add it first before evaluating.")))
Result by map key (200): Ok(())
test types::csm_types::csm::test_id_mismatch_bug::test_eval_after_add_with_inconsistent_id ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 980 filtered out; finished in 0.00s
```

```
running 1 test
Remove by internal ID (100): Err(UpdateError("State 100 does not exist and cannot be removed"))
Remove by map key (200): Ok(())
test types::csm_types::csm::test_id_mismatch_bug::test_remove_with_inconsistent_id ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 980 filtered out; finished in 0.00s
```

```
running 1 test
CSM::new - eval by internal ID (100): Ok(())
add_single_state - eval by internal ID (100): Err(Action(ActionError("State 100 does not exist. Add it first before evaluating.")))
test types::csm_types::csm::test_id_mismatch_bug::test_new_vs_add_inconsistency ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 980 filtered out; finished in 0.00s
```

## Example

Consider a scenario where a developer creates a CSM and needs to dynamically manage states:

1. Developer creates a state with internal ID 100:
   ```rust
   let state = CausalState::new(100, 1, data, causaloid, None);
   ```

2. Developer adds this state to an existing CSM, using what they think is a reasonable index:
   ```rust
   csm.add_single_state(200, (state, action));
   ```

3. Later, the developer tries to evaluate the state by its internal ID (100), which seems logical:
   ```rust
   csm.eval_single_state(100, &eval_data); // FAILS! State 100 does not exist
   ```

4. The evaluation fails with error: `"State 100 does not exist. Add it first before evaluating."`

5. To make it work, the developer must remember the arbitrary index (200) used in `add_single_state`:
   ```rust
   csm.eval_single_state(200, &eval_data); // Works
   ```

This violates the principle of least surprise and creates a maintenance burden where developers must track two separate ID systems: the state's internal ID and the HashMap key.

## Inconsistency within the codebase

### Reference code
`deep_causality/src/types/csm_types/csm/mod.rs:65-70`
```rust
pub fn new(state_actions: &[(&CausalState<I, O, C>, &CausalAction)]) -> Self {
    let mut map = CSMMap::with_capacity(state_actions.len());

    for (state, action) in state_actions {
        map.insert(state.id(), ((*state).clone(), (*action).clone()));
    }
```

### Current code
`deep_causality/src/types/csm_types/csm/state_remove.rs:17-27`
```rust
pub fn remove_single_state(&self, id: usize) -> Result<(), UpdateError> {
    let mut binding = self.state_actions.write().unwrap();

    if binding.remove(&id).is_none() {
        return Err(UpdateError(format!(
            "State {id} does not exist and cannot be removed"
        )));
    }

    Ok(())
}
```

`deep_causality/src/types/csm_types/csm/state_add.rs:17-34`
```rust
pub fn add_single_state(
    &self,
    idx: usize,
    state_action: StateAction<I, O, C>,
) -> Result<(), UpdateError> {
    if self.state_actions.read().unwrap().contains_key(&idx) {
        return Err(UpdateError(format!("State {idx} already exists.")));
    }

    self.state_actions
        .write()
        .unwrap()
        .insert(idx, state_action);

    Ok(())
}
```

### Contradiction

The constructor `CSM::new()` uses `state.id()` to key states in the internal HashMap. This establishes the expectation that states are identified by their internal ID throughout the CSM API.

However, `add_single_state()` uses the `idx` parameter (not `state_action.0.id()`) as the HashMap key, and `remove_single_state()` uses the `id` parameter as the HashMap key. Similarly, `update_single_state()` and `eval_single_state()` use their respective ID parameters as HashMap keys.

This creates two incompatible ID systems:
1. **CSM::new()**: Uses state's internal ID (`state.id()`)
2. **Mutation methods**: Use arbitrary index parameter (`idx` or `id`)

The result is that states added via `new()` are accessible by their internal IDs, while states added via `add_single_state()` are accessible only by the arbitrary index provided, regardless of their internal ID. This is a fundamental API inconsistency.

# Full context

The CSM (Causal State Machine) is a core type in the deep_causality library that manages relationships between causal states and actions. It maintains an internal HashMap (`CSMMap<I, O, C>` which is `HashMap<usize, StateAction<I, O, C>>`) where states are stored.

The CSM API provides multiple ways to work with states:

1. **Initial construction**: `CSM::new(state_actions: &[(&CausalState, &CausalAction)])` - creates a CSM with initial states
2. **Dynamic state management**:
    - `add_single_state(idx: usize, state_action: StateAction)` - adds a new state
    - `update_single_state(idx: usize, state_action: StateAction)` - updates an existing state
    - `remove_single_state(id: usize)` - removes a state
3. **State evaluation**:
    - `eval_single_state(id: usize, data: &PropagatingEffect)` - evaluates a specific state
    - `eval_all_states()` - evaluates all states

The bug affects all these dynamic operations because they rely on consistent key lookup in the internal HashMap. When states are keyed inconsistently (by internal ID in `new()` vs arbitrary index in mutation methods), operations fail in unexpected ways.

This is particularly problematic for:
- **State evaluation**: `eval_single_state()` needs the correct HashMap key to find and evaluate a state
- **State removal**: `remove_single_state()` needs the correct HashMap key to remove a state
- **State updates**: `update_single_state()` needs the correct HashMap key to update a state

The CSM is used in event-driven systems, monitoring systems, and control systems where reliable state management is critical. This bug undermines that reliability by making state access unpredictable.

# Why has this bug gone undetected?

The bug has gone undetected for several reasons:

1. **Limited test coverage for mixed usage patterns**: The existing tests in `deep_causality/tests/types/csm_types/csm/csm_single_state_tests.rs` only test scenarios where the arbitrary index parameter happens to work correctly. They don't test the natural usage pattern where a developer would want to add a state and then access it by its internal ID.

2. **Tests use matching IDs**: In the existing test `remove_single_state()` (line 112), the test creates a state with ID=2 and adds it with index=43. The test then removes it using index=43 (not ID=2), so it works. The test never attempts to use the state's internal ID, masking the inconsistency.

3. **API design allows arbitrary indices**: The mutation methods accept arbitrary `idx`/`id` parameters without validating that they match the state's internal ID. This gives the appearance that the API is flexible, when in fact it creates an inconsistency with the constructor.

4. **No documentation warning**: The API documentation doesn't warn users about this inconsistency or explain that they need to track both the state's internal ID and the HashMap key separately.

5. **Works in simple scenarios**: For users who only use `CSM::new()` and never use `add_single_state()`, the bug never manifests because all states are consistently keyed by their internal IDs.

6. **Subtle cognitive mismatch**: The parameter name `id` in `remove_single_state()` suggests it should be the state's ID, which reinforces the incorrect mental model that the methods use state IDs. But it's actually just the HashMap key.

# Recommended fix

There are two possible approaches to fix this inconsistency:

## Option 1: Use state.id() consistently (Recommended)

Modify all mutation methods to extract and use the state's internal ID as the HashMap key, making them consistent with `CSM::new()`:

```rust
// state_add.rs
pub fn add_single_state(
    &self,
    state_action: StateAction<I, O, C>,  // <-- FIX 🟢 Remove idx parameter
) -> Result<(), UpdateError> {
    let state_id = state_action.0.id();  // <-- FIX 🟢 Extract state's internal ID

    if self.state_actions.read().unwrap().contains_key(&state_id) {
        return Err(UpdateError(format!("State {state_id} already exists.")));
    }

    self.state_actions
        .write()
        .unwrap()
        .insert(state_id, state_action);

    Ok(())
}
```

This would also require updating `update_single_state()` to extract the state ID from the `state_action` parameter instead of using the `idx` parameter.

For `remove_single_state()`, the method is already correct - it just needs consistent behavior from `add_single_state()`.

**Pros**:
- States are always accessible by their internal ID
- Consistent with constructor behavior
- Simpler mental model for users
- No need to track separate HashMap keys

**Cons**:
- Breaking API change for existing users of `add_single_state()` and `update_single_state()`
- May break existing code that relies on arbitrary indices

## Option 2: Document and validate the current behavior

Keep the arbitrary index approach but add validation and clear documentation:

```rust
pub fn add_single_state(
    &self,
    idx: usize,
    state_action: StateAction<I, O, C>,
) -> Result<(), UpdateError> {
    // Validate that idx matches state's internal ID for consistency
    if idx != state_action.0.id() {
        return Err(UpdateError(format!(
            "Index {idx} does not match state ID {}. They must be equal.",
            state_action.0.id()
        )));
    }

    // ... rest of the method
}
```

**Pros**:
- No breaking changes
- Makes the requirement explicit

**Cons**:
- Still requires tracking both IDs
- More complex mental model
- Doesn't fix the fundamental inconsistency

**Recommendation**: Option 1 is preferred as it creates a more intuitive and maintainable API. However, if backward compatibility is critical, Option 2 provides a path to at least prevent silent failures.

# Related bugs

The same inconsistency exists in the following related methods:

1. `add_single_state()` in `deep_causality/src/types/csm_types/csm/state_add.rs` - uses `idx` parameter instead of `state.id()`
2. `update_single_state()` in `deep_causality/src/types/csm_types/csm/state_update.rs` - uses `idx` parameter instead of `state.id()`
3. `eval_single_state()` in `deep_causality/src/types/csm_types/csm/eval.rs` - uses `id` parameter for lookup (correct behavior, but inconsistent with mutation methods)

All of these methods should be reviewed and fixed together to ensure consistent behavior across the entire CSM API.
