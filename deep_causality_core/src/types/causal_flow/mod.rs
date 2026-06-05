/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # `CausalFlow`: a fluent facade over the causal monad
//!
//! [`CausalFlow`] is a thin builder over [`CausalEffectPropagationProcess`] (through the
//! [`PropagatingEffect`] / [`PropagatingProcess`] aliases). It hides the HKT witness types, the
//! verbose constructors (`pure` / `with_state`), the [`EffectValue`] wrapping, and the manual error
//! short-circuit, so a pipeline reads as a clean flow. Every method lowers to an existing monad
//! operation, so the facade adds sugar, not new semantics.
//!
//! ```
//! use deep_causality_core::CausalFlow;
//!
//! let outcome = CausalFlow::value(2_i64)
//!     .try_step(|x| Ok(x + 3))
//!     .map(|x| x * 10)
//!     .finish();
//! assert_eq!(outcome, Ok(50));
//! ```

use crate::{
    CausalEffectPropagationProcess, CausalityError, CausalityErrorEnum, EffectLog, EffectValue,
    Intervenable, PropagatingEffect, PropagatingProcess,
};
use core::fmt::Debug;

/// A fluent flow over the causal monad. `State` and `Context` default to `()`, so `CausalFlow<T>`
/// is the stateless form (lowering to [`PropagatingEffect`]); a non-unit `State`/`Context` lowers
/// to [`PropagatingProcess`]. `Error` and `Log` are fixed to [`CausalityError`] / [`EffectLog`], so
/// the witness types never appear in a signature a caller reads.
#[derive(Debug, Clone, PartialEq)]
pub struct CausalFlow<Value, State = (), Context = ()> {
    inner: PropagatingProcess<Value, State, Context>,
}

// Free helpers building a leaf process with an empty log (so `bind`/`bind_or_error` carry only the
// upstream logs forward).
#[inline]
fn ok_leaf<U, State, Context>(
    value: U,
    state: State,
    context: Option<Context>,
) -> PropagatingProcess<U, State, Context> {
    CausalEffectPropagationProcess {
        value: EffectValue::Value(value),
        state,
        context,
        error: None,
        logs: EffectLog::new(),
    }
}

#[inline]
fn err_leaf<U, State, Context>(
    err: CausalityError,
    state: State,
    context: Option<Context>,
) -> PropagatingProcess<U, State, Context> {
    CausalEffectPropagationProcess {
        value: EffectValue::None,
        state,
        context,
        error: Some(err),
        logs: EffectLog::new(),
    }
}

// =============================================================================
// Construction (hides `pure` / `with_state` / the witnesses)
// =============================================================================

impl CausalFlow<(), (), ()> {
    /// Start a stateless flow carrying the unit value (the seed for an effect chain).
    pub fn effect() -> Self {
        CausalFlow {
            inner: ok_leaf((), (), None),
        }
    }
}

impl<Value> CausalFlow<Value, (), ()> {
    /// Start a stateless flow carrying `value`.
    pub fn value(value: Value) -> Self {
        CausalFlow {
            inner: ok_leaf(value, (), None),
        }
    }

    /// Start a flow already in the error channel.
    pub fn fail(err: CausalityError) -> Self {
        CausalFlow {
            inner: err_leaf(err, (), None),
        }
    }
}

impl<State> CausalFlow<(), State, ()> {
    /// Start a stateful flow with an initial `state` (lowers to `with_state(pure(()), state, None)`).
    pub fn process(state: State) -> Self {
        CausalFlow {
            inner: ok_leaf((), state, None),
        }
    }
}

impl<Value, State> CausalFlow<Value, State, ()> {
    /// Attach a read-only context, promoting the flow to its stateful/contextful form.
    pub fn context<Context>(self, context: Context) -> CausalFlow<Value, State, Context> {
        let inner = self.inner;
        CausalFlow {
            inner: CausalEffectPropagationProcess {
                value: inner.value,
                state: inner.state,
                context: Some(context),
                error: inner.error,
                logs: inner.logs,
            },
        }
    }
}

// =============================================================================
// Fluent steps (hide `EffectValue` + auto short-circuit)
// =============================================================================

impl<Value, State, Context> CausalFlow<Value, State, Context>
where
    State: Clone,
    Context: Clone,
{
    /// Full monadic step: the closure receives the unwrapped value and returns the next flow.
    /// Effect-returning stages adapt with [`From`] / `.into()`. Short-circuits on error / no value.
    pub fn and_then<U, F>(self, f: F) -> CausalFlow<U, State, Context>
    where
        F: FnOnce(Value) -> CausalFlow<U, State, Context>,
    {
        let inner = self.inner.bind_or_error(
            |v, _state, _context| f(v).inner,
            "and_then received no value",
        );
        CausalFlow { inner }
    }

    /// Common stateless step: `Ok` lifts to a value, `Err` to the error channel.
    pub fn try_step<U, F>(self, f: F) -> CausalFlow<U, State, Context>
    where
        F: FnOnce(Value) -> Result<U, CausalityError>,
    {
        let inner = self.inner.bind_or_error(
            |v, state, context| match f(v) {
                Ok(u) => ok_leaf(u, state, context),
                Err(e) => err_leaf(e, state, context),
            },
            "try_step received no value",
        );
        CausalFlow { inner }
    }

    /// Infallible value transform. `None` passes through, an error short-circuits.
    pub fn map<U, F>(self, f: F) -> CausalFlow<U, State, Context>
    where
        F: FnOnce(Value) -> U,
    {
        let inner = self.inner.bind(|ev, state, context| {
            let value = match ev.into_value() {
                Some(v) => EffectValue::Value(f(v)),
                None => EffectValue::None,
            };
            CausalEffectPropagationProcess {
                value,
                state,
                context,
                error: None,
                logs: EffectLog::new(),
            }
        });
        CausalFlow { inner }
    }

    /// Validate the value: `Ok(())` passes it through, `Err` short-circuits.
    pub fn guard<F>(self, f: F) -> Self
    where
        F: FnOnce(&Value) -> Result<(), CausalityError>,
    {
        let inner = self.inner.bind_or_error(
            |v, state, context| match f(&v) {
                Ok(()) => ok_leaf(v, state, context),
                Err(e) => err_leaf(e, state, context),
            },
            "guard received no value",
        );
        CausalFlow { inner }
    }

    /// Turn the error channel back into a value. A no-op on a successful flow.
    pub fn recover<F>(self, f: F) -> Self
    where
        F: FnOnce(CausalityError) -> Value,
    {
        match self.inner.error {
            Some(err) => CausalFlow {
                inner: CausalEffectPropagationProcess {
                    value: EffectValue::Value(f(err)),
                    state: self.inner.state,
                    context: self.inner.context,
                    error: None,
                    logs: self.inner.logs,
                },
            },
            None => self,
        }
    }

    /// Stateful step with read-only access to state and context; the facade keeps threading them.
    pub fn try_step_with<U, F>(self, f: F) -> CausalFlow<U, State, Context>
    where
        F: FnOnce(Value, &State, Option<&Context>) -> Result<U, CausalityError>,
    {
        let inner = self.inner.bind_or_error(
            |v, state, context| match f(v, &state, context.as_ref()) {
                Ok(u) => ok_leaf(u, state, context),
                Err(e) => err_leaf(e, state, context),
            },
            "try_step_with received no value",
        );
        CausalFlow { inner }
    }

    /// Canonical stateful step: mutate the state while transforming the value, in one closure.
    pub fn step_mut<U, F>(self, f: F) -> CausalFlow<U, State, Context>
    where
        F: FnOnce(Value, &mut State, Option<&Context>) -> Result<U, CausalityError>,
    {
        let inner = self.inner.bind_or_error(
            |v, mut state, context| match f(v, &mut state, context.as_ref()) {
                Ok(u) => ok_leaf(u, state, context),
                Err(e) => err_leaf(e, state, context),
            },
            "step_mut received no value",
        );
        CausalFlow { inner }
    }

    /// Update the carried value in place; state and context pass through unchanged. A same-typed
    /// sibling of [`map`](Self::map) (which can also change the value's type), kept for symmetry with
    /// the rest of the `update_*` family.
    pub fn update_value<F>(self, f: F) -> Self
    where
        F: FnOnce(Value) -> Value,
    {
        let inner = self.inner.bind_or_error(
            |v, state, context| ok_leaf(f(v), state, context),
            "update_value received no value",
        );
        CausalFlow { inner }
    }

    /// Evolve the Markovian state from the current value; the value passes through unchanged. The
    /// closure owns the state and borrows the value.
    pub fn update_state<F>(self, f: F) -> Self
    where
        F: FnOnce(State, &Value) -> State,
    {
        let inner = self.inner.bind_or_error(
            |v, state, context| {
                let state = f(state, &v);
                ok_leaf(v, state, context)
            },
            "update_state received no value",
        );
        CausalFlow { inner }
    }

    /// Evolve the read-only context from the current value; the value passes through unchanged. The
    /// closure owns the context and borrows the value.
    pub fn update_context<F>(self, f: F) -> Self
    where
        F: FnOnce(Option<Context>, &Value) -> Option<Context>,
    {
        let inner = self.inner.bind_or_error(
            |v, state, context| {
                let context = f(context, &v);
                ok_leaf(v, state, context)
            },
            "update_context received no value",
        );
        CausalFlow { inner }
    }

    /// Rewrite all three channels at once: the closure owns the value, state, and context and returns
    /// the next triple. This is the one operator that touches value, state, and context together; the
    /// confined `update_value` / `update_state` / `update_context` each evolve a single channel.
    pub fn update_value_state_context<F>(self, f: F) -> Self
    where
        F: FnOnce(Value, State, Option<Context>) -> (Value, State, Option<Context>),
    {
        let inner = self.inner.bind_or_error(
            |v, state, context| {
                let (value, state, context) = f(v, state, context);
                ok_leaf(value, state, context)
            },
            "update_value_state_context received no value",
        );
        CausalFlow { inner }
    }

    /// Drop-in passthrough to the underlying monad's `bind` (accepts an existing stage signature).
    pub fn bind<U, F>(self, f: F) -> CausalFlow<U, State, Context>
    where
        F: FnOnce(
            EffectValue<Value>,
            State,
            Option<Context>,
        ) -> PropagatingProcess<U, State, Context>,
    {
        CausalFlow {
            inner: self.inner.bind(f),
        }
    }

    /// Drop-in passthrough to the underlying monad's `bind_or_error`.
    pub fn bind_or_error<U, F>(self, f: F, err_msg: &str) -> CausalFlow<U, State, Context>
    where
        F: FnOnce(Value, State, Option<Context>) -> PropagatingProcess<U, State, Context>,
    {
        CausalFlow {
            inner: self.inner.bind_or_error(f, err_msg),
        }
    }
}

// =============================================================================
// Closed-loop intervention (Pearl Layer 2)
// =============================================================================

impl<Value, State, Context> CausalFlow<Value, State, Context>
where
    Value: Debug,
{
    /// Force-substitute the carried value (an interventional `do(value)`), recording the override
    /// in the audit log. A no-op on a failed flow (the underlying `Intervenable` preserves the error).
    pub fn intervene(self, new_value: Value) -> Self {
        CausalFlow {
            inner: self.inner.intervene(new_value),
        }
    }
}

impl<Value, State, Context> CausalFlow<Value, State, Context>
where
    Value: Debug + Clone,
{
    /// Intervene only when `cond` holds over the current value: replace it with `f(value)` and log
    /// the override; otherwise pass the value through untouched.
    pub fn intervene_if<P, F>(self, cond: P, f: F) -> Self
    where
        P: FnOnce(&Value) -> bool,
        F: FnOnce(Value) -> Value,
    {
        match self.inner.value.clone().into_value() {
            Some(v) if cond(&v) => self.intervene(f(v)),
            _ => self,
        }
    }
}

// =============================================================================
// Terminals and interop
// =============================================================================

impl<Value, State, Context> CausalFlow<Value, State, Context> {
    /// Extract the final value, or the error the flow short-circuited with.
    pub fn finish(self) -> Result<Value, CausalityError> {
        if let Some(err) = self.inner.error {
            return Err(err);
        }
        self.inner
            .value
            .into_value()
            .ok_or_else(|| CausalityError::new(CausalityErrorEnum::ValueNotAvailable))
    }

    /// Consume the flow, dispatching to `on_ok` or `on_err` by outcome.
    pub fn run<OnOk, OnErr>(self, on_ok: OnOk, on_err: OnErr)
    where
        OnOk: FnOnce(Value),
        OnErr: FnOnce(CausalityError),
    {
        match self.finish() {
            Ok(v) => on_ok(v),
            Err(e) => on_err(e),
        }
    }

    /// Whether the flow is in the error channel.
    pub fn is_err(&self) -> bool {
        self.inner.error.is_some()
    }

    /// Drop back to the underlying process for interop with code that expects the concrete type.
    pub fn into_process(self) -> PropagatingProcess<Value, State, Context> {
        self.inner
    }
}

impl<Value> CausalFlow<Value, (), ()> {
    /// Drop back to the underlying stateless effect.
    pub fn into_effect(self) -> PropagatingEffect<Value> {
        self.inner
    }
}

impl<Value, State, Context> From<PropagatingProcess<Value, State, Context>>
    for CausalFlow<Value, State, Context>
{
    fn from(inner: PropagatingProcess<Value, State, Context>) -> Self {
        CausalFlow { inner }
    }
}
