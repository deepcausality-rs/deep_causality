# Summary
- **Context**: The `DeonticError` enum wraps errors from the `ultragraph` crate's `GraphError` type to provide domain-specific error handling for deontic inference operations in the Effect Ethos system.
- **Bug**: When certain `GraphError` variants are converted to `DeonticError` via the `From` trait, the error source chain is lost because they're mapped to specialized variants instead of being wrapped.
- **Actual vs. expected**: The `Error::source()` method returns `None` for converted errors like `FailedToAddEdge`, `GraphIsFrozen`, etc., when it should return `Some(&GraphError)` to preserve the error chain.
- **Impact**: Debugging becomes significantly harder because developers lose access to the original low-level error details when errors are propagated up the call stack.

# Code with bug

The issue is in the `From<GraphError>` implementation in `deep_causality_ethos/src/errors/deontic_error.rs`:

```rust
impl From<GraphError> for DeonticError {
    fn from(err: GraphError) -> Self {
        match err {
            GraphError::GraphIsFrozen => DeonticError::GraphIsFrozen,  // <-- BUG 🔴 Original error lost
            GraphError::GraphNotFrozen => DeonticError::GraphNotFrozen,  // <-- BUG 🔴 Original error lost
            GraphError::GraphContainsCycle => DeonticError::GraphIsCyclic,  // <-- BUG 🔴 Original error lost
            GraphError::EdgeCreationError { source, target } => {
                DeonticError::FailedToAddEdge(source, target)  // <-- BUG 🔴 Original error lost
            }
            _ => DeonticError::GraphError(err),
        }
    }
}
```

And the corresponding `Error::source()` implementation:

```rust
impl Error for DeonticError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            DeonticError::GraphError(e) => Some(e),  // Only wrapped errors preserve source
            _ => None,  // <-- BUG 🔴 All converted variants return None
        }
    }
}
```

# Evidence

## Example

Consider this error propagation scenario:

1. **Low-level operation fails**: `ultragraph` tries to add an edge but fails because the graph is frozen, creating `GraphError::EdgeCreationError { source: 1, target: 2 }`
2. **Error is converted**: The error is converted using `.map_err(DeonticError::from)` in `teloidable.rs:69`
3. **Original error is lost**: The conversion produces `DeonticError::FailedToAddEdge(1, 2)`, discarding the original `GraphError`
4. **Debugging is impaired**: When a developer calls `.source()` on the error, they get `None` instead of the original `GraphError`, losing valuable debugging information

## Inconsistency within the codebase

### Reference code
`deep_causality_ethos/src/errors/deontic_error.rs:126-138`
```rust
impl From<GraphError> for DeonticError {
    fn from(err: GraphError) -> Self {
        match err {
            GraphError::GraphIsFrozen => DeonticError::GraphIsFrozen,
            GraphError::GraphNotFrozen => DeonticError::GraphNotFrozen,
            GraphError::GraphContainsCycle => DeonticError::GraphIsCyclic,
            GraphError::EdgeCreationError { source, target } => {
                DeonticError::FailedToAddEdge(source, target)
            }
            _ => DeonticError::GraphError(err),  // <-- Other variants preserve the error
        }
    }
}
```

### Contradiction

The `From` implementation is inconsistent: some `GraphError` variants (like `NodeNotFound`) are wrapped in `DeonticError::GraphError(err)` which preserves the error chain, while others are converted to specialized variants that discard the original error. This inconsistency violates the Rust error handling convention that error chains should be preserved for debugging purposes.

## Failing test

### Test script
```rust
/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_ethos::DeonticError;
use std::error::Error;
use ultragraph::GraphError;

/// This test demonstrates a bug: error source chain is lost when
/// GraphError::EdgeCreationError is converted to DeonticError::FailedToAddEdge
#[test]
fn test_error_source_chain_preserved_for_edge_creation_error() {
    // Create a GraphError::EdgeCreationError
    let graph_error = GraphError::EdgeCreationError {
        source: 1,
        target: 2,
    };

    // Convert to DeonticError via From trait
    let deontic_error: DeonticError = graph_error.into();

    // The conversion produces DeonticError::FailedToAddEdge(1, 2)
    assert_eq!(deontic_error, DeonticError::FailedToAddEdge(1, 2));

    // BUG: source() returns None because FailedToAddEdge doesn't preserve the original error
    // Expected: source() should return Some(&GraphError::EdgeCreationError{...})
    // Actual: source() returns None
    let source = deontic_error.source();

    // This assertion should PASS if the bug is fixed
    assert!(
        source.is_some(),
        "BUG: Error source chain is lost! DeonticError::FailedToAddEdge should preserve the original GraphError"
    );
}
```

### Test output
```
running 1 test
test errors::deontic_error_tests::test_error_source_chain_preserved_for_edge_creation_error ... FAILED

failures:

---- errors::deontic_error_tests::test_error_source_chain_preserved_for_edge_creation_error stdout ----

thread 'errors::deontic_error_tests::test_error_source_chain_preserved_for_edge_creation_error' panicked at deep_causality_ethos/tests/errors/deontic_error_tests.rs:193:5:
BUG: Error source chain is lost! DeonticError::FailedToAddEdge should preserve the original GraphError
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    errors::deontic_error_tests::test_error_source_chain_preserved_for_edge_creation_error

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 105 filtered out; finished in 0.00s
```

# Full context

The `DeonticError` type is used throughout the `deep_causality_ethos` crate to handle errors during deontic inference operations. The Effect Ethos system performs ethical reasoning by evaluating proposed actions against norms (represented as Teloids) stored in a graph structure managed by the `ultragraph` crate.

When graph operations fail (e.g., adding nodes, edges, or querying the graph), the `ultragraph` crate returns a `GraphError`. These errors are converted to `DeonticError` at API boundaries using `.map_err(DeonticError::from)` in several locations:

1. **`teloidable.rs:21`**: When adding a teloid (norm) to the graph
2. **`teloidable.rs:69`**: When adding inheritance edges between norms
3. **`teloidable.rs:90`**: When adding defeasance edges between norms
4. **`graph.rs:26`**: When clearing the graph

In all these cases, if the underlying `GraphError` is one of the specially-mapped variants (`EdgeCreationError`, `GraphIsFrozen`, etc.), the error source chain is lost. This makes debugging difficult because:

- Developers can't use standard Rust error introspection tools like `error.source()` to trace the root cause
- Stack traces and error reports don't show the complete error chain
- Integration with error handling libraries (like `anyhow` or `thiserror`) that rely on error chains is impaired

The Effect Ethos system is used in production scenarios for ethical decision-making (as shown in the CSM example at `examples/csm_examples/csm_effect_ethos/main.rs:92-94`), where errors are displayed to users using debug formatting (`{:?}`). While the current implementation provides human-readable error messages, it violates Rust's error handling best practices.

# Why has this bug gone undetected?

This bug has gone undetected for several reasons:

1. **Error messages are still informative**: The `Display` implementation for both `GraphError` and `DeonticError` provide clear, human-readable error messages. For example, both `GraphError::EdgeCreationError{source: 1, target: 2}` and `DeonticError::FailedToAddEdge(1, 2)` produce similar messages: "Edge from 1 to 2 could not be created; a node may not exist or the edge already exists."

2. **Existing tests explicitly check for this behavior**: The test suite at `tests/errors/deontic_error_tests.rs` contains tests that explicitly verify `error.source().is_none()` for all the converted variants (lines 19, 31, 43, 55, 67, 79, 91, 103, 115). This suggests the current behavior was intentional, though it violates Rust error handling conventions.

3. **Simple error handling in examples**: The example code (e.g., `examples/csm_examples/csm_effect_ethos/main.rs:92-94`) uses simple error handling that only displays the top-level error without introspecting the error chain:
   ```rust
   Err(e) => {
       println!("Ethos evaluation error: {:?}", e);
   }
   ```

4. **No use of error chain introspection**: The codebase doesn't appear to use error introspection tools or libraries that rely on the `Error::source()` chain (like `anyhow::Error` or specialized error reporters).

5. **Production code primarily fails fast**: Most error paths in the deontic inference system fail immediately and return the error to the caller, rather than attempting error recovery or detailed error analysis where the source chain would be valuable.

# Recommended fix

The recommended fix is to change the `DeonticError` enum to preserve the original `GraphError` for all converted variants. This can be done in one of two ways:

## Option 1: Always wrap GraphError (simpler)

Remove the specialized variants that duplicate `GraphError` information and always use `DeonticError::GraphError`:

```rust
impl From<GraphError> for DeonticError {
    fn from(err: GraphError) -> Self {
        DeonticError::GraphError(err)  // <-- FIX 🟢 Always preserve the original error
    }
}
```

However, this would lose the specialized error messages and type information.

## Option 2: Store the original error alongside specialized information (better)

Modify the affected `DeonticError` variants to include the original `GraphError`:

```rust
pub enum DeonticError {
    // ... other variants ...

    /// Failed to add edge to the graph.
    FailedToAddEdge(usize, usize, GraphError),  // <-- FIX 🟢 Add GraphError field

    /// The graph is frozen and cannot be modified.
    GraphIsFrozen(GraphError),  // <-- FIX 🟢 Add GraphError field

    /// The graph is not frozen but should be.
    GraphNotFrozen(GraphError),  // <-- FIX 🟢 Add GraphError field

    /// The graph contains a cycle.
    GraphIsCyclic(GraphError),  // <-- FIX 🟢 Add GraphError field

    // ... other variants ...
}

impl Error for DeonticError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            DeonticError::GraphError(e) => Some(e),
            DeonticError::FailedToAddEdge(_, _, e) => Some(e),  // <-- FIX 🟢
            DeonticError::GraphIsFrozen(e) => Some(e),  // <-- FIX 🟢
            DeonticError::GraphNotFrozen(e) => Some(e),  // <-- FIX 🟢
            DeonticError::GraphIsCyclic(e) => Some(e),  // <-- FIX 🟢
            _ => None,
        }
    }
}
```

This approach preserves both the specialized error information (for better error messages and type matching) and the original error chain (for debugging).

# Related bugs

The same pattern might exist in other error conversions throughout the codebase. A search for similar `From` implementations that convert between error types should be conducted to ensure error chains are preserved consistently across the entire project.

# Summary
- **Context**: The `verify_graph()` method in `EffectEthos` checks the teloid graph for cycles before the graph can be used for deontic inference (ethical reasoning).
- **Bug**: When `verify_graph()` detects a cycle, it freezes the graph but sets `is_verified` to `false`, leaving the system in an inconsistent state.
- **Actual vs. expected**: After cycle detection, the graph is frozen (preventing modifications) but not verified (preventing evaluation), whereas it should either remain unfrozen (allowing cycle fixes) or be properly verified.
- **Impact**: Users cannot fix detected cycles without discovering and manually calling `unfreeze()`, making error recovery non-obvious and potentially blocking legitimate workflows.

# Code with bug
```rust
pub fn verify_graph(&mut self) -> Result<(), DeonticError> {
    self.teloid_graph.graph.freeze();  // <-- Always freezes the graph
    if self.teloid_graph.graph.has_cycle()? {
        self.is_verified = false;  // <-- BUG 🔴: Graph remains frozen but is marked as not verified
        Err(DeonticError::GraphIsCyclic)
    } else {
        self.is_verified = true;
        Ok(())
    }
}
```
From `deep_causality_ethos/src/types/effect_ethos/verify.rs:26-35`

# Evidence

## Failing test

### Test script
```rust
/*
 * Test to reproduce the verify_graph bug
 * When verify_graph detects a cycle, the graph remains frozen but is_verified is false,
 * creating an inconsistent state where the user cannot modify the graph to fix the cycle.
 */

use deep_causality_ethos::utils_test::test_utils_effect_ethos;
use deep_causality_ethos::{DeonticError, TeloidModal};

#[test]
fn test_verify_graph_leaves_frozen_on_cycle() {
    // Create an ethos with a cycle
    let mut ethos = test_utils_effect_ethos::TestEthos::new()
        .add_deterministic_norm(
            1,
            "a",
            &[],
            test_utils_effect_ethos::always_true_predicate,
            TeloidModal::Obligatory,
            1,
            1,
            1,
        )
        .unwrap()
        .add_deterministic_norm(
            2,
            "b",
            &[],
            test_utils_effect_ethos::always_true_predicate,
            TeloidModal::Obligatory,
            1,
            1,
            1,
        )
        .unwrap()
        .link_inheritance(1, 2)
        .unwrap()
        .link_inheritance(2, 1) // Creates a cycle
        .unwrap();

    // Verify initial state
    assert!(!ethos.is_frozen(), "Graph should not be frozen initially");
    assert!(!ethos.is_verified(), "Graph should not be verified initially");

    // Try to verify the graph - this should detect the cycle
    let result = ethos.verify_graph();

    // The verification should fail with GraphIsCyclic error
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), DeonticError::GraphIsCyclic));

    // BUG: After a failed verification, the graph is frozen but not verified!
    // This is an inconsistent state:
    println!("After failed verify_graph():");
    println!("  is_frozen: {}", ethos.is_frozen());
    println!("  is_verified: {}", ethos.is_verified());

    // The graph is now frozen (this is the bug!)
    assert!(ethos.is_frozen(), "BUG: Graph is frozen after failed verification");

    // And it's correctly not verified
    assert!(!ethos.is_verified(), "Graph should not be verified when cycle detected");

    // The only way to recover is to manually unfreeze
    ethos.unfreeze();
    assert!(!ethos.is_frozen(), "Graph should be unfrozen after calling unfreeze()");
    assert!(!ethos.is_verified(), "Graph should still not be verified after unfreeze()");
}
```

### Test output
```
running 1 test
After failed verify_graph():
  is_frozen: true
  is_verified: false
test test_verify_graph_leaves_frozen_on_cycle ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Example

Consider a developer building a deontic reasoning system who accidentally creates a cycle:

1. **Setup**: Developer creates two norms with a cyclic inheritance relationship:
    - Norm 1: "Driving is obligatory"
    - Norm 2: "Safe driving is obligatory"
    - Links: 1 → 2 and 2 → 1 (cycle!)

2. **Verification attempt**: Developer calls `verify_graph()`:
   ```rust
   let result = ethos.verify_graph();
   // Returns Err(DeonticError::GraphIsCyclic)
   ```

3. **Bug manifestation**: The graph state is now:
    - `is_frozen()` returns `true`
    - `is_verified()` returns `false`

4. **Attempted fix**: Developer tries to fix the cycle by removing a link:
   ```rust
   // This won't work because the builder methods check is_frozen()
   ethos.link_inheritance(1, 3) // Would fail with GraphIsFrozen
   ```

5. **Recovery requirement**: Developer must discover and call `unfreeze()`:
   ```rust
   ethos.unfreeze(); // Not obvious from the API
   // Now can modify the graph to fix the cycle
   ```

# Full context

The `EffectEthos` struct is a deontic reasoning engine that evaluates proposed actions against teleological norms (called Teloids). It maintains an internal graph (`teloid_graph`) that represents relationships between norms through inheritance and defeasance edges.

The system has a two-phase lifecycle:
1. **Construction phase**: Norms are added and linked together. The graph must be unfrozen.
2. **Evaluation phase**: Actions are evaluated against norms. The graph must be frozen and verified.

The `verify_graph()` method is the transition point between these phases. It:
1. Freezes the graph (optimizes it for read-only operations)
2. Checks for cycles (which would cause infinite loops during evaluation)
3. Sets the `is_verified` flag based on the result

The freeze/unfreeze mechanism prevents modifications during evaluation. Methods like `link_inheritance()` and `link_defeasance()` check `is_frozen()` and return `DeonticError::GraphIsFrozen` if the graph is frozen (see `deep_causality_ethos/src/types/effect_ethos/api.rs:170` and `api.rs:205`).

The `evaluate_action()` method requires both conditions:
- Graph must be frozen (checked at `deontic_inference.rs:57`)
- Graph must be verified (checked at `deontic_inference.rs:62`)

When `verify_graph()` detects a cycle, it satisfies the first condition (frozen) but not the second (verified), making the graph unusable for both modification and evaluation.

# Why has this bug gone undetected?

This bug has gone undetected for several reasons:

1. **Test coverage gap**: The existing test `test_verify_graph_fails_on_cycle` (in `effect_ethos_verify_graph_tests.rs:35-71`) only checks that the error is returned and `is_verified` is false. It doesn't check the frozen state or attempt recovery operations.

2. **Happy path focus**: Most tests follow the successful verification path where `verify_graph()` succeeds, so the inconsistent error state is rarely encountered.

3. **Limited practical impact**: In many workflows, users might simply create a new `EffectEthos` instance rather than trying to fix and reuse one with a cycle. The builder pattern with ownership-taking methods (like `add_deterministic_norm` and `link_inheritance`) encourages rebuilding rather than modifying.

4. **Non-obvious symptom**: The bug manifests as a "frozen but not verified" state, which isn't immediately visible unless the user checks both `is_frozen()` and `is_verified()` after a failed verification. The error message doesn't hint at the frozen state.

5. **Code review blind spot**: The freeze-before-check pattern appears intentional (since freezing is needed for the cycle check), and reviewers might not have considered the error path implications.

# Recommended fix

The fix should ensure that on verification failure, the graph is unfrozen so users can modify it to fix the cycle:

```rust
pub fn verify_graph(&mut self) -> Result<(), DeonticError> {
    self.teloid_graph.graph.freeze();
    if self.teloid_graph.graph.has_cycle()? {
        self.is_verified = false;
        self.teloid_graph.graph.unfreeze(); // <-- FIX 🟢: Unfreeze on failure
        Err(DeonticError::GraphIsCyclic)
    } else {
        self.is_verified = true;
        Ok(())
    }
}
```

This ensures the graph state is consistent:
- **On success**: Graph is frozen and verified (ready for evaluation)
- **On failure**: Graph is unfrozen and not verified (ready for modification)

Alternative considerations:
1. Document the current behavior and make `unfreeze()` part of the error recovery pattern
2. Make `verify_graph()` idempotent by checking if already frozen and only freezing if needed
3. Add a `reset()` or `retry_verification()` method that explicitly handles the recovery

The first fix (unfreeze on failure) is the most intuitive and maintains the principle of least surprise for API users.
