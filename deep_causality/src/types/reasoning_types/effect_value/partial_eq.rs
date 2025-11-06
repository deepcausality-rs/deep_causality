/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::CausalEffectValue;

impl PartialEq for CausalEffectValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (CausalEffectValue::None, CausalEffectValue::None) => true,
            (CausalEffectValue::Deterministic(l0), CausalEffectValue::Deterministic(r0)) => {
                l0 == r0
            }
            (CausalEffectValue::Numeric(l0), CausalEffectValue::Numeric(r0)) => l0 == r0,
            (CausalEffectValue::Probabilistic(l0), CausalEffectValue::Probabilistic(r0)) => {
                l0 == r0
            }
            (CausalEffectValue::Tensor(l0), CausalEffectValue::Tensor(r0)) => l0 == r0,
            (CausalEffectValue::Complex(l0), CausalEffectValue::Complex(r0)) => l0 == r0,
            (CausalEffectValue::ComplexTensor(l0), CausalEffectValue::ComplexTensor(r0)) => {
                l0 == r0
            }
            (CausalEffectValue::Quaternion(l0), CausalEffectValue::Quaternion(r0)) => l0 == r0,
            (CausalEffectValue::QuaternionTensor(l0), CausalEffectValue::QuaternionTensor(r0)) => {
                l0 == r0
            }
            (CausalEffectValue::UncertainBool(l0), CausalEffectValue::UncertainBool(r0)) => {
                l0 == r0
            }
            (CausalEffectValue::UncertainFloat(l0), CausalEffectValue::UncertainFloat(r0)) => {
                l0 == r0
            }
            (
                CausalEffectValue::MaybeUncertainBool(l0),
                CausalEffectValue::MaybeUncertainBool(r0),
            ) => l0 == r0,
            (
                CausalEffectValue::MaybeUncertainFloat(l0),
                CausalEffectValue::MaybeUncertainFloat(r0),
            ) => l0 == r0,
            (
                CausalEffectValue::ContextualLink(l0, l1),
                CausalEffectValue::ContextualLink(r0, r1),
            ) => l0 == r0 && l1 == r1,
            (CausalEffectValue::Map(l0), CausalEffectValue::Map(r0)) => l0 == r0,
            (CausalEffectValue::RelayTo(l0, l1), CausalEffectValue::RelayTo(r0, r1)) => {
                l0 == r0 && l1 == r1
            }
            _ => false,
        }
    }
}
