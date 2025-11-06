/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::CausalValue;
use std::sync::Arc;

impl PartialEq for CausalValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (CausalValue::None, CausalValue::None) => true,
            (CausalValue::Deterministic(l0), CausalValue::Deterministic(r0)) => l0 == r0,
            (CausalValue::Numerical(l0), CausalValue::Numerical(r0)) => l0 == r0,
            (CausalValue::Probabilistic(l0), CausalValue::Probabilistic(r0)) => l0 == r0,
            (CausalValue::Tensor(l0), CausalValue::Tensor(r0)) => l0 == r0,
            (CausalValue::Complex(l0), CausalValue::Complex(r0)) => l0 == r0,
            (CausalValue::ComplexTensor(l0), CausalValue::ComplexTensor(r0)) => l0 == r0,
            (CausalValue::Quaternion(l0), CausalValue::Quaternion(r0)) => l0 == r0,
            (CausalValue::QuaternionTensor(l0), CausalValue::QuaternionTensor(r0)) => l0 == r0,
            (CausalValue::UncertainBool(l0), CausalValue::UncertainBool(r0)) => l0 == r0,
            (CausalValue::UncertainFloat(l0), CausalValue::UncertainFloat(r0)) => l0 == r0,
            (CausalValue::MaybeUncertainBool(l0), CausalValue::MaybeUncertainBool(r0)) => l0 == r0,
            (CausalValue::MaybeUncertainFloat(l0), CausalValue::MaybeUncertainFloat(r0)) => {
                l0 == r0
            }
            (CausalValue::ContextualLink(l0, l1), CausalValue::ContextualLink(r0, r1)) => {
                l0 == r0 && l1 == r1
            }
            (CausalValue::Map(l0), CausalValue::Map(r0)) => l0 == r0,
            (CausalValue::Graph(l0), CausalValue::Graph(r0)) => Arc::ptr_eq(l0, r0),
            (CausalValue::RelayTo(l0, l1), CausalValue::RelayTo(r0, r1)) => l0 == r0 && l1 == r1,
            _ => false,
        }
    }
}
