# Pre-Spec: Refactoring Causaloid to a Monad Transformer

**Version**: 0.5
**Date**: 2025-11-05
**Status**: Draft

## 1. Overview

This document outlines a proposal to refactor the core `Causaloid` and `PropagatingEffect` types to leverage a type-encoded monadic effect system, using the patterns established in the `deep_causality_haft` crate.

The goal is to transform the `Causaloid` from a stateful object into a pure, reusable "transformer" function. The `PropagatingEffect` will evolve from a simple enum into a monadic container that carries the full state of a causal computation (value, errors, and logs).

## 2. Core Idea: Shifting Roles

The fundamental shift is to separate the *description* of a computation from its *state*.

- **`Causaloid` (The Transformer)**: Becomes a pure, immutable description of a causal transformation. It no longer stores its own `effect` or result. It is analogous to a function `A -> M<B>`, where `M` is the Monad.
- **`PropagatingEffect` (The Monad)**: Becomes the stateful container that is passed through the computation. It accumulates logs and errors as it is transformed by `Causaloid`s. The `explain` method, which details the history of the computation, will be moved to this type.

This change guarantees purity, enables full auditable traceability, and improves composability and testability.

## 3. The New `PropagatingEffect` (The Monad)

The current `PropagatingEffect` enum will be replaced by a generic struct that serves as the concrete data type for our effect system.

```rust
// Conceptual new PropagatingEffect struct
#[derive(Debug, Clone)]
pub struct PropagatingEffect<T, E, L> {
    pub value: T,          // The primary causal value
    pub error: Option<E>,  // Error state (e.g., String or a custom error type)
    pub logs: Vec<L>,      // Accumulated log entries
}

impl<T, E, L> PropagatingEffect<T, E, L> {
    // Constructor for a new effect
    pub fn new(value: T, error: Option<E>, logs: Vec<L>) -> Self {
        Self { value, error, logs }
    }

    // The new home for the explain method
    pub fn explain(&self) -> String {
        // Iterates over self.logs to build a comprehensive
        // history of the computation.
        // ... implementation ...
    }
}
```

## 4. HKT & Effect System Setup

We will use the `deep_causality_haft` crate to build the effect system around the new `PropagatingEffect`.

1.  **HKT Witness (`PropagatingEffectHktWitness`)**: A new unit struct that will implement `HKT3<E, L>` to represent the `PropagatingEffect<T, E, L>` type constructor, where `T` is the variable type.

2.  **System Witness (`CausalEffectSystem`)**: A new unit struct that will implement `Effect3`. This is where we will fix the concrete types for the effects:
    - `type Fixed1 = CausalityError;` // Error type
    - `type Fixed2 = String;`         // Log type

3.  **Monad Implementation (`CausalMonad`)**: A new unit struct that will implement `MonadEffect3<CausalEffectSystem>`. This will provide:
    - `pure(value)`: Lifts a `CausalValue` into a clean `PropagatingEffect` with no initial error or logs.
    - `bind(effect, f)`: The core sequencing logic. It will handle error propagation and the automatic concatenation of `logs` from one step to the next.

## 5. The New `Causaloid` (The Pure Transformer)

The `Causaloid` struct will be purified by removing its internal state.

```rust
// In the Causaloid struct definition:
use std::marker::PhantomData;

#[allow(clippy::type_complexity)]
pub struct Causaloid<D, S, T, ST, SYM, VS, VT>
where
    // ... existing trait bounds ...
{
    // --- Fields to KEEP ---
    id: u64,
    description: String,
    causal_type: CausaloidType,
    causal_fn: Option<CausalFn<D, S, T, ST, SYM, VS, VT>>,
    context_causal_fn: Option<ContextualCausalFn<D, S, T, ST, SYM, VS, VT>>,
    // ... other pure data fields ...

    // --- Field to REMOVE ---
    // effect: ArcRWLock<Option<PropagatingEffect>>,

    // --- Field to ADD ---
    // This marks the struct as being part of a monadic system,
    // helping to enforce type rules.
    _phantom: PhantomData<fn() -> PropagatingEffect<D, S, T, ST, SYM>>,
}
```

## 6. The New `MonadicCausable` Trait

The `Causable` trait will be replaced by a new `MonadicCausable` trait that reflects the monadic `bind` signature.

```rust
// New Trait Signature (Conceptual)
// P is the HKT Witness for our PropagatingEffect system.
trait MonadicCausable<P, CausalValue>
where
    P: MonadEffect3, // Using the arity-3 effect system
{
    /// The core monadic bind operation.
    /// Takes a Causaloid and a monadic context (the incoming effect)
    /// and returns the new monadic context (the outgoing effect).
    fn evaluate_monadic(&self, incoming_effect: P::Type<CausalValue>) -> P::Type<CausalValue>;
}
```

## 7. Implementation Strategy for `evaluate_monadic`

The implementation delegates the complex sequencing logic to the `bind` method of our `CausalMonad`.

```rust
// Inside `impl<P> MonadicCausable<P, CausalValue> for Causaloid<...>`
fn evaluate_monadic(&self, incoming_effect: P::Type<CausalValue>) -> P::Type<CausalValue> {
    // The logic of this function is now simply to call the Monad's bind operation.
    // The `inner_fn` closure contains the purified causal logic.
    CausalMonad::bind(incoming_effect, |causal_value_in| {
        // --- PASS 1: Execute the Purified Causal Function ---
        let causal_value_out = match self.causal_type {
            CausaloidType::Singleton => {
                // Logic to call the pure `causal_fn` or `context_causal_fn`.
                self.causal_fn.unwrap()(causal_value_in)
            }
            CausaloidType::Collection(coll) => {
                // DELEGATION: Use the new MonadicCausableCollection extension trait.
                // See Section 10 for details.
                let effect = PropagatingEffect::new(causal_value_in, None, Vec::new());
                return coll.fold_monadic(effect).value;
            }
            CausaloidType::Graph(graph) => {
                // DELEGATION: This is now handled by a monadic traversal.
                // See Section 10 for details.
                let effect = PropagatingEffect::new(causal_value_in, None, Vec::new());
                return graph.traverse_monadic(effect).value;
            }
        };

        // --- PASS 2: Generate Side Effects for this Step ---
        let (new_error, new_logs) = self.generate_context(&causal_value_in, &causal_value_out);

        // --- PASS 3: Return the new Monad State ---
        PropagatingEffect::new(
            causal_value_out,
            new_error,
            new_logs,
        )
    })
}
```

## 8. Conclusion

This refactoring aligns the `Causaloid` and `PropagatingEffect` with modern functional programming principles. It makes the system more robust, easier to reason about, and provides a powerful, auditable trace of every computation by design.

## 9. Detailed `CausalEffectSystem` and `CausalMonad` Implementation

This section provides a more detailed look at the boilerplate and logic required to set up the monadic system.

### 9.1. HKT Witness and System Witness

These are the foundational pieces that connect our concrete `PropagatingEffect` type to the generic traits in `deep_causality_haft`.

```rust
use deep_causality_haft::{HKT, HKT3, Effect3, Placeholder};
use crate::errors::CausalityError; // Assuming this is the error type

// The HKT Witness for PropagatingEffect
pub struct PropagatingEffectHktWitness<E, L>(Placeholder, E, L);

impl<E, L> HKT for PropagatingEffectHktWitness<E, L> {
    type Type<T> = PropagatingEffect<T, E, L>;
}

impl<E, L> HKT3<E, L> for PropagatingEffectHktWitness<E, L> {
    type Type<T> = PropagatingEffect<T, E, L>;
}

// The System Witness
pub struct CausalEffectSystem;

impl Effect3 for CausalEffectSystem {
    type Fixed1 = CausalityError;
    type Fixed2 = String;
    type HktWitness = PropagatingEffectHktWitness<Self::Fixed1, Self::Fixed2>;
}
```

### 9.2. The CausalMonad Implementation 

`CausalMonad` implements the core `pure` and `bind` logic. The `bind` implementation is critical for correct short-circuiting behavior.

```rust
use deep_causality_haft::{MonadEffect3, Functor};
use crate::types::CausalValue; // Assuming the null value is here

// The Monad Implementation
pub struct CausalMonad;

type CausalEffect<T> = <CausalEffectSystem as Effect3>::HktWitness::Type<T>;

impl MonadEffect3<CausalEffectSystem> for CausalMonad
where
    CausalEffectSystem::HktWitness: Functor<CausalEffectSystem::HktWitness> + Sized,
{
    fn pure<T>(value: T) -> CausalEffect<T> {
        PropagatingEffect::new(value, None, Vec::new())
    }

    fn bind<T, U, Func>(
        incoming_effect: CausalEffect<T>,
        mut f: Func,
    ) -> CausalEffect<U>
    where
        Func: FnMut(T) -> CausalEffect<U>,
        U: Default, // Require U to have a default value for error cases
    {
        // --- 1. Handle Error Propagation (Short-circuiting) ---
        if let Some(error) = incoming_effect.error {
            // **CORRECTED LOGIC**
            // Do NOT call f(). Instead, create a new effect with a null value.
            return PropagatingEffect {
                value: U::default(), // Use a cheap default/null value for the type.
                error: Some(error), // Propagate the original error
                logs: incoming_effect.logs, // Propagate the existing logs
            };
        }

        // --- 2. Apply the function to get the next effect ---
        let mut next_effect = f(incoming_effect.value);

        // --- 3. Combine the effects ---
        let mut combined_logs = incoming_effect.logs;
        combined_logs.append(&mut next_effect.logs);

        PropagatingEffect {
            value: next_effect.value,
            error: next_effect.error,
            logs: combined_logs,
        }
    }
}
```

## 10. Closing the Gaps: `Collection` and `Graph` Types

To make the `evaluate_monadic` strategy viable, we must define how it delegates work for recursive `Causaloid` types. We will follow the existing extension trait pattern.

### 10.1. New Trait: `MonadicCausableCollection`

We introduce a new extension trait for collections that can be evaluated monadically.

```rust
// The new monadic extension trait
trait MonadicCausableCollection<P, CausalValue>
where
    P: MonadEffect3,
    Self: CausableCollectionReasoning<MonadicCausable<P, CausalValue>> // The items in the collection must be MonadicCausable
{
    /// Performs a monadic fold over the collection of Causaloids.
    fn fold_monadic(&self, initial_effect: P::Type<CausalValue>) -> P::Type<CausalValue>;
}
```

### 10.2. Implementation for `[T]` and `Vec<T>`

We then implement this trait for standard collections.

```rust
// Implementation for any slice of MonadicCausable items
impl<T, P, CausalValue> MonadicCausableCollection<P, CausalValue> for [T]
where
    T: MonadicCausable<P, CausalValue>,
    P: MonadEffect3,
{
    fn fold_monadic(&self, initial_effect: P::Type<CausalValue>) -> P::Type<CausalValue> {
        // Use iterator fold to chain the bind operations
        self.iter().fold(initial_effect, |acc_effect, causaloid| {
            // The accumulator is the effect from the previous step.
            // The current item is the next causaloid to apply.
            causaloid.evaluate_monadic(acc_effect)
        })
    }
}

// Vec<T> gets the implementation automatically via Deref<Target=[T]>
```
This ensures that any `Vec<Causaloid>` (or other collection that derefs to a slice) can be evaluated sequentially, with effects being correctly propagated and accumulated.

### 10.3. `CausaloidType::Graph`: Monadic Traversal

A `Graph` Causaloid contains a graph of other `Causaloid`s. Its evaluation depends on the graph's structure.

1.  **Topological Sort**: First, perform a topological sort on the graph nodes to get a linearized sequence of `Causaloid`s that respects their causal dependencies.
2.  **Monadic Fold**: Once the linearized sequence is obtained, the evaluation becomes identical to the `Collection` case: call `fold_monadic` on the sorted sequence.

```rust
// Conceptual implementation for a Graph Causaloid
fn traverse_monadic(&self, incoming_effect: CausalEffect<CausalValue>) -> CausalEffect<CausalValue> {
    // 1. Get the execution order.
    let execution_order: Vec<Causaloid> = self.graph.topological_sort();

    // 2. Perform a monadic fold over the sorted sequence.
    execution_order.fold_monadic(incoming_effect)
}
```
This strategy ensures that the monadic evaluation correctly follows the causal flow defined by the graph, making the graph structure itself a guarantee of the logical flow of the monad.
