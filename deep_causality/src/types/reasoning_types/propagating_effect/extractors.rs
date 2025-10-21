/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{ContextId, ContextoidId, NumericalValue, PropagatingEffect};
use deep_causality_tensor::CausalTensor;
use deep_causality_uncertain::{MaybeUncertain, Uncertain};

// Extractors
impl PropagatingEffect {
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            PropagatingEffect::Deterministic(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_numerical(&self) -> Option<NumericalValue> {
        match self {
            PropagatingEffect::Numerical(p) => Some(*p),
            _ => None,
        }
    }

    pub fn as_probability(&self) -> Option<NumericalValue> {
        match self {
            PropagatingEffect::Probabilistic(p) => Some(*p),
            _ => None,
        }
    }

    pub fn as_tensor(&self) -> Option<CausalTensor<f64>> {
        match self {
            PropagatingEffect::Tensor(p) => Some(p.clone()),
            _ => None,
        }
    }

    pub fn as_uncertain_bool(&self) -> Option<Uncertain<bool>> {
        match self {
            PropagatingEffect::UncertainBool(b) => Some(b.clone()),
            _ => None,
        }
    }

    pub fn as_uncertain_float(&self) -> Option<Uncertain<f64>> {
        match self {
            PropagatingEffect::UncertainFloat(b) => Some(b.clone()),
            _ => None,
        }
    }

    pub fn as_maybe_uncertain_bool(&self) -> Option<MaybeUncertain<bool>> {
        match self {
            PropagatingEffect::MaybeUncertainBool(b) => Some(b.clone()),
            _ => None,
        }
    }

    pub fn as_maybe_uncertain_float(&self) -> Option<MaybeUncertain<f64>> {
        match self {
            PropagatingEffect::MaybeUncertainFloat(b) => Some(b.clone()),
            _ => None,
        }
    }

    pub fn as_contextual_link(&self) -> Option<(ContextId, ContextoidId)> {
        match self {
            PropagatingEffect::ContextualLink(context_id, contextoid_id) => {
                Some((*context_id, *contextoid_id))
            }
            _ => None,
        }
    }
}
