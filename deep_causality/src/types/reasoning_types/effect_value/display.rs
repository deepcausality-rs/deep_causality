/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::CausalEffectValue;
use std::fmt::Display;

impl Display for CausalEffectValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CausalEffectValue::None => write!(f, "None"),
            CausalEffectValue::Deterministic(b) => write!(f, "Deterministic({})", b),
            CausalEffectValue::Numeric(n) => write!(f, "Numerical({})", n),
            CausalEffectValue::Probabilistic(n) => write!(f, "Probabilistic({})", n),
            CausalEffectValue::Tensor(t) => write!(f, "Tensor({:?})", t),
            CausalEffectValue::Complex(c) => write!(f, "Complex({:?})", c),
            CausalEffectValue::ComplexTensor(t) => write!(f, "ComplexTensor({:?})", t),
            CausalEffectValue::Quaternion(q) => write!(f, "Quaternion({:?})", q),
            CausalEffectValue::QuaternionTensor(q) => write!(f, "QuaternionTensor({:?})", q),
            CausalEffectValue::UncertainBool(ub) => write!(f, "UncertainBool({:?})", ub),
            CausalEffectValue::UncertainFloat(uf) => write!(f, "UncertainFloat({:?})", uf),
            CausalEffectValue::MaybeUncertainBool(mub) => {
                write!(f, "MaybeUncertainBool({:?})", mub)
            }
            CausalEffectValue::MaybeUncertainFloat(muf) => {
                write!(f, "MaybeUncertainFloat({:?})", muf)
            }
            CausalEffectValue::ContextualLink(cid, coid) => {
                write!(
                    f,
                    "ContextualLink(ContextId: {}, ContextoidId: {})",
                    cid, coid
                )
            }
            CausalEffectValue::Map(m) => write!(f, "Map({:?})", m),
            CausalEffectValue::RelayTo(target, effect) => {
                write!(f, "RelayTo(target: {}, effect: {:?})", target, effect)
            }
        }
    }
}
