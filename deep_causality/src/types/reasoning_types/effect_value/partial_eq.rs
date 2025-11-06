/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::EffectValue;

impl PartialEq for EffectValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (EffectValue::None, EffectValue::None) => true,
            (EffectValue::Deterministic(l0), EffectValue::Deterministic(r0)) => {
                l0 == r0
            }
            (EffectValue::Numeric(l0), EffectValue::Numeric(r0)) => l0 == r0,
            (EffectValue::Probabilistic(l0), EffectValue::Probabilistic(r0)) => {
                l0 == r0
            }
            (EffectValue::Tensor(l0), EffectValue::Tensor(r0)) => l0 == r0,
            (EffectValue::Complex(l0), EffectValue::Complex(r0)) => l0 == r0,
            (EffectValue::ComplexTensor(l0), EffectValue::ComplexTensor(r0)) => {
                l0 == r0
            }
            (EffectValue::Quaternion(l0), EffectValue::Quaternion(r0)) => l0 == r0,
            (EffectValue::QuaternionTensor(l0), EffectValue::QuaternionTensor(r0)) => {
                l0 == r0
            }
            (EffectValue::UncertainBool(l0), EffectValue::UncertainBool(r0)) => {
                l0 == r0
            }
            (EffectValue::UncertainFloat(l0), EffectValue::UncertainFloat(r0)) => {
                l0 == r0
            }
            (
                EffectValue::MaybeUncertainBool(l0),
                EffectValue::MaybeUncertainBool(r0),
            ) => l0 == r0,
            (
                EffectValue::MaybeUncertainFloat(l0),
                EffectValue::MaybeUncertainFloat(r0),
            ) => l0 == r0,
            (
                EffectValue::ContextualLink(l0, l1),
                EffectValue::ContextualLink(r0, r1),
            ) => l0 == r0 && l1 == r1,
            (EffectValue::Map(l0), EffectValue::Map(r0)) => l0 == r0,
            (EffectValue::RelayTo(l0, l1), EffectValue::RelayTo(r0, r1)) => {
                l0 == r0 && l1 == r1
            }
            _ => false,
        }
    }
}
