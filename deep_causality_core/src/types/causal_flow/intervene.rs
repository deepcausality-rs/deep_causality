/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// =============================================================================
// Value, context and state alternation (counterfactual substitution on one channel)
// =============================================================================
//
// These are the substitution lenses, `alternate_value`, `alternate_context` and `alternate_state`. They are NOT
// Pearl's `do(...)` operator: that is graph surgery (parent-edge deletion / variable isolation) and
// lives at the `deep_causality` Causaloid + hypergraph layer, where a graph is in scope. At the
// value/monad level there is no graph to mutilate, so the honest operation is counterfactual
// substitution of a channel.

use crate::{AlternatableContext, AlternatableState, AlternatableValue, CausalFlow};
use core::fmt::Debug;

impl<Value, State, Context> CausalFlow<Value, State, Context>
where
    Value: Debug,
{
    /// Substitute the carried value with `new_value` (counterfactual value substitution),
    /// recording the override in the audit log. A no-op on a failed flow (the underlying
    /// `AlternatableValue` preserves the error).
    pub fn alternate_value(self, new_value: Value) -> Self {
        CausalFlow {
            inner: self.inner.alternate_value(new_value),
        }
    }
}

impl<Value, State, Context> CausalFlow<Value, State, Context>
where
    Value: Debug + Clone,
{
    /// Alternate only when `cond` holds over the current value: replace it with `f(value)` and log
    /// the override; otherwise pass the value through untouched.
    pub fn alternate_value_if<P, F>(self, cond: P, f: F) -> Self
    where
        P: FnOnce(&Value) -> bool,
        F: FnOnce(Value) -> Value,
    {
        // An errored flow short-circuits without running `cond` or `f`, consistent with
        // `alternate_value` (a no-op on a failed flow). Value and error are one channel, so the
        // formerly representable "value AND error" carrier — which would have executed the
        // user closures before the substitution discarded them — cannot exist.
        match &self.inner.outcome {
            Ok(value) => match value.clone().into_value() {
                Some(v) if cond(&v) => self.alternate_value(f(v)),
                _ => self,
            },
            Err(_) => self,
        }
    }
}

impl<Value, State, Context> CausalFlow<Value, State, Context> {
    /// Substitute the carried context with `new_context` (counterfactual context substitution),
    /// recording the substitution in the audit log as `!!ContextAlternation!!`. Value and state
    /// pass through unchanged, so the steps after it run the same causal law against another
    /// world. A no-op on a failed flow (the underlying `AlternatableContext` preserves the error).
    pub fn alternate_context(self, new_context: Context) -> Self {
        CausalFlow {
            inner: self.inner.alternate_context(new_context),
        }
    }

    /// Substitute the carried state with `new_state` (counterfactual state substitution),
    /// recording the substitution in the audit log as `!!StateAlternation!!`. Value and context
    /// pass through unchanged, so the steps after it run the same causal law from another
    /// history. A no-op on a failed flow (the underlying `AlternatableState` preserves the error).
    pub fn alternate_state(self, new_state: State) -> Self {
        CausalFlow {
            inner: self.inner.alternate_state(new_state),
        }
    }
}

// The alternation traits delegate to the inherent methods above, so a `CausalFlow` satisfies the
// `Alternatable<V, C, S>` bound that generic code places on a carrier of all three channels.

impl<Value, State, Context> AlternatableValue<Value> for CausalFlow<Value, State, Context>
where
    Value: Debug,
{
    fn alternate_value(self, new_value: Value) -> Self {
        CausalFlow::alternate_value(self, new_value)
    }
}

impl<Value, State, Context> AlternatableContext<Context> for CausalFlow<Value, State, Context> {
    fn alternate_context(self, new_context: Context) -> Self {
        CausalFlow::alternate_context(self, new_context)
    }
}

impl<Value, State, Context> AlternatableState<State> for CausalFlow<Value, State, Context> {
    fn alternate_state(self, new_state: State) -> Self {
        CausalFlow::alternate_state(self, new_state)
    }
}
