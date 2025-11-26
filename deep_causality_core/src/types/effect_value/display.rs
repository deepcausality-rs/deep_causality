/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::effect_value::EffectValue;
use core::fmt::Display;
use core::fmt::Formatter;

impl<T: Display> Display for EffectValue<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            EffectValue::None => write!(f, "None"),
            EffectValue::Value(v) => write!(f, "Value({})", v),
            EffectValue::ContextualLink(ctx_id, ctxoid_id) => {
                write!(f, "ContextualLink({}, {})", ctx_id, ctxoid_id)
            }
            EffectValue::RelayTo(target, _) => write!(f, "RelayTo({})", target),
            #[cfg(feature = "std")]
            EffectValue::Map(_) => write!(f, "Map(...)"),
            EffectValue::External(_) => write!(f, "External(...)"),
        }
    }
}
