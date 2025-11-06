/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::CausalEffectValue;

impl CausalEffectValue {
    pub fn is_none(&self) -> bool {
        matches!(self, CausalEffectValue::None)
    }

    pub fn is_deterministic(&self) -> bool {
        matches!(self, CausalEffectValue::Deterministic(_))
    }

    pub fn is_numeric(&self) -> bool {
        matches!(self, CausalEffectValue::Numeric(_))
    }

    pub fn is_probabilistic(&self) -> bool {
        matches!(self, CausalEffectValue::Probabilistic(_))
    }

    pub fn is_tensor(&self) -> bool {
        matches!(self, CausalEffectValue::Tensor(_))
    }

    pub fn is_complex(&self) -> bool {
        matches!(self, CausalEffectValue::Complex(_))
    }

    pub fn is_complex_tensor(&self) -> bool {
        matches!(self, CausalEffectValue::ComplexTensor(_))
    }

    pub fn is_quaternion(&self) -> bool {
        matches!(self, CausalEffectValue::Quaternion(_))
    }

    pub fn is_quaternion_tensor(&self) -> bool {
        matches!(self, CausalEffectValue::QuaternionTensor(_))
    }

    pub fn is_uncertain_bool(&self) -> bool {
        matches!(self, CausalEffectValue::UncertainBool(_))
    }

    pub fn is_uncertain_float(&self) -> bool {
        matches!(self, CausalEffectValue::UncertainFloat(_))
    }

    pub fn is_maybe_uncertain_bool(&self) -> bool {
        matches!(self, CausalEffectValue::MaybeUncertainBool(_))
    }

    pub fn is_maybe_uncertain_float(&self) -> bool {
        matches!(self, CausalEffectValue::MaybeUncertainFloat(_))
    }

    pub fn is_contextual_link(&self) -> bool {
        matches!(self, CausalEffectValue::ContextualLink(_, _))
    }

    pub fn is_map(&self) -> bool {
        matches!(self, CausalEffectValue::Map(_))
    }

    pub fn is_relay_to(&self) -> bool {
        matches!(self, CausalEffectValue::RelayTo(_, _))
    }
}
