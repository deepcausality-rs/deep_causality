execute_causal_logic drops error when value is present (violates error precedence)

# Summary
- **Context**: The `execute_causal_logic` function in `causable_utils.rs` converts a stateful `PropagatingProcess` returned by context-aware causal functions into a stateless `PropagatingEffect` for singleton causaloids.
- **Bug**: When a `PropagatingProcess` contains both a value and an error, the function silently drops the error and only returns the value.
- **Actual vs. expected**: The function only checks if `process.value.into_value()` is `Some`, ignoring `process.error` entirely; it should check `process.error` first and propagate it, following monadic error-handling semantics.
- **Impact**: Errors from user-defined causal functions are silently lost, breaking error propagation and auditability in causal chains, potentially causing incorrect reasoning results and masking critical failures.

# Code with bug
```rust
// File: deep_causality/src/types/causal_types/causaloid/causable_utils.rs, lines 44-57

let mut effect = match process.value.into_value() {
    Some(val) => PropagatingEffect::pure(val),  // <-- BUG 🔴 Ignores process.error
    None => {
        let error = process.error.unwrap_or_else(|| {
            CausalityError(deep_causality_core::CausalityErrorEnum::Custom(
                "execute_causal_logic: context_fn returned None value and no error"
                    .into(),
            ))
        });
        PropagatingEffect::from_error(error)
    }
};
effect.logs = process.logs;
effect
```

The bug is on line 45: when `process.value.into_value()` returns `Some(val)`, the code creates a `PropagatingEffect::pure(val)` which sets `error` to `None`, completely ignoring any error that might exist in `process.error`.

# Evidence

## Example

Consider a context-aware causal function that computes a result but also encounters a non-fatal error condition:

```rust
fn problematic_fn(
    obs: EffectValue<f64>,
    _state: (),
    _ctx: Option<Arc<RwLock<BaseContext>>>,
) -> PropagatingProcess<f64, (), Arc<RwLock<BaseContext>>> {
    let mut process = PropagatingProcess::pure(42.0); // Returns a value
    process.error = Some(CausalityError::new(CausalityErrorEnum::Custom(
        "Warning: Using fallback value due to invalid data".into(),
    )));
    process
}
```

**Step-by-step trace:**

1. User calls `causaloid.evaluate(&effect)` where the causaloid uses `problematic_fn`
2. `execute_causal_logic` is called
3. `context_fn` is invoked, returning `process` with:
    - `process.value` = `EffectValue::Value(42.0)`
    - `process.error` = `Some(CausalityError(...))`
4. `process.value.into_value()` returns `Some(42.0)`
5. The code takes the first branch: `PropagatingEffect::pure(42.0)`
6. This creates a new effect with `error = None`, **discarding the error from step 3**
7. The function returns success (value 42.0, no error)

**Expected behavior:** The error should be preserved and propagated, signaling that while a value was computed, an error condition was encountered.

**Actual behavior:** The error is silently dropped, and the causaloid evaluation appears to succeed without issues.

## Inconsistency with API documentation

### Reference: CausalEffectPropagationProcess documentation

From `deep_causality_core/src/types/causal_effect_propagation_process/mod.rs`:

```rust
/// The current error state. If `Some`, new `bind` operations will skip execution.
pub error: Option<Error>,
```

The documentation explicitly states that if `error` is `Some`, the process is in an error state. The `bind` method implementation confirms this:

```rust
pub fn bind<F, NewValue>(self, f: F) -> CausalEffectPropagationProcess<...> {
    if let Some(error) = self.error {
        return CausalEffectPropagationProcess {
            value: EffectValue::default(),
            state: self.state,
            context: self.context,
            error: Some(error),  // <-- Error takes precedence
            logs: self.logs,
        };
    }
    // ... continue with computation only if no error
}
```

### Current code in execute_causal_logic

```rust
let mut effect = match process.value.into_value() {
    Some(val) => PropagatingEffect::pure(val),  // Creates effect with error = None
    None => { /* handle error */ }
};
```

### Contradiction

The `bind` method checks `if let Some(error) = self.error` **first**, before looking at the value, establishing that errors take precedence in the monadic chain. However, `execute_causal_logic` only checks `process.value.into_value()`, violating this semantic contract. When converting from `PropagatingProcess` to `PropagatingEffect`, the error should be checked first and take precedence over the value.

## Failing test

### Test script

```rust
/*
 * Test to demonstrate the bug where execute_causal_logic ignores errors
 * when a value is present in the PropagatingProcess.
 */

use deep_causality::*;
use deep_causality::utils_test::test_utils::get_base_context;
use deep_causality_core::{CausalityErrorEnum, EffectValue};
use std::sync::{Arc, RwLock};

#[test]
fn test_error_silently_dropped_when_value_present() {
    let id: IdentificationValue = 99;
    let description = "test error dropped bug";
    let context = get_base_context();

    // This function returns BOTH a value AND an error
    // This simulates a function that computed a result but also encountered a non-fatal error
    fn problematic_fn(
        _obs: EffectValue<f64>,
        _state: (),
        _ctx: Option<Arc<RwLock<BaseContext>>>,
    ) -> PropagatingProcess<f64, (), Arc<RwLock<BaseContext>>> {
        let mut process = PropagatingProcess::pure(42.0); // Returns a value
        process.error = Some(CausalityError::new(CausalityErrorEnum::Custom(
            "This error should not be ignored!".into(),
        ))); // But also sets an error
        process
    }

    let causaloid = BaseCausaloid::<f64, f64>::new_with_context(
        id,
        problematic_fn,
        Arc::new(RwLock::new(context)),
        description,
    );

    let effect = PropagatingEffect::from_value(1.0);
    let result = causaloid.evaluate(&effect);

    println!("Result value: {:?}", result.value);
    println!("Result error: {:?}", result.error);

    // The bug: The error is silently dropped!
    // The user's function returned BOTH a value and an error
    // But execute_causal_logic only checks if value.into_value() is Some
    // If it is, it ignores the error field completely

    // This assertion SHOULD pass (error should be preserved)
    // But due to the bug, it will fail
    assert!(
        result.error.is_some(),
        "Error should be preserved even when value is present, but it was silently dropped!"
    );
}
```

### Test output

```
running 1 test
Result value: Value(42.0)
Result error: None

thread 'test_error_silently_dropped_when_value_present' panicked at deep_causality/tests/bug_test_error_silently_dropped.rs:51:5:
Error should be preserved even when value is present, but it was silently dropped!
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test test_error_silently_dropped_when_value_present ... FAILED

failures:
    test_error_silently_dropped_when_value_present

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
```

The test output clearly shows:
- `Result value: Value(42.0)` - The value was preserved
- `Result error: None` - **The error was silently dropped**

The test fails with the message: "Error should be preserved even when value is present, but it was silently dropped!"

# Full context

The `execute_causal_logic` function is called from `causable.rs` in the `MonadicCausable::evaluate` implementation for singleton causaloids. This is the core evaluation path that processes every singleton causaloid with a context-aware causal function (`ContextualCausalFn`).

When a user creates a singleton causaloid with `Causaloid::new_with_context()`, they provide a context-aware function with signature:
```rust
fn(EffectValue<I>, S, Option<C>) -> PropagatingProcess<O, S, C>
```

This function returns a `PropagatingProcess`, which is a stateful causal computation that can carry:
- A value (`EffectValue<O>`)
- State (`S`)
- Context (`Option<C>`)
- **Error (`Option<CausalityError>`)**
- Logs (`EffectLog`)

The `execute_causal_logic` function must convert this stateful `PropagatingProcess` into a stateless `PropagatingEffect` (which drops state and context but preserves value, error, and logs). However, it fails to properly preserve the error field when a value is present.

This bug affects all singleton causaloids that use context-aware functions and could return both a value and an error. While it's uncommon for user functions to return both simultaneously, it's a valid state in the type system and represents scenarios like:
- Computing a fallback value after encountering an error
- Producing a partial result with warnings/errors
- Implementing recovery mechanisms that yield a value but flag an issue

The monadic chain built by `evaluate` uses `bind` operations which properly check and propagate errors. However, `execute_causal_logic` breaks this chain by silently dropping errors, violating the monadic error-handling contract.

# Why has this bug gone undetected?

This bug has remained undetected for several reasons:

1. **Uncommon usage pattern**: Most user-defined causal functions follow the standard monadic pattern of returning either success (value + no error) or failure (no value + error). Setting both a value and an error simultaneously is rare.

2. **Type system allows it but convention discourages it**: While `PropagatingProcess` structurally allows both `value` and `error` to be `Some` simultaneously, the common convention is to treat them as mutually exclusive states (similar to `Result<T, E>`).

3. **No existing tests for edge case**: The test suite includes tests for:
    - Successful execution (value, no error)
    - Failed execution (no value, with error)
    - None value with no error (generates synthetic error)

   But there are no tests covering the case where a user function returns both a value and an error.

4. **API migration introduced the bug**: The bug was introduced during the refactoring from `Result`-based API to `PropagatingProcess`-based API in commit `89a73199`. The original implementation correctly checked `match result { Ok(output) => ..., Err(e) => ... }`, but the new implementation only checks `match process.value.into_value() { Some(val) => ..., None => ... }`, omitting the error check.

5. **The converted code compiles and runs**: The buggy code has no compilation errors or runtime crashes. It only exhibits incorrect behavior in the edge case scenario.

6. **Logs are preserved**: Since `effect.logs = process.logs` is executed after the match statement, logs are correctly propagated, which may have masked the missing error propagation during testing and code review.

# Recommended fix

The function should check `process.error` **before** checking `process.value`, following the monadic error-handling semantics where errors take precedence:

```rust
let mut effect = if let Some(error) = process.error {
    // Error takes precedence - propagate it regardless of value
    PropagatingEffect::from_error(error)  // <-- FIX 🟢
} else {
    // No error - check if we have a value
    match process.value.into_value() {
        Some(val) => PropagatingEffect::pure(val),
        None => PropagatingEffect::from_error(CausalityError(
            deep_causality_core::CausalityErrorEnum::Custom(
                "execute_causal_logic: context_fn returned None value and no error".into(),
            )
        ))
    }
};
effect.logs = process.logs;
effect
```

This ensures that:
1. Errors are never silently dropped
2. The behavior matches the monadic error-handling contract established by `bind`
3. Error propagation is maintained throughout the causal chain
4. The audit trail (logs) remains complete


execute_causal_logic drops non-Value EffectValue variants and returns error

# Summary
- **Context**: The `execute_causal_logic` function in `causable_utils.rs` converts `PropagatingProcess` results from context-aware causal functions to `PropagatingEffect` for use in singleton causaloid evaluation chains.
- **Bug**: When a `context_causal_fn` returns non-`Value` variants of `EffectValue` (like `ContextualLink`, `RelayTo`, or `Map`), they are incorrectly treated as errors instead of being preserved.
- **Actual vs. expected**: Non-`Value` variants are converted to errors with the message "context_fn returned None value and no error", when they should be preserved as valid effect values.
- **Impact**: Context-aware causaloids cannot return advanced effect types like `ContextualLink` (for lazy data fetching) or `RelayTo` (for dynamic graph navigation), severely limiting the expressiveness of the causal reasoning system.

# Code with bug
```rust
// In execute_causal_logic function (lines 28-52)
if let Some(context_fn) = &causaloid.context_causal_fn {
    if let Some(context) = causaloid.context.as_ref() {
        let ev = EffectValue::from(input);
        let process = context_fn(ev, PS::default(), Some(context.clone()));

        // Convert PropagatingProcess to PropagatingEffect, preserving logs.
        let mut effect = match process.value.into_value() {  // <-- BUG 🔴: into_value() only extracts Value variants
            Some(val) => PropagatingEffect::pure(val),
            None => {
                let error = process.error.unwrap_or_else(|| {
                    CausalityError(deep_causality_core::CausalityErrorEnum::Custom(
                        "execute_causal_logic: context_fn returned None value and no error"
                            .into(),
                    ))
                });
                PropagatingEffect::from_error(error)
            }
        };
        effect.logs = process.logs;
        effect
    }
    // ... rest of function
}
```

The bug is on line 39 where `process.value.into_value()` is called. This method only returns `Some(v)` for `EffectValue::Value(v)` and returns `None` for all other variants.

# Evidence

## Example

Consider a context function that returns a `ContextualLink` to enable lazy data fetching:

```rust
fn contextual_causal_fn(
    _obs: EffectValue<f64>,
    _state: (),
    _ctx: Option<Arc<RwLock<BaseContext>>>,
) -> PropagatingProcess<bool, (), Arc<RwLock<BaseContext>>> {
    // Return a link to contextoid IDs (42, 100) for lazy fetching
    let contextual_link = EffectValue::ContextualLink(42, 100);
    PropagatingProcess::from_effect_value(contextual_link)
}
```

**Step-by-step execution:**
1. `process.value` = `EffectValue::ContextualLink(42, 100)`
2. `process.value.into_value()` is called
3. Since `ContextualLink` is not a `Value` variant, `into_value()` returns `None`
4. The code enters the `None` branch and creates an error
5. Result: Error message "context_fn returned None value and no error"

**Expected behavior:** The `ContextualLink(42, 100)` should be preserved and returned in the `PropagatingEffect`.

## Inconsistency within the codebase

### Reference code
`deep_causality/src/types/causal_types/causaloid/causable.rs` lines 92-95:
```rust
.bind(|output_effect_val, _, _| match output_effect_val {
    EffectValue::Value(v) => causable_utils::log_output(v, self.id),
    _ => PropagatingEffect::from_effect_value(output_effect_val),  // <-- Correctly handles all variants
})
```

### Current code
`deep_causality/src/types/causal_types/causaloid/causable_utils.rs` lines 39-49:
```rust
let mut effect = match process.value.into_value() {  // <-- Only handles Value variant
    Some(val) => PropagatingEffect::pure(val),
    None => {
        let error = process.error.unwrap_or_else(|| {
            CausalityError(deep_causality_core::CausalityErrorEnum::Custom(
                "execute_causal_logic: context_fn returned None value and no error"
                    .into(),
            ))
        });
        PropagatingEffect::from_error(error)
    }
};
```

### Contradiction
In `causable.rs`, the evaluation logic correctly uses `PropagatingEffect::from_effect_value()` to handle all `EffectValue` variants when they're not `Value`. However, in `causable_utils.rs`, the `execute_causal_logic` function only handles the `Value` variant via `into_value()`, treating all other valid variants as errors.

## Failing test

### Test script
```rust
/*
 * Test to reproduce the bug where context_causal_fn returning
 * EffectValue::ContextualLink is incorrectly treated as an error
 */

use deep_causality::*;
use deep_causality_core::EffectValue;
use std::sync::{Arc, RwLock};

#[test]
fn test_context_fn_returning_contextual_link() {
    let id: IdentificationValue = 1;
    let description = "Tests context function returning ContextualLink";
    let context = Arc::new(RwLock::new(deep_causality::utils_test::test_utils::get_base_context()));

    // This context function returns a ContextualLink instead of a Value
    fn contextual_causal_fn(
        _obs: EffectValue<NumericalValue>,
        _state: (),
        _ctx: Option<Arc<RwLock<BaseContext>>>,
    ) -> PropagatingProcess<bool, (), Arc<RwLock<BaseContext>>> {
        // Return a ContextualLink - this is a valid effect value that should be preserved
        let contextual_link = EffectValue::ContextualLink(42, 100);

        PropagatingProcess::from_effect_value(contextual_link)
    }

    let causaloid = BaseCausaloid::<NumericalValue, bool>::new_with_context(
        id,
        contextual_causal_fn,
        context,
        description,
    );

    let input_effect = PropagatingEffect::from_value(0.5);
    let result = causaloid.evaluate(&input_effect);

    // The result should preserve the ContextualLink, not treat it as an error
    assert!(result.is_ok(), "Result should not be an error. Got error: {:?}", result.error);
    assert!(
        matches!(result.value, EffectValue::ContextualLink(42, 100)),
        "Expected ContextualLink(42, 100), got {:?}",
        result.value
    );
}

#[test]
fn test_context_fn_returning_relay_to() {
    let id: IdentificationValue = 2;
    let description = "Tests context function returning RelayTo";
    let context = Arc::new(RwLock::new(deep_causality::utils_test::test_utils::get_base_context()));

    // This context function returns a RelayTo instead of a Value
    fn contextual_causal_fn(
        _obs: EffectValue<NumericalValue>,
        _state: (),
        _ctx: Option<Arc<RwLock<BaseContext>>>,
    ) -> PropagatingProcess<bool, (), Arc<RwLock<BaseContext>>> {
        // Return a RelayTo - this is a valid effect value that should be preserved
        let relay_effect = PropagatingEffect::from_value(true);
        let relay_to = EffectValue::RelayTo(5, Box::new(relay_effect));

        PropagatingProcess::from_effect_value(relay_to)
    }

    let causaloid = BaseCausaloid::<NumericalValue, bool>::new_with_context(
        id,
        contextual_causal_fn,
        context,
        description,
    );

    let input_effect = PropagatingEffect::from_value(0.5);
    let result = causaloid.evaluate(&input_effect);

    // The result should preserve the RelayTo, not treat it as an error
    assert!(result.is_ok(), "Result should not be an error. Got error: {:?}", result.error);
    assert!(
        matches!(result.value, EffectValue::RelayTo(5, _)),
        "Expected RelayTo(5, _), got {:?}",
        result.value
    );
}

#[test]
fn test_context_fn_returning_none_should_error() {
    let id: IdentificationValue = 3;
    let description = "Tests context function returning None";
    let context = Arc::new(RwLock::new(deep_causality::utils_test::test_utils::get_base_context()));

    // This context function returns None - which should be treated as an error
    fn contextual_causal_fn(
        _obs: EffectValue<NumericalValue>,
        _state: (),
        _ctx: Option<Arc<RwLock<BaseContext>>>,
    ) -> PropagatingProcess<bool, (), Arc<RwLock<BaseContext>>> {
        // Return None with no error set - should trigger the default error
        PropagatingProcess::from_effect_value(EffectValue::None)
    }

    let causaloid = BaseCausaloid::<NumericalValue, bool>::new_with_context(
        id,
        contextual_causal_fn,
        context,
        description,
    );

    let input_effect = PropagatingEffect::from_value(0.5);
    let result = causaloid.evaluate(&input_effect);

    // None should be treated as an error
    assert!(result.is_err(), "Result should be an error when value is None");
}
```

### Test output
```
running 3 tests
test test_context_fn_returning_none_should_error ... ok
test test_context_fn_returning_contextual_link ... FAILED
test test_context_fn_returning_relay_to ... FAILED

failures:

---- test_context_fn_returning_contextual_link stdout ----

thread 'test_context_fn_returning_contextual_link' (5230) panicked at deep_causality/tests/contextual_link_bug_test.rs:39:5:
Result should not be an error. Got error: Some(CausalityError(Custom("execute_causal_logic: context_fn returned None value and no error")))
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- test_context_fn_returning_relay_to stdout ----

thread 'test_context_fn_returning_relay_to' (5232) panicked at deep_causality/tests/contextual_link_bug_test.rs:77:5:
Result should not be an error. Got error: Some(CausalityError(Custom("execute_causal_logic: context_fn returned None value and no error")))


failures:
    test_context_fn_returning_contextual_link
    test_context_fn_returning_relay_to

test result: FAILED. 1 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

The tests clearly demonstrate that `ContextualLink` and `RelayTo` are treated as errors, while `None` (which should be an error) is correctly handled.

# Full context

The `execute_causal_logic` function is called from the `evaluate` method in `causable.rs` (line 89) as part of the monadic evaluation chain for singleton causaloids. When a causaloid has a `context_causal_fn`, this function:

1. Receives the input value
2. Wraps it in an `EffectValue`
3. Calls the context-aware causal function, which returns a `PropagatingProcess<O, PS, C>`
4. Converts the result to `PropagatingEffect<O>` (stateless, no context)
5. Returns the effect for further processing

The `PropagatingEffect` is then logged and returned to the caller. This is a critical path in the causal reasoning engine, as it determines how context-aware causaloids propagate effects through the causal graph.

The `EffectValue` enum supports several advanced features:
- `ContextualLink`: Enables lazy data fetching by referencing contextoid IDs
- `RelayTo`: Enables dynamic graph navigation by jumping to specific causaloid indices
- `Map`: Enables structured data passing with named values

These features are essential for building sophisticated causal reasoning systems, particularly for:
- **Adaptive reasoning**: Using `RelayTo` to dynamically navigate based on intermediate results
- **Lazy evaluation**: Using `ContextualLink` to defer data fetching until needed
- **Complex data flow**: Using `Map` to pass multiple related values

## External documentation

From `deep_causality_core/src/types/effect_value/mod.rs`:
```rust
/// Represents the payload of a propagating effect.
///
/// This enum encapsulates various types of effect data that can be propagated
/// through the causal effect system. It is generic over type `T` to allow
/// flexibility in the value type.
#[derive(Debug, Clone, Default)]
pub enum EffectValue<T> {
    /// Represents the absence of a signal or evidence.
    #[default]
    None,
    /// Represents a value of type T
    Value(T),
    /// A link to a complex, structured result in a Contextoid. As an output, this
    /// can be interpreted by a reasoning engine as a command to fetch data.
    ContextualLink(ContextoidId, ContextoidId),
    /// A dispatch command that directs the reasoning engine to dynamically jump to a specific
    /// causaloid within the graph. The `usize` is the target causaloid's index, and the `Box<PropagatingEffect>`
    /// is the effect to be passed as input to that target causaloid. This enables adaptive reasoning.
    RelayTo(usize, Box<PropagatingEffect<T>>),
    /// A collection of named values, allowing for complex, structured data passing.
    #[cfg(feature = "std")]
    Map(HashMap<IdentificationValue, Box<PropagatingEffect<T>>>),
}
```

From `deep_causality_core/src/types/effect_value/predicates.rs`:
```rust
pub fn into_value(self) -> Option<T> {
    match self {
        EffectValue::Value(v) => Some(v),
        _ => None,
    }
}
```

# Why has this bug gone undetected?

The bug has gone undetected because:

1. **Limited test coverage**: The existing tests for context-aware causaloids only test functions that return `EffectValue::Value` variants (see `test_evaluate_singleton_with_context` in `causaloid_singleton_tests.rs`).

2. **Recent feature introduction**: The bug was introduced in commit 89a73199 during a major refactoring that reduced type parameters from 9 to 4. The advanced `EffectValue` variants (`ContextualLink`, `RelayTo`, `Map`) are newer features that may not have been heavily used yet.

3. **Subtle error message**: When the bug occurs, it produces an error message that sounds reasonable ("context_fn returned None value"), making it seem like a user error rather than a framework bug.

4. **Most common use case works**: The most common pattern is for causal functions to return `EffectValue::Value`, which works correctly. Only advanced use cases that leverage the dynamic reasoning capabilities (`ContextualLink` and `RelayTo`) are affected.

5. **Pattern inconsistency hidden**: The correct pattern is used in `causable.rs` for handling output values, but a different (incorrect) pattern is used in `causable_utils.rs` for converting process results. This inconsistency might not be obvious without comparing both files.

# Recommended fix

Replace the `into_value()` pattern with proper handling of all `EffectValue` variants:

```rust
// Current (buggy) code:
let mut effect = match process.value.into_value() {
    Some(val) => PropagatingEffect::pure(val),
    None => {
        let error = process.error.unwrap_or_else(|| {
            CausalityError(deep_causality_core::CausalityErrorEnum::Custom(
                "execute_causal_logic: context_fn returned None value and no error"
                    .into(),
            ))
        });
        PropagatingEffect::from_error(error)
    }
};
effect.logs = process.logs;
```

Should be replaced with:

```rust
// Fixed code:
let mut effect = if process.value.is_none() && process.error.is_none() {  // <-- FIX 🟢
    // Only treat EffectValue::None without error as an error condition
    PropagatingEffect::from_error(CausalityError(
        deep_causality_core::CausalityErrorEnum::Custom(
            "execute_causal_logic: context_fn returned None value and no error".into(),
        )
    ))
} else if let Some(error) = process.error {
    // Preserve any error from the process
    PropagatingEffect::from_error(error)
} else {
    // Preserve the effect value (whether Value, ContextualLink, RelayTo, or Map)
    PropagatingEffect::from_effect_value(process.value)  // <-- FIX 🟢
};
effect.logs = process.logs;
```

This fix:
1. Preserves all valid `EffectValue` variants (not just `Value`)
2. Correctly treats only `EffectValue::None` without an error as an error condition
3. Preserves error states from the process
4. Maintains log provenance as intended
5. Matches the pattern used successfully in `causable.rs`


Causaloid::evaluate: None output not converted to error (invalid PropagatingEffect state)


# Summary
- **Context**: The `Causaloid::evaluate` method for singleton causaloids chains three monadic operations: logging input, executing causal logic, and logging output.
- **Bug**: When a causal function returns `EffectValue::None` without an error, the final logging step fails to convert this to an error, resulting in a `PropagatingEffect` with `value: None` and `error: None`.
- **Actual vs. expected**: The code allows `None` values without errors to pass through, while it should treat `None` as an error (consistent with the framework's error handling semantics).
- **Impact**: Silent failures can occur when causal functions return `None` without explicitly setting an error, leading to invalid states where computations produce no result but also report no error.

# Code with bug

```rust
// From: deep_causality/src/types/causal_types/causaloid/causable.rs (lines 92-95)

.bind(|output_effect_val, _, _| match output_effect_val {
    EffectValue::Value(v) => causable_utils::log_output(v, self.id),
    _ => PropagatingEffect::from_effect_value(output_effect_val), // <-- BUG 🔴 When output_effect_val is None, this creates a PropagatingEffect with no error
})
```

The `_` catch-all branch handles `EffectValue::None` and `EffectValue::Error` identically by calling `from_effect_value`, which simply wraps the value without adding an error. This is incorrect for the `None` case.

# Evidence

## Failing test

### Test script

```rust
// Test to demonstrate bug: causal function returning None should result in error
// This test will FAIL, demonstrating the bug

use deep_causality::*;
use deep_causality_core::EffectValue;
use deep_causality_haft::LogAddEntry;

#[test]
#[should_panic(expected = "Expected an error when causal function returns None")]
fn test_causal_fn_returns_none_should_error() {
    let id: IdentificationValue = 1;
    let description = "test causaloid that returns None";

    // This causal function returns EffectValue::None without an error
    fn causal_fn_returns_none(_obs: NumericalValue) -> PropagatingEffect<bool> {
        let mut log = EffectLog::new();
        log.add_entry("Causal function executed and returned None");

        // Return None value with logs (no error set)
        let mut effect = PropagatingEffect::from_effect_value(EffectValue::None);
        effect.logs = log;
        effect
    }

    let causaloid = BaseCausaloid::<NumericalValue, bool>::new(
        id,
        causal_fn_returns_none,
        description
    );

    // Create valid input
    let mut input_effect = PropagatingEffect::from_value(0.5);
    input_effect.logs.add_entry("Initial input log");

    // Evaluate the causaloid
    let result = causaloid.evaluate(&input_effect);

    println!("Result value: {:?}", result.value);
    println!("Result error: {:?}", result.error);
    println!("Result logs:\n{}", result.logs);

    // BUG: When causal_fn returns EffectValue::None, the result should have an error
    // because None indicates a failed computation (no meaningful result produced).
    //
    // This is consistent with:
    // 1. bind_or_error converting None to error in earlier stages
    // 2. context_causal_fn behavior (execute_causal_logic converts None to error)
    // 3. The semantic meaning of None (no result = failure)

    if result.error.is_none() {
        panic!("Expected an error when causal function returns None, but got none. Value: {:?}", result.value);
    }
}
```

### Test output

```
running 1 test
Result value: None
Result error: None
Result logs:
EffectLog (3 entries):
[[1765874758444639] Initial input log
[[1765874758444646] Causaloid 1: Incoming effect: Value(0.5)
[[1765874758444650] Causal function executed and returned None

thread 'test_causal_fn_returns_none_should_error' panicked at deep_causality/tests/test_none_output_bug.rs:51:9:
Expected an error when causal function returns None, but got none. Value: None
```

The test demonstrates that when a causal function returns `EffectValue::None`, the evaluation produces a result with `value: None` and `error: None`, which violates the expected behavior.

## Inconsistency within the codebase

### Reference code

`deep_causality/src/types/causal_types/causaloid/causable_utils.rs:39-50`

```rust
let mut effect = match process.value.into_value() {
    Some(val) => PropagatingEffect::pure(val),
    None => {
        let error = process.error.unwrap_or_else(|| {
            CausalityError(deep_causality_core::CausalityErrorEnum::Custom(
                "execute_causal_logic: context_fn returned None value and no error"
                    .into(),
            ))
        });
        PropagatingEffect::from_error(error)
    }
};
```

### Current code

`deep_causality/src/types/causal_types/causaloid/causable.rs:92-95`

```rust
.bind(|output_effect_val, _, _| match output_effect_val {
    EffectValue::Value(v) => causable_utils::log_output(v, self.id),
    _ => PropagatingEffect::from_effect_value(output_effect_val),
})
```

### Contradiction

When `context_causal_fn` is used, `execute_causal_logic` explicitly converts `None` values to errors (reference code). However, when a regular `causal_fn` returns `None`, the final bind step does NOT convert it to an error. This creates inconsistent behavior depending on which type of causal function is used.

## Inconsistency within the codebase

### Reference code

`deep_causality/src/types/causal_types/causaloid/causable.rs:84-87`

```rust
.bind_or_error(
    |input, _, _| causable_utils::log_input(input, self.id),
    "Cannot evaluate: input value is None",
)
```

### Current code

`deep_causality/src/types/causal_types/causaloid/causable.rs:92-95`

```rust
.bind(|output_effect_val, _, _| match output_effect_val {
    EffectValue::Value(v) => causable_utils::log_output(v, self.id),
    _ => PropagatingEffect::from_effect_value(output_effect_val),
})
```

### Contradiction

The first two steps in the monadic chain use `bind_or_error`, which explicitly converts `None` values to errors. The third step uses regular `bind` with a match that does NOT convert `None` to an error. This creates inconsistent error handling within the same evaluation chain.

## Inconsistency with own spec

### Reference spec

From the docstring of `Causaloid::evaluate` in `deep_causality/src/types/causal_types/causaloid/causable.rs:61`:

```rust
/// The evaluation process is monadic, ensuring that errors are propagated
```

From line 81:

```rust
// The `bind` operations ensure that logs are aggregated and errors short-circuit.
```

### Current code

```rust
.bind(|output_effect_val, _, _| match output_effect_val {
    EffectValue::Value(v) => causable_utils::log_output(v, self.id),
    _ => PropagatingEffect::from_effect_value(output_effect_val),
})
```

### Contradiction

The documentation promises that "errors are propagated" and that the bind operations "ensure errors short-circuit". However, when `output_effect_val` is `None` (which semantically represents a failed computation), the code does not treat it as an error. Instead, it returns a `PropagatingEffect` with `value: None` and `error: None`, which is an invalid state that violates the monadic error propagation guarantee.

## Example

Consider this execution flow:

**Step 1:** Input logging with `bind_or_error`
- Input: `PropagatingEffect { value: Value(0.5), error: None, logs: [...] }`
- Output: `PropagatingEffect { value: Value(0.5), error: None, logs: [..."Incoming effect"...] }`
- If input were `None`, it would convert to an error ✓

**Step 2:** Execute causal logic with `bind_or_error`
- Input: `PropagatingEffect { value: Value(0.5), error: None, logs: [...] }`
- The causal function returns: `PropagatingEffect { value: None, error: None, logs: [..."returned None"] }`
- Output: `PropagatingEffect { value: None, error: None, logs: [..."returned None"] }`
- Note: `bind_or_error` doesn't trigger because the effect itself has no error, only the value is None

**Step 3:** Output logging with `bind` (BUG HERE)
- Input: `EffectValue::None` (extracted from the effect)
- The match catches this in the `_` branch
- Output: `PropagatingEffect::from_effect_value(EffectValue::None)`
- Result: `PropagatingEffect { value: None, error: None, logs: [...] }` ❌

**Expected behavior at Step 3:**
- When `output_effect_val` is `None`, it should be converted to an error, just like Steps 1 and 2 would do
- Result should be: `PropagatingEffect { value: None, error: Some(...), logs: [...] }` ✓

# Full context

The `Causaloid` struct is the fundamental building block for causal reasoning in DeepCausality. It encapsulates causal logic and supports three types:
1. `Singleton` - a single causal function
2. `Collection` - a collection of causaloids
3. `Graph` - a graph of interconnected causaloids

The `evaluate` method is the entry point for causal reasoning. For singleton causaloids, it performs a three-step monadic chain:
1. Log the input effect
2. Execute the causal function
3. Log the output effect

Each step is designed to preserve logs and propagate errors. The first two steps use `bind_or_error`, which automatically converts `None` values to errors. However, the third step uses regular `bind` with a match statement that fails to handle `None` correctly.

The bug affects all singleton causaloids that use regular `causal_fn` (as opposed to `context_causal_fn`). When such a causal function returns `EffectValue::None` without explicitly setting an error, the evaluation produces an invalid result where both the value and error are `None`.

This can occur in scenarios where:
- A causal function encounters an edge case it doesn't explicitly handle
- A computation produces no meaningful result (e.g., division by zero that's caught but not converted to an error)
- A function uses optional logic that may fail to produce a value

The evaluation chain is called by:
- Direct users calling `causaloid.evaluate(&effect)`
- Collection evaluation logic that chains multiple causaloids
- Graph evaluation logic that processes interconnected causaloids
- Test suites validating causal reasoning

# Why has this bug gone undetected?

This bug has gone undetected for several reasons:

1. **Rare edge case**: In practice, most causal functions either return a valid value with `PropagatingEffect::from_value(...)` or explicitly return an error with `PropagatingEffect::from_error(...)`. Functions that return `None` without an error are uncommon.

2. **Recent introduction**: The bug was introduced in commit `15f021cb` ("Improved test coverage") when the evaluation chain was refactored from a single `bind` call to a three-step chain. The previous code had proper `None` handling:
   ```rust
   .bind(|effect_value, _, _| match effect_value.into_value() {
       Some(input) => causable_utils::execute_causal_logic(input, self),
       None => PropagatingEffect::from_error(CausalityError(...)),
   })
   ```

3. **Type system doesn't catch it**: Rust's type system allows `PropagatingEffect` to have both `value: None` and `error: None`, even though this is semantically invalid. There's no type-level enforcement that `None` values must have associated errors.

4. **Tests focus on happy path**: Most existing tests verify successful evaluations or explicit error cases. They don't test the edge case of causal functions returning `None` without errors.

5. **Monadic abstraction hides the issue**: The `bind` operation properly merges logs, which creates the impression that everything works correctly. The subtle issue is that `None` should trigger error handling, not just log preservation.

6. **Context functions mask the problem**: When using `context_causal_fn`, the `execute_causal_logic` function explicitly converts `None` to errors, so the bug doesn't manifest in that code path.

# Recommended fix

The final `bind` operation should use `bind_or_error` instead of `bind`, or the match statement should explicitly convert `None` to an error:

```rust
.bind(|output_effect_val, _, _| match output_effect_val {
    EffectValue::Value(v) => causable_utils::log_output(v, self.id),
    EffectValue::None => PropagatingEffect::from_error(CausalityError(  // <-- FIX 🟢
        deep_causality_core::CausalityErrorEnum::Custom(
            "Cannot log output: causal function returned None".into(),
        )
    )),
    EffectValue::Error => PropagatingEffect::from_effect_value(output_effect_val),
})
```

Alternatively, restructure to use `bind_or_error` for consistency:

```rust
.bind_or_error(
    |output, _, _| causable_utils::log_output(output, self.id),  // <-- FIX 🟢
    "Cannot log output: causal function returned None",
)
```

This would make the error handling consistent across all three steps of the evaluation chain.
