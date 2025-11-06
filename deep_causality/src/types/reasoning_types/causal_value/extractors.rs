/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{CausalValue, ContextId, ContextoidId, IdentificationValue, NumericValue};
use deep_causality_num::{Complex, Quaternion};
use deep_causality_tensor::CausalTensor;
use deep_causality_uncertain::{
    MaybeUncertainBool, MaybeUncertainF64, UncertainBool, UncertainF64,
};
use std::collections::HashMap;
use std::sync::Arc;
use ultragraph::UltraGraph;

impl CausalValue {
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            CausalValue::Deterministic(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_numerical(&self) -> Option<&NumericValue> {
        match self {
            CausalValue::Numerical(n) => Some(n),
            _ => None,
        }
    }

    pub fn as_probabilistic(&self) -> Option<f64> {
        match self {
            CausalValue::Probabilistic(f) => Some(*f),
            _ => None,
        }
    }

    pub fn as_tensor(&self) -> Option<&CausalTensor<f64>> {
        match self {
            CausalValue::Tensor(t) => Some(t),
            _ => None,
        }
    }

    pub fn as_complex(&self) -> Option<&Complex<f64>> {
        match self {
            CausalValue::Complex(c) => Some(c),
            _ => None,
        }
    }

    pub fn as_complex_tensor(&self) -> Option<&CausalTensor<Complex<f64>>> {
        match self {
            CausalValue::ComplexTensor(t) => Some(t),
            _ => None,
        }
    }

    pub fn as_quaternion(&self) -> Option<&Quaternion<f64>> {
        match self {
            CausalValue::Quaternion(q) => Some(q),
            _ => None,
        }
    }

    pub fn as_quaternion_tensor(&self) -> Option<&CausalTensor<Quaternion<f64>>> {
        match self {
            CausalValue::QuaternionTensor(t) => Some(t),
            _ => None,
        }
    }

    pub fn as_uncertain_bool(&self) -> Option<&UncertainBool> {
        match self {
            CausalValue::UncertainBool(ub) => Some(ub),
            _ => None,
        }
    }

    pub fn as_uncertain_float(&self) -> Option<&UncertainF64> {
        match self {
            CausalValue::UncertainFloat(uf) => Some(uf),
            _ => None,
        }
    }

    pub fn as_maybe_uncertain_bool(&self) -> Option<&MaybeUncertainBool> {
        match self {
            CausalValue::MaybeUncertainBool(mub) => Some(mub),
            _ => None,
        }
    }

    pub fn as_maybe_uncertain_float(&self) -> Option<&MaybeUncertainF64> {
        match self {
            CausalValue::MaybeUncertainFloat(muf) => Some(muf),
            _ => None,
        }
    }

    pub fn as_contextual_link(&self) -> Option<(&ContextId, &ContextoidId)> {
        match self {
            CausalValue::ContextualLink(cid, coid) => Some((cid, coid)),
            _ => None,
        }
    }

    pub fn as_map(&self) -> Option<&HashMap<IdentificationValue, Box<CausalValue>>> {
        match self {
            CausalValue::Map(m) => Some(m),
            _ => None,
        }
    }

    pub fn as_graph(&self) -> Option<&Arc<UltraGraph<CausalValue>>> {
        match self {
            CausalValue::Graph(g) => Some(g),
            _ => None,
        }
    }

    pub fn as_relay_to(&self) -> Option<(&usize, &CausalValue)> {
        match self {
            CausalValue::RelayTo(target, effect) => Some((target, effect)),
            _ => None,
        }
    }
}
