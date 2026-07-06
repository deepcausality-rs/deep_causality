/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// =============================================================================
// Closed-loop intervention (Pearl Layer 2)
// =============================================================================

use crate::{CausalFlow, Intervenable};
use core::fmt::Debug;

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
        // An errored flow short-circuits without running `cond` or `f`, consistent with
        // `intervene` (a no-op on a failed flow). Value and error are one channel, so the
        // formerly representable "value AND error" carrier — which would have executed the
        // user closures before `intervene` discarded them — cannot exist.
        match &self.inner.outcome {
            Ok(value) => match value.clone().into_value() {
                Some(v) if cond(&v) => self.intervene(f(v)),
                _ => self,
            },
            Err(_) => self,
        }
    }
}
