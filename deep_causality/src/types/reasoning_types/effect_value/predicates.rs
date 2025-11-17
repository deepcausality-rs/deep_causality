/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::EffectValue;

impl EffectValue {
    pub fn is_none(&self) -> bool {
        matches!(self, EffectValue::None)
    }

    pub fn is_deterministic(&self) -> bool {
        matches!(self, EffectValue::Boolean(_))
    }

    pub fn is_number(&self) -> bool {
        matches!(self, EffectValue::Number(_))
    }

    pub fn is_numeric(&self) -> bool {
        matches!(self, EffectValue::Numerical(_))
    }

    pub fn is_probabilistic(&self) -> bool {
        matches!(self, EffectValue::Probabilistic(_))
    }

    pub fn is_tensor(&self) -> bool {
        matches!(self, EffectValue::Tensor(_))
    }

    pub fn is_complex(&self) -> bool {
        matches!(self, EffectValue::Complex(_))
    }

    pub fn is_complex_tensor(&self) -> bool {
        matches!(self, EffectValue::ComplexTensor(_))
    }

    pub fn is_quaternion(&self) -> bool {
        matches!(self, EffectValue::Quaternion(_))
    }

    pub fn is_quaternion_tensor(&self) -> bool {
        matches!(self, EffectValue::QuaternionTensor(_))
    }

    pub fn is_uncertain_bool(&self) -> bool {
        matches!(self, EffectValue::UncertainBool(_))
    }

    pub fn is_uncertain_float(&self) -> bool {
        matches!(self, EffectValue::UncertainFloat(_))
    }

    pub fn is_maybe_uncertain_bool(&self) -> bool {
        matches!(self, EffectValue::MaybeUncertainBool(_))
    }

    pub fn is_maybe_uncertain_float(&self) -> bool {
        matches!(self, EffectValue::MaybeUncertainFloat(_))
    }

    pub fn is_contextual_link(&self) -> bool {
        matches!(self, EffectValue::ContextualLink(_, _))
    }

    pub fn is_map(&self) -> bool {
        matches!(self, EffectValue::Map(_))
    }

    pub fn is_relay_to(&self) -> bool {
        matches!(self, EffectValue::RelayTo(_, _))
    }

    pub fn is_external(&self) -> bool {
        matches!(self, EffectValue::External(_))
    }
}
