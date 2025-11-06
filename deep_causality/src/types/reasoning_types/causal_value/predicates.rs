/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::CausalValue;

impl CausalValue {
    pub fn is_none(&self) -> bool {
        matches!(self, CausalValue::None)
    }

    pub fn is_deterministic(&self) -> bool {
        matches!(self, CausalValue::Deterministic(_))
    }

    pub fn is_numerical(&self) -> bool {
        matches!(self, CausalValue::Numerical(_))
    }

    pub fn is_probabilistic(&self) -> bool {
        matches!(self, CausalValue::Probabilistic(_))
    }

    pub fn is_tensor(&self) -> bool {
        matches!(self, CausalValue::Tensor(_))
    }

    pub fn is_complex(&self) -> bool {
        matches!(self, CausalValue::Complex(_))
    }

    pub fn is_complex_tensor(&self) -> bool {
        matches!(self, CausalValue::ComplexTensor(_))
    }

    pub fn is_quaternion(&self) -> bool {
        matches!(self, CausalValue::Quaternion(_))
    }

    pub fn is_quaternion_tensor(&self) -> bool {
        matches!(self, CausalValue::QuaternionTensor(_))
    }

    pub fn is_uncertain_bool(&self) -> bool {
        matches!(self, CausalValue::UncertainBool(_))
    }

    pub fn is_uncertain_float(&self) -> bool {
        matches!(self, CausalValue::UncertainFloat(_))
    }

    pub fn is_maybe_uncertain_bool(&self) -> bool {
        matches!(self, CausalValue::MaybeUncertainBool(_))
    }

    pub fn is_maybe_uncertain_float(&self) -> bool {
        matches!(self, CausalValue::MaybeUncertainFloat(_))
    }

    pub fn is_contextual_link(&self) -> bool {
        matches!(self, CausalValue::ContextualLink(_, _))
    }

    pub fn is_map(&self) -> bool {
        matches!(self, CausalValue::Map(_))
    }

    pub fn is_graph(&self) -> bool {
        matches!(self, CausalValue::Graph(_))
    }

    pub fn is_relay_to(&self) -> bool {
        matches!(self, CausalValue::RelayTo(_, _))
    }
}
