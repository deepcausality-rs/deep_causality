/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{ContextoidId, EffectValue, IdentificationValue, NumericValue, PropagatingEffect};
use deep_causality_num::{Complex, Quaternion};
use deep_causality_tensor::CausalTensor;
use deep_causality_uncertain::{
    MaybeUncertainBool, MaybeUncertainF64, UncertainBool, UncertainF64,
};
use std::collections::HashMap;

impl EffectValue {
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            EffectValue::Deterministic(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<&NumericValue> {
        match self {
            EffectValue::Number(n) => Some(n),
            _ => None,
        }
    }

    pub fn as_numerical(&self) -> Option<&NumericValue> {
        match self {
            EffectValue::Number(n) => Some(n),
            _ => None,
        }
    }

    pub fn as_probabilistic(&self) -> Option<f64> {
        match self {
            EffectValue::Probabilistic(f) => Some(*f),
            _ => None,
        }
    }

    pub fn as_tensor(&self) -> Option<&CausalTensor<f64>> {
        match self {
            EffectValue::Tensor(t) => Some(t),
            _ => None,
        }
    }

    pub fn as_complex(&self) -> Option<&Complex<f64>> {
        match self {
            EffectValue::Complex(c) => Some(c),
            _ => None,
        }
    }

    pub fn as_complex_tensor(&self) -> Option<&CausalTensor<Complex<f64>>> {
        match self {
            EffectValue::ComplexTensor(t) => Some(t),
            _ => None,
        }
    }

    pub fn as_quaternion(&self) -> Option<&Quaternion<f64>> {
        match self {
            EffectValue::Quaternion(q) => Some(q),
            _ => None,
        }
    }

    pub fn as_quaternion_tensor(&self) -> Option<&CausalTensor<Quaternion<f64>>> {
        match self {
            EffectValue::QuaternionTensor(t) => Some(t),
            _ => None,
        }
    }

    pub fn as_uncertain_bool(&self) -> Option<&UncertainBool> {
        match self {
            EffectValue::UncertainBool(ub) => Some(ub),
            _ => None,
        }
    }

    pub fn as_uncertain_float(&self) -> Option<&UncertainF64> {
        match self {
            EffectValue::UncertainFloat(uf) => Some(uf),
            _ => None,
        }
    }

    pub fn as_maybe_uncertain_bool(&self) -> Option<&MaybeUncertainBool> {
        match self {
            EffectValue::MaybeUncertainBool(mub) => Some(mub),
            _ => None,
        }
    }

    pub fn as_maybe_uncertain_float(&self) -> Option<&MaybeUncertainF64> {
        match self {
            EffectValue::MaybeUncertainFloat(muf) => Some(muf),
            _ => None,
        }
    }

    pub fn as_contextual_link(&self) -> Option<&ContextoidId> {
        match self {
            EffectValue::ContextualLink(coid) => Some(coid),
            _ => None,
        }
    }

    pub fn as_map(&self) -> Option<&HashMap<IdentificationValue, Box<PropagatingEffect>>> {
        match self {
            EffectValue::Map(m) => Some(m),
            _ => None,
        }
    }

    pub fn as_relay_to(&self) -> Option<(&usize, &PropagatingEffect)> {
        match self {
            EffectValue::RelayTo(target, effect) => Some((target, effect)),
            _ => None,
        }
    }
}
