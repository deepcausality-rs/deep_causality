/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// =============================================================================
// Branches — conditional routing with flow-endomorphism arms
// =============================================================================
//
// A branch routes the flow to one of two continuations. The arms are flow endomorphisms
// (`CausalFlow<V, S, C> -> CausalFlow<V, S, C>`), so they keep threading state, context, and logs,
// and an arm can itself be a composable sub-pipeline. An errored flow short-circuits without running
// either arm.

use crate::{CausalEffectPropagationProcess, CausalFlow, EffectValue, Either};

impl<Value, State, Context> CausalFlow<Value, State, Context>
where
    State: Clone,
    Context: Clone,
{
    /// Route the flow by a predicate over the carried value: `on_true` runs when `cond` holds,
    /// `on_false` otherwise. An errored or value-less flow passes through unchanged.
    pub fn branch<P, T, F>(self, cond: P, on_true: T, on_false: F) -> Self
    where
        P: FnOnce(&Value) -> bool,
        T: FnOnce(Self) -> Self,
        F: FnOnce(Self) -> Self,
    {
        if self.is_err() {
            return self;
        }
        let take_true = match &self.inner.value {
            EffectValue::Value(v) => Some(cond(v)),
            _ => None,
        };
        match take_true {
            Some(true) => on_true(self),
            Some(false) => on_false(self),
            None => self,
        }
    }

    /// Route the flow by a predicate over the carried value, the state, and the context. Otherwise
    /// identical to [`branch`](Self::branch).
    pub fn branch_with<P, T, F>(self, cond: P, on_true: T, on_false: F) -> Self
    where
        P: FnOnce(&Value, &State, Option<&Context>) -> bool,
        T: FnOnce(Self) -> Self,
        F: FnOnce(Self) -> Self,
    {
        if self.is_err() {
            return self;
        }
        let take_true = match &self.inner.value {
            EffectValue::Value(v) => Some(cond(v, &self.inner.state, self.inner.context.as_ref())),
            _ => None,
        };
        match take_true {
            Some(true) => on_true(self),
            Some(false) => on_false(self),
            None => self,
        }
    }
}

impl<L, R, State, Context> CausalFlow<Either<L, R>, State, Context>
where
    State: Clone,
    Context: Clone,
{
    /// Route a flow whose value is `Either<L, R>` to its arm: `left` on `Left`, `right` on `Right`.
    /// An errored flow short-circuits without running either arm; a value-less flow passes through
    /// as a value-less `CausalFlow<U>`.
    pub fn either<U, FL, FR>(self, left: FL, right: FR) -> CausalFlow<U, State, Context>
    where
        FL: FnOnce(CausalFlow<L, State, Context>) -> CausalFlow<U, State, Context>,
        FR: FnOnce(CausalFlow<R, State, Context>) -> CausalFlow<U, State, Context>,
    {
        let inner = self.inner;
        // An errored or value-less carrier carries no `Either`, so re-thread state/context/logs into
        // a `CausalFlow<U>` without running an arm.
        let (value, state, context, error, logs) = (
            inner.value,
            inner.state,
            inner.context,
            inner.error,
            inner.logs,
        );
        if error.is_some() {
            return CausalFlow {
                inner: CausalEffectPropagationProcess {
                    value: EffectValue::None,
                    state,
                    context,
                    error,
                    logs,
                },
            };
        }
        match value {
            EffectValue::Value(Either::Left(l)) => left(CausalFlow {
                inner: CausalEffectPropagationProcess {
                    value: EffectValue::Value(l),
                    state,
                    context,
                    error: None,
                    logs,
                },
            }),
            EffectValue::Value(Either::Right(r)) => right(CausalFlow {
                inner: CausalEffectPropagationProcess {
                    value: EffectValue::Value(r),
                    state,
                    context,
                    error: None,
                    logs,
                },
            }),
            _ => CausalFlow {
                inner: CausalEffectPropagationProcess {
                    value: EffectValue::None,
                    state,
                    context,
                    error: None,
                    logs,
                },
            },
        }
    }
}
