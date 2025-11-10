/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::EffectValue;
use std::fmt::Display;

impl Display for EffectValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EffectValue::None => write!(f, "None"),
            EffectValue::Deterministic(b) => write!(f, "Deterministic({})", b),
            EffectValue::Number(n) => write!(f, "Numerical({})", n),
            EffectValue::Numerical(b) => write!(f, "Numerical({})", b),
            EffectValue::Probabilistic(n) => write!(f, "Probabilistic({})", n),
            EffectValue::Tensor(t) => write!(f, "Tensor({:?})", t),
            EffectValue::Complex(c) => write!(f, "Complex({:?})", c),
            EffectValue::ComplexTensor(t) => write!(f, "ComplexTensor({:?})", t),
            EffectValue::Quaternion(q) => write!(f, "Quaternion({:?})", q),
            EffectValue::QuaternionTensor(q) => write!(f, "QuaternionTensor({:?})", q),
            EffectValue::UncertainBool(ub) => write!(f, "UncertainBool({:?})", ub),
            EffectValue::UncertainFloat(uf) => write!(f, "UncertainFloat({:?})", uf),
            EffectValue::MaybeUncertainBool(mub) => {
                write!(f, "MaybeUncertainBool({:?})", mub)
            }
            EffectValue::MaybeUncertainFloat(muf) => {
                write!(f, "MaybeUncertainFloat({:?})", muf)
            }
            EffectValue::ContextualLink(coid) => {
                write!(f, "ContextualLink(ContextoidId: {})", coid)
            }
            EffectValue::Map(m) => write!(f, "Map({:?})", m),
            EffectValue::RelayTo(target, effect) => {
                write!(f, "RelayTo(target: {}, effect: {:?})", target, effect)
            }
        }
    }
}
