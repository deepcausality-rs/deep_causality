CSM::new silently overwrites duplicate state IDs, causing data loss

# Summary
- **Context**: The `CSM::new` constructor initializes a Causal State Machine by building an internal HashMap that maps state IDs to state-action pairs.
- **Bug**: When multiple states with identical IDs are passed to `CSM::new`, the constructor silently overwrites earlier states with later ones, causing data loss.
- **Actual vs. expected**: The constructor accepts duplicate IDs without error and silently discards all but the last state with each duplicate ID, whereas it should either reject duplicates with an error or preserve all states.
- **Impact**: Users lose state-action pairs silently during initialization, leading to missing functionality and potentially undetected system failures in production.

# Code with bug
```rust
pub fn new(state_actions: &[(&CausalState<I, O, C>, &CausalAction)]) -> Self {
    let mut map = CSMMap::with_capacity(state_actions.len());

    for (state, action) in state_actions {
        map.insert(state.id(), ((*state).clone(), (*action).clone())); // <-- BUG 🔴 Silently overwrites duplicate IDs
    }

    Self {
        state_actions: Arc::new(RwLock::new(map)),
    }
}
```

# Evidence

## Example

Consider this scenario:

```rust
// Two states with the same ID 42 but different data
let cs1 = CausalState::new(42, 1, PropagatingEffect::from_value(0.23), causaloid1, None);
let cs2 = CausalState::new(42, 1, PropagatingEffect::from_value(0.99), causaloid2, None);

let state_actions = &[(&cs1, &ca1), (&cs2, &ca2)];
let csm = CSM::new(state_actions);
```

**Step-by-step execution:**
1. Loop iteration 1: `map.insert(42, (cs1, ca1))` - Map now contains: `{42: (cs1, ca1)}`
2. Loop iteration 2: `map.insert(42, (cs2, ca2))` - HashMap overwrites! Map now contains: `{42: (cs2, ca2)}`
3. Result: `csm.len() == 1` (expected 2 or an error)
4. State cs1 and action ca1 are completely lost without warning

## Inconsistency with API documentation

### Reference API documentation
[Rust HashMap::insert documentation](https://doc.rust-lang.org/std/collections/struct.HashMap.html#method.insert)

> **pub fn insert(&mut self, k: K, v: V) -> Option\<V\>**
>
> Inserts a key-value pair into the map.
>
> If the map did not have this key present, None is returned.
>
> **If the map did have this key present, the value is updated, and the old value is returned.**

### Current API usage
```rust
for (state, action) in state_actions {
    map.insert(state.id(), ((*state).clone(), (*action).clone()));
    // ^^^ Returns Some(old_value) when duplicate exists, but we ignore it
}
```

### Contradiction
The code ignores the return value of `HashMap::insert`, which would be `Some(old_state_action)` when a duplicate ID is inserted. This silent overwrite behavior violates the principle of least surprise and leads to data loss without any indication to the caller.

## Failing test

### Test script
```rust
/*
 * Failing test demonstrating CSM::new silent overwrite bug with duplicate IDs
 */

use deep_causality::utils_test::test_utils;
use deep_causality::utils_test::test_utils_csm;
use deep_causality::{CSM, CausalState, PropagatingEffect};

#[test]
fn test_csm_new_duplicate_ids_should_fail() {
    let version = 1;
    let data1 = PropagatingEffect::from_value(0.23f64);
    let data2 = PropagatingEffect::from_value(0.99f64);
    let causaloid = test_utils::get_test_causaloid_deterministic(23);

    // Create two states with the SAME id
    let duplicate_id = 42;
    let cs1 = CausalState::new(duplicate_id, version, data1, causaloid.clone(), None);
    let cs2 = CausalState::new(duplicate_id, version, data2, causaloid, None);

    let ca1 = test_utils_csm::get_test_action();
    let ca2 = test_utils_csm::get_test_action();

    // Pass both states with duplicate IDs to the constructor
    let state_actions = &[(&cs1, &ca1), (&cs2, &ca2)];
    let csm = CSM::new(state_actions);

    // BUG: CSM::new silently overwrites the first state with the second
    // Expected: Either error or both states present (2 states)
    // Actual: Only 1 state (second one overwrote the first)

    // This assertion FAILS, demonstrating the bug
    assert_eq!(
        csm.len(),
        2,
        "Expected 2 states, but got {} due to silent overwrite",
        csm.len()
    );
}

fn main() {
    println!("Running test_csm_new_duplicate_ids_should_fail...\n");
    test_csm_new_duplicate_ids_should_fail();
}
```

### Test output
```
running 1 test
test test_csm_new_duplicate_ids_should_fail ... FAILED

failures:

---- test_csm_new_duplicate_ids_should_fail stdout ----

thread 'test_csm_new_duplicate_ids_should_fail' (5371) panicked at test_csm_duplicate_bug.rs:33:5:
assertion `left == right` failed: Expected 2 states, but got 1 due to silent overwrite
  left: 1
 right: 2
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    test_csm_new_duplicate_ids_should_fail

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

# Full context

The `CSM` (Causal State Machine) is a core type in the deep_causality library that manages relationships between causal states and actions. It is used to model event-driven systems where specific conditions (states) trigger associated actions.

The constructor `CSM::new` is the primary entry point for initializing a CSM with an initial set of state-action pairs. Users pass an array of state-action tuples, and the constructor builds an internal HashMap keyed by state IDs.

The CSM is used in various contexts:
- **Sensor monitoring systems** (see `examples/csm_examples/csm_basic/main.rs`): Multiple sensors are modeled as states, each with unique IDs (SMOKE_SENSOR=1, FIRE_SENSOR=2, EXPLOSION_SENSOR=3)
- **State evaluation**: Methods like `eval_single_state(id, data)` rely on unique state IDs to look up and evaluate specific states
- **State management**: Methods like `add_single_state`, `update_single_state`, and `remove_single_state` all expect unique IDs and will error if trying to add an existing ID

The bug undermines the assumption that all provided state-action pairs will be registered in the CSM. If a user accidentally provides duplicate IDs (perhaps due to a configuration error, programmatic generation, or copy-paste mistake), the CSM silently accepts them but only keeps the last occurrence.

## External documentation

- [Rust std::collections::HashMap::insert](https://doc.rust-lang.org/std/collections/struct.HashMap.html#method.insert)
```rust
pub fn insert(&mut self, k: K, v: V) -> Option<V>

Inserts a key-value pair into the map.

If the map did not have this key present, None is returned.

If the map did have this key present, the value is updated,
and the old value is returned.
```

# Why has this bug gone undetected?

This bug has remained undetected for several reasons:

1. **Correct usage in examples and tests**: All existing examples and tests use unique state IDs. For instance, `examples/csm_examples/csm_basic/main.rs` uses distinct constants (SMOKE_SENSOR=1, FIRE_SENSOR=2, EXPLOSION_SENSOR=3).

2. **Runtime checks exist elsewhere**: The `add_single_state` method explicitly checks for duplicates and returns an error, which may have created the false assumption that the constructor also validates uniqueness.

3. **Single-developer scenarios**: In simple use cases with a small number of manually created states, developers naturally use unique IDs, making duplicates rare in practice.

4. **No capacity mismatch detection**: The constructor uses `CSMMap::with_capacity(state_actions.len())`, but HashMap doesn't error when the final size is less than the capacity, so there's no runtime indication that states were lost.

5. **Silent failure nature**: Unlike methods that return `Result<(), UpdateError>`, the constructor has no error path. Users who provide duplicate IDs simply get a working CSM with fewer states than expected, with no indication that something went wrong.

6. **Recent refactoring**: The bug was introduced in commit `d4cb2cd6` during a refactoring that simplified CSM types, suggesting it may not have received extensive testing with edge cases like duplicate IDs.

# Recommended fix

Check for duplicate state IDs during construction and return an error:

```rust
pub fn new(state_actions: &[(&CausalState<I, O, C>, &CausalAction)]) -> Result<Self, UpdateError> { // <-- FIX 🟢 Return Result
    let mut map = CSMMap::with_capacity(state_actions.len());

    for (state, action) in state_actions {
        let state_id = state.id();
        if map.insert(state_id, ((*state).clone(), (*action).clone())).is_some() { // <-- FIX 🟢 Check return value
            return Err(UpdateError(format!(
                "Duplicate state ID {state_id} detected. Each state must have a unique ID."
            )));
        }
    }

    Ok(Self {
        state_actions: Arc::new(RwLock::new(map)),
    })
}
```

**Note**: This is a breaking API change, so it requires updating all call sites to handle the `Result` type.

# Related bugs

The same silent overwrite issue exists in `CSM::update_all_states` at `deep_causality/src/types/csm_types/csm/state_update.rs:42-55`:

```rust
pub fn update_all_states(
    &self,
    state_actions: &[(&CausalState<I, O, C>, &CausalAction)],
) -> Result<(), UpdateError> {
    let mut state_map = CSMMap::with_capacity(state_actions.len());

    for (state, action) in state_actions {
        state_map.insert(state.id(), ((*state).clone(), (*action).clone())); // <-- Also silently overwrites
    }

    // Replace the existing map with the newly generated one.
    *self.state_actions.write().unwrap() = state_map;
    Ok(())
}
```

This method should also validate that no duplicate IDs exist in the input array before replacing the internal state map.


Race condition in add_single_state allows duplicate state IDs (non-atomic check-then-insert)

# Summary
- **Context**: The `add_single_state()` method in `deep_causality/src/types/csm_types/csm/state_add.rs` is part of the CSM (Causal State Machine) API that allows dynamic state management in a thread-safe manner using `Arc<RwLock<...>>`.
- **Bug**: The method has a race condition between the duplicate check (line 21) and the insert operation (lines 26-29) because it releases the read lock before acquiring the write lock.
- **Actual vs. expected**: Multiple threads can successfully add the same state ID concurrently, with later additions silently overwriting earlier ones, instead of all but one thread receiving an error as documented.
- **Impact**: In concurrent scenarios, the method violates its documented contract ("Returns UpdateError if a state with that ID already exists"), potentially causing silent data loss and unpredictable behavior in multi-threaded applications.

# Code with bug

`deep_causality/src/types/csm_types/csm/state_add.rs`:
```rust
pub fn add_single_state(&self, state_action: StateAction<I, O, C>) -> Result<(), UpdateError> {
    let state_id = state_action.0.id();

    // Check if the key exists, if so return error
    if self.state_actions.read().unwrap().contains_key(&state_id) {  // <-- BUG 🔴 Read lock released here
        return Err(UpdateError(format!("State {state_id} already exists.")));
    }

    // Insert the new state/action using internal ID
    self.state_actions
        .write()
        .unwrap()  // <-- BUG 🔴 Write lock acquired here, but another thread could have inserted between locks
        .insert(state_id, state_action);

    Ok(())
}
```

This is a classic "check-then-act" race condition where the check and action are not atomic.

# Evidence

## Example

Consider two threads trying to add a state with the same ID (42) concurrently:

**Timeline:**
1. Thread A: Acquires read lock, checks `contains_key(&42)` → `false`
2. Thread A: Releases read lock
3. Thread B: Acquires read lock, checks `contains_key(&42)` → `false` (still doesn't exist)
4. Thread B: Releases read lock
5. Thread A: Acquires write lock, inserts state 42, releases write lock
6. Thread B: Acquires write lock, inserts state 42 (overwrites Thread A's state!), releases write lock

**Result:** Both threads return `Ok(())`, but Thread A's state was silently overwritten by Thread B. The method contract is violated - Thread B should have received an error since state 42 already existed when it tried to insert.

## Failing test

### Test script
```rust
/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

/*
 * Test to demonstrate race condition in CSM::add_single_state
 *
 * This test attempts to add the same state from multiple threads concurrently.
 * The expected behavior is that only one thread succeeds and all others get an error.
 * However, due to the race condition, multiple threads may succeed in overwriting each other.
 */

use deep_causality::utils_test::test_utils;
use deep_causality::utils_test::test_utils_csm;
use deep_causality::{CSM, CausalState, PropagatingEffect};
use std::sync::{Arc, Barrier};
use std::thread;

#[test]
fn test_race_condition_add_single_state() {
    const NUM_THREADS: usize = 20;
    const NUM_ITERATIONS: usize = 50;

    println!("\nTesting for race condition in CSM::add_single_state");
    println!("Running {} iterations with {} threads each", NUM_ITERATIONS, NUM_THREADS);

    let mut race_detected = false;

    for iteration in 0..NUM_ITERATIONS {
        // Create a CSM with an initial state
        let initial_state = CausalState::new(
            1,
            1,
            PropagatingEffect::from_value(0.5f64),
            test_utils::get_test_causaloid_deterministic(23),
            None,
        );
        let initial_action = test_utils_csm::get_test_action();
        let csm = Arc::new(CSM::new(&[(&initial_state, &initial_action)]));

        // Use a barrier to synchronize thread starts for maximum contention
        let barrier = Arc::new(Barrier::new(NUM_THREADS));

        let mut handles = vec![];

        // Spawn threads that all try to add the same state (ID=42)
        for thread_id in 0..NUM_THREADS {
            let csm_clone = Arc::clone(&csm);
            let barrier_clone = Arc::clone(&barrier);

            let handle = thread::spawn(move || {
                // Wait for all threads to be ready
                barrier_clone.wait();

                // All threads try to add state with ID=42 simultaneously
                let state = CausalState::new(
                    42,
                    1,
                    PropagatingEffect::from_value(0.7f64),
                    test_utils::get_test_causaloid_deterministic(23),
                    None,
                );
                let action = test_utils_csm::get_test_action();

                let result = csm_clone.add_single_state((state, action));
                (thread_id, result)
            });

            handles.push(handle);
        }

        // Collect results
        let mut success_count = 0;
        let mut error_count = 0;

        for handle in handles {
            let (_thread_id, result) = handle.join().unwrap();
            if result.is_ok() {
                success_count += 1;
            } else {
                error_count += 1;
            }
        }

        // Check for race condition: if more than 1 thread succeeded, we have a problem
        if success_count > 1 {
            println!("\n>>> Iteration {}: RACE CONDITION DETECTED! <<<", iteration);
            println!("    {} threads succeeded (expected: 1)", success_count);
            println!("    {} threads failed (expected: {})", error_count, NUM_THREADS - 1);
            race_detected = true;
        }

        // Verify final state
        assert_eq!(csm.len(), 2, "CSM should have exactly 2 states (initial + added)");
    }

    println!("\n=== RESULTS ===");
    println!("Total iterations: {}", NUM_ITERATIONS);
    println!("Race conditions detected: {}", if race_detected { "YES" } else { "NO" });

    if race_detected {
        println!("\nBUG CONFIRMED: Race condition exists in add_single_state()");
        println!("Multiple threads were able to add the same state concurrently,");
        println!("violating the method's contract that it should return an error");
        println!("if the state already exists.\n");
        panic!("Race condition detected in add_single_state()");
    } else {
        println!("\nNo race condition detected in this run.");
        println!("Note: Race conditions are timing-dependent and may not always manifest.");
        println!("The bug still exists in the code structure (check-then-act with separate locks).\n");
    }
}
```

### Test output
```
Testing for race condition in CSM::add_single_state
Running 50 iterations with 20 threads each

>>> Iteration 0: RACE CONDITION DETECTED! <<<
    2 threads succeeded (expected: 1)
    18 threads failed (expected: 19)

>>> Iteration 10: RACE CONDITION DETECTED! <<<
    2 threads succeeded (expected: 1)
    18 threads failed (expected: 19)

>>> Iteration 43: RACE CONDITION DETECTED! <<<
    2 threads succeeded (expected: 1)
    18 threads failed (expected: 19)

>>> Iteration 44: RACE CONDITION DETECTED! <<<
    2 threads succeeded (expected: 1)
    18 threads failed (expected: 19)

=== RESULTS ===
Total iterations: 50
Race conditions detected: YES

BUG CONFIRMED: Race condition exists in add_single_state()
Multiple threads were able to add the same state concurrently,
violating the method's contract that it should return an error
if the state already exists.

thread 'types::csm_types::csm::csm_race_condition_test::test_race_condition_add_single_state' panicked at deep_causality/tests/types/csm_types/csm/csm_race_condition_test.rs:107:9:
Race condition detected in add_single_state()
```

The test demonstrates that in 4 out of 50 iterations (8%), multiple threads successfully added the same state, violating the method's contract.

## Inconsistency within the codebase

### Reference code

`deep_causality/src/types/csm_types/csm/state_remove.rs` (correctly handles atomicity):
```rust
pub fn remove_single_state(&self, id: usize) -> Result<(), UpdateError> {
    let mut binding = self.state_actions.write().unwrap();  // <-- Acquires write lock once

    if binding.remove(&id).is_none() {  // <-- Check and remove within same lock
        return Err(UpdateError(format!(
            "State {id} does not exist and cannot be removed"
        )));
    }

    Ok(())
}
```

### Current code

`deep_causality/src/types/csm_types/csm/state_add.rs`:
```rust
pub fn add_single_state(&self, state_action: StateAction<I, O, C>) -> Result<(), UpdateError> {
    let state_id = state_action.0.id();

    // Check if the key exists, if so return error
    if self.state_actions.read().unwrap().contains_key(&state_id) {
        return Err(UpdateError(format!("State {state_id} already exists.")));
    }

    // Insert the new state/action using internal ID
    self.state_actions
        .write()
        .unwrap()
        .insert(state_id, state_action);

    Ok(())
}
```

### Comparison

The `remove_single_state()` method correctly acquires a write lock once and performs both the check (`remove().is_none()`) and action (removal) within the same lock, ensuring atomicity.

However, `add_single_state()` acquires a read lock for the check, releases it, then acquires a write lock for the insert. This creates a window where another thread can modify the map between the two operations, breaking atomicity.

# Full context

The CSM (Causal State Machine) is a core type in the deep_causality library designed for concurrent use, as evidenced by its internal `Arc<RwLock<CSMMap<I, O, C>>>` structure. The type is documented as being useful for "event-driven systems", "monitoring systems", and "control systems" - all of which commonly involve concurrent access patterns.

The CSM provides these state management methods:
- `new()` - Initial construction with states
- `add_single_state()` - Adds a new state dynamically (has race condition)
- `update_single_state()` - Updates an existing state (has same race condition)
- `remove_single_state()` - Removes a state (correctly atomic)
- `eval_single_state()` - Evaluates a state
- `eval_all_states()` - Evaluates all states

The bug affects operations that need to enforce invariants:
- **add_single_state()**: Must ensure no duplicate IDs are added
- **update_single_state()**: Must ensure only existing states are updated

When used in multi-threaded scenarios (which the `Arc<RwLock<...>>` design explicitly supports), the race condition can cause:
1. **Silent data loss**: Later thread overwrites earlier thread's state without either thread knowing
2. **Contract violation**: Multiple threads receive `Ok(())` when the contract says only one should succeed
3. **Unpredictable behavior**: The final state depends on thread scheduling, making bugs non-deterministic and hard to reproduce

The CSM is used in systems where reliable state management is critical. This bug undermines that reliability.

# Why has this bug gone undetected?

This bug has gone undetected for several reasons:

1. **Test coverage focused on single-threaded scenarios**: All existing tests in `deep_causality/tests/types/csm_types/csm/csm_single_state_tests.rs` execute operations sequentially from a single thread, never exercising concurrent access patterns.

2. **Race conditions are timing-dependent**: The bug only manifests when specific thread interleaving occurs, which may be rare in testing environments with fast execution and low CPU contention. Our test detected the race in only 8% of iterations (4 out of 50).

3. **Real-world usage may not trigger it**: If users primarily add states during initialization (before multi-threading begins) or use external synchronization (like channels or mutexes), the race window may never be hit in practice.

4. **Introduced during a fix**: The bug was introduced in commit `4d762df1` ("fix(deep_causality): Fixed a number of bugs. Updated tests for verification.") when fixing the ID mismatch bug. The fix correctly changed the method to use `state_action.0.id()` instead of an arbitrary `idx` parameter, but kept the unsafe two-lock pattern, possibly due to:
   - Performance optimization attempt (read locks are cheaper than write locks)
   - Not recognizing the check-then-act race condition pattern
   - Copying the pattern from `update_single_state()` which had the same issue

5. **Rust's type system doesn't prevent it**: The RwLock API allows separate read and write lock acquisitions, so the compiler cannot detect that atomicity is broken. This is a logical error, not a type system violation.

6. **Silent failure mode**: Rather than panicking, the bug causes silent overwrites, making it harder to detect. Both threads return `Ok(())`, so there's no immediate indication that something went wrong.

# Recommended fix

Acquire the write lock once and perform both the check and insert atomically, similar to how `remove_single_state()` is correctly implemented.

## Fixed code for add_single_state

```rust
pub fn add_single_state(&self, state_action: StateAction<I, O, C>) -> Result<(), UpdateError> {
    let state_id = state_action.0.id();

    // Acquire write lock once and hold it for both check and insert  // <-- FIX 🟢
    let mut binding = self.state_actions.write().unwrap();

    // Check if the key exists, if so return error
    if binding.contains_key(&state_id) {
        return Err(UpdateError(format!("State {state_id} already exists.")));
    }

    // Insert the new state/action using internal ID
    binding.insert(state_id, state_action);

    Ok(())
}
```

## Fixed code for update_single_state

`deep_causality/src/types/csm_types/csm/state_update.rs` has the same pattern and needs the same fix:

```rust
pub fn update_single_state(
    &self,
    state_action: StateAction<I, O, C>,
) -> Result<(), UpdateError> {
    let state_id = state_action.0.id();

    // Acquire write lock once and hold it for both check and update  // <-- FIX 🟢
    let mut binding = self.state_actions.write().unwrap();

    // Check if the key exists, if not return error
    if !binding.contains_key(&state_id) {
        return Err(UpdateError(format!(
            "State {state_id} does not exist. Add it first before updating."
        )));
    }

    // Update state/action using internal ID
    binding.insert(state_id, state_action);

    Ok(())
}
```

**Trade-offs:**
- **Con**: Holding a write lock for the check is slightly less performant than using a read lock, as it blocks all readers
- **Pro**: Ensures correctness and atomicity, which is critical for concurrent systems
- **Pro**: Consistent with the `remove_single_state()` implementation
- **Pro**: Simple and clear - the entire operation is atomic

**Note**: The performance difference is negligible in practice because:
1. The check (`contains_key`) is O(1) and very fast
2. The insert is also O(1)
3. The entire critical section is just two HashMap operations
4. Correctness is more important than microsecond-level performance optimizations

# Related bugs

The same race condition exists in:
1. `update_single_state()` in `deep_causality/src/types/csm_types/csm/state_update.rs` - Lines 24-34 use the same unsafe pattern (read lock for check, separate write lock for update)

Both methods should be fixed together to ensure consistent behavior across the CSM API.

CSM mutation methods key states by arbitrary indices, inconsistent with constructor (state.id())

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
