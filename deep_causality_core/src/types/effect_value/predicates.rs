/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::effect_value::EffectValue;

impl<T> EffectValue<T> {
    pub fn is_none(&self) -> bool {
        matches!(self, EffectValue::None)
    }

    pub fn is_value(&self) -> bool {
        matches!(self, EffectValue::Value(_))
    }

    pub fn is_contextual_link(&self) -> bool {
        matches!(self, EffectValue::ContextualLink(_, _))
    }

    pub fn is_relay_to(&self) -> bool {
        matches!(self, EffectValue::RelayTo(_, _))
    }

    #[cfg(feature = "std")]
    pub fn is_map(&self) -> bool {
        matches!(self, EffectValue::Map(_))
    }

    pub fn as_value(&self) -> Option<&T> {
        match self {
            EffectValue::Value(v) => Some(v),
            _ => None,
        }
    }

    pub fn into_value(self) -> Option<T> {
        match self {
            EffectValue::Value(v) => Some(v),
            _ => None,
        }
    }
}
