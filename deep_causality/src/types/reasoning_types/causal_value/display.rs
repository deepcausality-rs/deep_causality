/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::CausalValue;
use std::fmt::Display;

impl Display for CausalValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CausalValue::None => write!(f, "None"),
            CausalValue::Deterministic(b) => write!(f, "Deterministic({})", b),
            CausalValue::Numerical(n) => write!(f, "Numerical({})", n),
            CausalValue::Probabilistic(n) => write!(f, "Probabilistic({})", n),
            CausalValue::Tensor(t) => write!(f, "Tensor({:?})", t),
            CausalValue::Complex(c) => write!(f, "Complex({:?})", c),
            CausalValue::ComplexTensor(t) => write!(f, "ComplexTensor({:?})", t),
            CausalValue::Quaternion(q) => write!(f, "Quaternion({:?})", q),
            CausalValue::QuaternionTensor(q) => write!(f, "QuaternionTensor({:?})", q),
            CausalValue::UncertainBool(ub) => write!(f, "UncertainBool({:?})", ub),
            CausalValue::UncertainFloat(uf) => write!(f, "UncertainFloat({:?})", uf),
            CausalValue::MaybeUncertainBool(mub) => write!(f, "MaybeUncertainBool({:?})", mub),
            CausalValue::MaybeUncertainFloat(muf) => write!(f, "MaybeUncertainFloat({:?})", muf),
            CausalValue::ContextualLink(cid, coid) => {
                write!(
                    f,
                    "ContextualLink(ContextId: {}, ContextoidId: {})",
                    cid, coid
                )
            }
            CausalValue::Map(m) => write!(f, "Map({:?})", m),
            CausalValue::Graph(g) => write!(f, "Graph({:?})", g),
            CausalValue::RelayTo(target, effect) => {
                write!(f, "RelayTo(target: {}, effect: {:?})", target, effect)
            }
        }
    }
}
