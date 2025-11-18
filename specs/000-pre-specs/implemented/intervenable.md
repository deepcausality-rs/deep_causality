# Pre-Specification: The `Intervenable` Trait for Causal Counterfactuals

- **Status**: Pre-Spec
- **Version**: 0.1.0
- **Authors**: Gemini
- **Date**: 2025-11-18

---

## 1. Objective

To introduce a new capability, `intervene(value)`, to the `CausalMonad` that allows for counterfactual reasoning. This operation will override the computed value within a monadic chain with a specified "intervention" value, effectively modeling Pearl's `do(X=x)` operator. This allows a user to observe how a forced change at one point in a causal sequence affects the final outcome.

The design must:
1.  Be a clean extension, not a modification of the core `Monad` trait.
2.  Integrate seamlessly with the existing `CausalMonad` and `PropagatingEffect` types.
3.  Preserve the principles of error propagation and log provenance, which are central to the current design.

## 2. Proposed Design

The design consists of two main parts: a new trait to define the `intervene` operation and its implementation for our specific `CausalMonad`.

### Part 1: The `Intervenable` Trait

A new, dedicated trait `Intervenable` will be created to define the intervention operation.

- **File Location**: `deep_causality/src/traits/intervenable/mod.rs`
- **Trait Definition**:
  ```rust
  use deep_causality_haft::{Effect3, HKT3};

  /// Defines the `intervene` operation for a monadic effect system.
  /// This trait is intended for causal reasoning systems where counterfactuals
  /// are modeled by forcing a value at a specific point in a computation chain.
  pub trait Intervenable<E: Effect3>
  where
      E::HktWitness: Sized,
  {
      /// Overrides the value within an effectful computation.
      ///
      /// This function takes an existing `effect` and a `new_value`. It returns a new
      /// effect where the original value is discarded and replaced by `new_value`.
      ///
      /// Crucially, it should preserve the context of the computation:
      /// - **Error State**: If the incoming `effect` was already in an error state,
      ///   that error is propagated. An intervention cannot fix a previously broken chain.
      /// - **Log History**: The logs from the incoming `effect` are preserved, and a
      ///   new entry is added to signify that an intervention occurred.
      fn intervene<T>(
          effect: <E::HktWitness as HKT3<E::Fixed1, E::Fixed2>>::Type<T>,
          new_value: T,
      ) -> <E::HktWitness as HKT3<E::Fixed1, E::Fixed2>>::Type<T>;
  }
  ```
- **Rationale**:
    - **Separation of Concerns**: This trait will reside in the `deep_causality` crate because intervention is a *causal concept*, not a general functional programming one suitable for `deep_causality_haft`.
    - **Compatibility**: It mirrors the design of `MonadEffect3`, making it compatible with the existing `CausalEffectSystem`.

### Part 2: Implementation for `CausalMonad`

The `Intervenable` trait will be implemented for `CausalMonad` to provide the concrete logic for how an intervention affects a `PropagatingEffect`.

- **File Location**: `deep_causality/src/types/monad_types/causal_monad/mod.rs` 
- **Implementation Logic**:
  ```rust
  use crate::traits::intervenable::Intervenable;
  use crate::{CausalEffectLog, CausalEffectSystem, CausalLogEntry, CausalPropagatingEffect};
  use deep_causality_haft::{Effect3, HKT3};

  impl Intervenable<CausalEffectSystem> for CausalMonad {
      fn intervene<T>(
          effect: CausalPropagatingEffect<T, <CausalEffectSystem as Effect3>::Fixed1, <CausalEffectSystem as Effect3>::Fixed2>,
          new_value: T,
      ) -> CausalPropagatingEffect<T, <CausalEffectSystem as Effect3>::Fixed1, <CausalEffectSystem as Effect3>::Fixed2>
      where
          T: std::fmt::Debug, // Add Debug bound to log the new value
      {
          // 1. Preserve the incoming logs and add a new entry for the intervention.
          let mut new_logs = effect.logs;
          let log_message = format!("Intervention: Value replaced with {:?}", new_value);
          new_logs.add(CausalLogEntry::new("Intervention", &log_message));

          // 2. Construct the new effect.
          CausalPropagatingEffect {
              // The value is replaced with the intervention value.
              value: new_value,
              // The error state is preserved.
              error: effect.error,
              // The updated logs are carried forward.
              logs: new_logs,
          }
      }
  }
  ```
- **Rationale**:
    - **Value Replacement**: The core of the `do()` operation is replacing `effect.value` with `new_value`.
    - **Log Provenance**: The intervention is a critical step in the causal story and must be recorded. The implementation creates a new log entry and appends it to the existing history, maintaining perfect explainability.
    - **Error Propagation**: If `effect.error` is `Some`, it remains `Some`. A counterfactual cannot be applied to a computation that has already failed. This aligns with the short-circuiting behavior of `bind`.

## 3. Conceptual Usage Example

This design enables intuitive, chainable counterfactual reasoning.

```rust
// Conceptual code showing the flow

// Start a causal chain
let initial_effect = CausalMonad::pure(10);

// A function representing a step in the causal chain
let double = |x| CausalMonad::pure(x * 2); // Returns a PropagatingEffect with value 20

// Another step
let add_five = |x| CausalMonad::pure(x + 5); // Returns a PropagatingEffect with value 25

// --- Scenario 1: Observational ---
let final_effect_observed = CausalMonad::bind(initial_effect, double)
    .bind(add_five);
// final_effect_observed.value would be 25

// --- Scenario 2: Interventional (Counterfactual) ---
let initial_effect_2 = CausalMonad::pure(10); // Restart for clarity

let final_effect_intervened = CausalMonad::bind(initial_effect_2, double) // value is 20
    .intervene(50) // INTERVENTION! Discard 20, force value to 50.
    .bind(add_five); // add_five now receives 50, not 20.

// final_effect_intervened.value would be 55.
// The logs would clearly show the chain: pure(10) -> bind(double) -> intervene(50) -> bind(add_five)
```

## 4. Benefits

- **Clean Abstraction**: `Intervenable` is a separate, orthogonal concept to `Monad`, which is correct from a type theory perspective.
- **Preserves Guarantees**: The design maintains the existing system's strict error propagation and complete log history.
- **Expressive Power**: It directly and idiomatically models a core concept of causal inference within the existing monadic framework.
- **Extensibility**: It follows the established pattern of using traits in `deep_causality` to extend the core `haft` abstractions.
