/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::EffectValue;

impl<T: PartialEq> PartialEq for EffectValue<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (EffectValue::None, EffectValue::None) => true,
            (EffectValue::Value(a), EffectValue::Value(b)) => a == b,
            (EffectValue::ContextualLink(a1, a2), EffectValue::ContextualLink(b1, b2)) => {
                a1 == b1 && a2 == b2
            }
            (EffectValue::RelayTo(a_target, _), EffectValue::RelayTo(b_target, _)) => {
                a_target == b_target
            }
            #[cfg(feature = "std")]
            (EffectValue::Map(_), EffectValue::Map(_)) => false, // Maps are not comparable
            _ => false,
        }
    }
}
