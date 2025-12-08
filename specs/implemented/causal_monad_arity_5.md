# Specification: Unifying `CausalMonad` to Arity-5

**Note:** This document outlines a necessary preparatory refactoring of the core monadic structures in `deep_causality_core`. The goal is to create a single, unified, arity-5 monadic system that can accommodate the existing `PropagatingEffect` as well as the new stateful `PropagatingProcess`. This ensures a clean, consistent, and scalable foundation.

The existing `PropagatingEffect` serves as a powerful tool for non-markovian causal chains, where the outcome of a step depends solely on the value of the preceding one. It follows the pattern of a `Writer` monad (for logging) combined with an `Either` monad (for error handling).

However, many real-world causal systems are **Markovian**: the next state of the system depends on its current state, not just the immediate value. `PropagatingProcess` is designed to fill this gap. It extends the concept of `PropagatingEffect` by explicitly including fields for `State` and `Context`, enabling the modeling of such state-dependent processes.

## 1. Unification Strategy

The current `CausalMonad` is built on `MonadEffect3`, specifically for the arity-3 `PropagatingEffect`. The introduction of the arity-5 `PropagatingProcess` requires a more general monadic framework. Instead of maintaining two separate monads, we will unify them.

The strategy is as follows:

1.  **Create a Base Type**: A new base struct, `CausalEffectPropagationProcess`, will be created to serve as the single underlying container for all monadic effects.
2.  **Define Type Aliases**: Both `PropagatingEffect` and `PropagatingProcess` will be redefined as type aliases of the base `CausalEffectPropagationProcess`, each fixing a different set of generic parameters.
3.  **Adopt Arity-5 Traits**: The core framework will be rebuilt around the `Effect5` and `MonadEffect5` traits from `deep_causality_haft`.
4.  **Create a Generic `CausalSystem`**: A new generic `CausalSystem<S, C>` will be created to implement `Effect5`.
5.  **Re-implement `CausalMonad`**: The `CausalMonad` will be re-implemented to be generic over `State` and `Context`, implementing `MonadEffect5`.
6.  **Provide a "Fluent" `bind`**: A developer-facing `bind` method will be implemented directly on the base `CausalEffectPropagationProcess` struct to provide ergonomic, stateful chaining.

## 2. Core Data Structure Refactoring

### 2.1. Base Type: `CausalEffectPropagationProcess`

This arity-5 struct will be the single foundational container for all effects, located in its own module.

```rust
// in deep_causality_core::types::causal_effect_propagation_process::mod.rs
use crate::EffectValue;

#[derive(Debug, PartialEq, Clone)]
pub struct CausalEffectPropagationProcess<Value, State, Context, Error, Log> {
    pub value: EffectValue<Value>,
    pub state: State,
    pub context: Option<Context>,
    pub error: Option<Error>,
    pub logs: Log,
}
```

### 2.2. Type Alias Definitions

With the new base type, `PropagatingEffect` and `PropagatingProcess` become clear, descriptive aliases.

#### `PropagatingEffect`
Represents a stateless, context-free effect, primarily used for simple value/error/log propagation.

```rust
// in deep_causality_core::types::propagating_effect.rs (updated)
use crate::types::causal_effect_propagation_process::CausalEffectPropagationProcess;
use crate::{CausalityError, EffectLog};

pub type PropagatingEffect<T> = CausalEffectPropagationProcess<T, (), (), CausalityError, EffectLog>;
```

#### `PropagatingProcess`
Represents a stateful, context-aware process with fixed error and log types.

```rust
// in deep_causality_core::types::propagating_process.rs (updated)
use crate::types::causal_effect_propagation_process::CausalEffectPropagationProcess;
use crate::{CausalityError, EffectLog};

pub type PropagatingProcess<T, S, C> = CausalEffectPropagationProcess<T, S, C, CausalityError, EffectLog>;
```

## 3. HKT and Monad Implementation

### 3.1. `CausalEffectPropagationProcessWitness`

The HKT witness is renamed to match the base struct and moved to the new module.

```rust
// in deep_causality_core::types::causal_effect_propagation_process::hkt.rs
use deep_causality_haft::{HKT, HKT5, Placeholder};
use std::marker::PhantomData;
use crate::CausalEffectPropagationProcess;

pub struct CausalEffectPropagationProcessWitness<S, C, E, L>(
    Placeholder,
    PhantomData<S>,
    PhantomData<C>,
    PhantomData<E>,
    PhantomData<L>,
);

// Impl for arity-5 fixed-effect HKT
impl<S, C, E, L> HKT5<S, C, E, L> for CausalEffectPropagationProcessWitness<S, C, E, L> {
    type Type<Value> = CausalEffectPropagationProcess<Value, S, C, E, L>;
}

// Impl for arity-1 HKT, required by Functor/Monad bounds on Effect5
impl<S, C, E, L> HKT for CausalEffectPropagationProcessWitness<S, C, E, L> {
    type Type<Value> = CausalEffectPropagationProcess<Value, S, C, E, L>;
}
```

### 3.2. Unified `CausalSystem`

This struct implements `Effect5` and is generic over State and Context.

```rust
// in deep_causality_core::types::causal_system.rs (new or updated file)
use deep_causality_haft::Effect5;
use crate::types::causal_effect_propagation_process::hkt::CausalEffectPropagationProcessWitness;
use crate::{CausalityError, EffectLog};
use std::marker::PhantomData;

pub struct CausalSystem<S, C>(PhantomData<(S, C)>);

impl<S, C> Effect5 for CausalSystem<S, C>
where
    S: Clone + Default,
    C: Clone,
{
    type Fixed1 = S;
    type Fixed2 = C;
    type Fixed3 = CausalityError;
    type Fixed4 = EffectLog;

    type HktWitness = CausalEffectPropagationProcessWitness<
        Self::Fixed1,
        Self::Fixed2,
        Self::Fixed3,
        Self::Fixed4,
    >;
}
```

### 3.3. Unified `CausalMonad`

`CausalMonad` is now generic and implements `MonadEffect5`. This implementation is "shallow" regarding state to conform to the trait's `bind` signature.

```rust
// in deep_causality_core::types::causal_monad.rs (updated)
use crate::types::causal_system::CausalSystem;
use crate::{CausalEffectPropagationProcess, EffectValue, EffectLog, CausalityError};
use deep_causality_haft::{MonadEffect5, HKT5, LogAppend, Functor, Effect5};
use std::marker::PhantomData;

pub struct CausalMonad<S, C>(PhantomData<(S, C)>);

impl<S, C> MonadEffect5<CausalSystem<S, C>> for CausalMonad<S, C>
where
    S: Clone + Default,
    C: Clone,
    <CausalSystem<S, C> as Effect5>::HktWitness: Functor<<CausalSystem<S, C> as Effect5>::HktWitness> + Sized,
{
    fn pure<T>(value: T) -> CausalEffectPropagationProcess<T, S, C, CausalityError, EffectLog> {
        CausalEffectPropagationProcess {
            value: EffectValue::Value(value),
            state: S::default(),
            context: None,
            error: None,
            logs: EffectLog::new(),
        }
    }

    fn bind<T, U, Func>(
        process: CausalEffectPropagationProcess<T, S, C, CausalityError, EffectLog>,
        mut f: Func,
    ) -> CausalEffectPropagationProcess<U, S, C, CausalityError, EffectLog>
    where
        Func: FnMut(T) -> CausalEffectPropagationProcess<U, S, C, CausalityError, EffectLog>,
        U: Default,
    {
        if let Some(error) = process.error {
            return CausalEffectPropagationProcess {
                value: EffectValue::Value(U::default()),
                state: process.state,
                context: process.context,
                error: Some(error),
                logs: process.logs,
            };
        }

        let value = process.value.into_value().expect("Bind on non-error process");
        let mut next_process = f(value);

        next_process.state = process.state;
        next_process.context = process.context;

        let mut combined_logs = process.logs;
        combined_logs.append(&mut next_process.logs);
        next_process.logs = combined_logs;

        next_process
    }
}
```

## 4. Developer-Facing Stateful `bind` Method

To provide true Markovian behavior, a fluent `bind` method is implemented on the base `CausalEffectPropagationProcess` struct. **This is the intended API for chaining stateful processes.**

```rust
// in deep_causality_core::types::causal_effect_propagation_process::mod.rs (additions)
use deep_causality_haft::LogAppend;

impl<Value, State, Context, Error, Log> CausalEffectPropagationProcess<Value, State, Context, Error, Log>
where
    Log: LogAppend + Default,
    State: Clone,
    Context: Clone,
    Error: Clone,
{
    /// Chains a stateful, context-aware computation.
    ///
    /// This is the primary method for building Markovian process chains, as the
    /// function `f` receives the value, state, and context from the previous step.
    pub fn bind<F, NewValue>(self, f: F) -> CausalEffectPropagationProcess<NewValue, State, Context, Error, Log>
    where
        F: FnOnce(EffectValue<Value>, State, Option<Context>) -> CausalEffectPropagationProcess<NewValue, State, Context, Error, Log>,
        NewValue: Default,
    {
        if let Some(error) = self.error {
            return CausalEffectPropagationProcess {
                value: EffectValue::default(),
                state: self.state,
                context: self.context,
                error: Some(error),
                logs: self.logs,
            };
        }

        let mut next_process = f(self.value, self.state, self.context);

        let mut combined_logs = self.logs;
        combined_logs.append(&mut next_process.logs);
        next_process.logs = combined_logs;

        next_process
    }
}
```

## 5. Lifting a Stateless Effect to a Stateful Process

To begin a stateful computation from a simpler, stateless effect (i.e., a `PropagatingEffect`), we need a way to "lift" it by injecting an initial state and context. This is achieved with a dedicated constructor on the base type.

```rust
// in deep_causality_core::types::causal_effect_propagation_process::mod.rs (additions)

impl<Value, State, Context, Error, Log> CausalEffectPropagationProcess<Value, State, Context, Error, Log>
where
    // assumes previous impl block is available
    Log: Clone,
    Error: Clone,
{
    /// Lifts a stateless effect into a stateful process by providing an initial state and context.
    ///
    /// This is the primary entry point for starting a stateful computation chain from a
    /// simple, pre-existing effect.
    ///
    /// # Arguments
    /// * `effect`: The stateless `PropagatingEffect` (where State and Context are `()`).
    /// * `initial_state`: The starting state for the new process.
    /// * `initial_context`: The optional starting context for the new process.
    ///
    /// # Returns
    /// A new `CausalEffectPropagationProcess` ready for stateful operations.
    pub fn with_state(
        effect: CausalEffectPropagationProcess<Value, (), (), Error, Log>,
        initial_state: State,
        initial_context: Option<Context>,
    ) -> Self {
        Self {
            value: effect.value,
            state: initial_state,
            context: initial_context,
            error: effect.error,
            logs: effect.logs,
        }
    }
}
```
