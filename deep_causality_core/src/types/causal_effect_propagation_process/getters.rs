/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{CausalEffectPropagationProcess, EffectValue};

impl<Value, State, Context, Error, Log>
    CausalEffectPropagationProcess<Value, State, Context, Error, Log>
{
    /// The value-XOR-error channel: an effect value, or the error that ended the computation.
    pub const fn outcome(&self) -> &Result<EffectValue<Value>, Error> {
        &self.outcome
    }

    /// The carried scalar, if the process holds a plain [`EffectValue::Value`].
    ///
    /// This is the everyday accessor: it lends `Some(&v)` only when the process carries an
    /// ordinary value, and `None` for an errored process or any non-`Value` effect
    /// (`None`, `ContextualLink`, `RelayTo`, `Map`). Reach for [`value_cloned`](Self::value_cloned)
    /// or [`into_value`](Self::into_value) when you need the value by value, and for
    /// [`effect`](Self::effect) when you need to discriminate the effect variants.
    pub fn value(&self) -> Option<&Value> {
        match self.outcome.as_ref().ok()? {
            EffectValue::Value(v) => Some(v),
            _ => None,
        }
    }

    /// The carried scalar cloned out, if the process holds a plain [`EffectValue::Value`].
    ///
    /// The borrowing, owned-result counterpart to [`value`](Self::value); the non-consuming
    /// counterpart to [`into_value`](Self::into_value).
    pub fn value_cloned(&self) -> Option<Value>
    where
        Value: Clone,
    {
        self.value().cloned()
    }

    /// The carried effect value, or `None` if the process holds an error.
    ///
    /// Yields the full [`EffectValue`] wrapper so callers can discriminate its variants
    /// (`Value`, `None`, `ContextualLink`, `RelayTo`, `Map`) — for dispatch and routing logic.
    /// Most callers want the plain scalar instead: use [`value`](Self::value). An errored process
    /// has no effect to lend — value and error are one channel (the W-invariant, by construction).
    pub fn effect(&self) -> Option<&EffectValue<Value>> {
        self.outcome.as_ref().ok()
    }

    pub const fn state(&self) -> &State {
        &self.state
    }

    pub const fn context(&self) -> &Option<Context> {
        &self.context
    }

    /// The error that ended the computation, or `None` if the process carries a value.
    pub fn error(&self) -> Option<&Error> {
        self.outcome.as_ref().err()
    }

    pub const fn logs(&self) -> &Log {
        &self.logs
    }
}
