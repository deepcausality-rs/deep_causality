/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::PropagatingEffect;

// Predicates
impl PropagatingEffect {
    pub fn is_none(&self) -> bool {
        matches!(self, PropagatingEffect::None)
    }
    pub fn is_deterministic(&self) -> bool {
        matches!(self, PropagatingEffect::Deterministic(_))
    }
    pub fn is_numerical(&self) -> bool {
        matches!(self, PropagatingEffect::Numerical(_))
    }
    pub fn is_probabilistic(&self) -> bool {
        matches!(self, PropagatingEffect::Probabilistic(_))
    }
    pub fn is_tensor(&self) -> bool {
        matches!(self, PropagatingEffect::Tensor(_))
    }
    pub fn is_complex_tensor(&self) -> bool {
        matches!(self, PropagatingEffect::ComplexTensor(_))
    }
    pub fn is_uncertain_bool(&self) -> bool {
        matches!(self, PropagatingEffect::UncertainBool(_))
    }
    pub fn is_uncertain_float(&self) -> bool {
        matches!(self, PropagatingEffect::UncertainFloat(_))
    }
    pub fn is_maybe_uncertain_bool(&self) -> bool {
        matches!(self, PropagatingEffect::MaybeUncertainBool(_))
    }
    pub fn is_maybe_uncertain_float(&self) -> bool {
        matches!(self, PropagatingEffect::MaybeUncertainFloat(_))
    }
    pub fn is_contextual_link(&self) -> bool {
        matches!(self, PropagatingEffect::ContextualLink(_, _))
    }
    pub fn is_map(&self) -> bool {
        matches!(self, PropagatingEffect::Map(_))
    }
    pub fn is_graph(&self) -> bool {
        matches!(self, PropagatingEffect::Graph(_))
    }
    pub fn is_relay_to(&self) -> bool {
        matches!(self, PropagatingEffect::RelayTo(_, _))
    }
}
