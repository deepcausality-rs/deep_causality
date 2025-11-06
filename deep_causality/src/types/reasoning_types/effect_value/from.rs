/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{EffectValue, NumericValue, NumericalValue};
use deep_causality_num::{Complex, Quaternion};
use deep_causality_tensor::CausalTensor;
use deep_causality_uncertain::{
    MaybeUncertainBool, MaybeUncertainF64, UncertainBool, UncertainF64,
};

impl From<bool> for EffectValue {
    fn from(b: bool) -> Self {
        EffectValue::Deterministic(b)
    }
}

impl From<NumericValue> for EffectValue {
    fn from(n: NumericValue) -> Self {
        EffectValue::Number(n)
    }
}

impl From<NumericalValue> for EffectValue {
    fn from(value: NumericalValue) -> Self {
        EffectValue::Numerical(value)
    }
}

impl From<CausalTensor<f64>> for EffectValue {
    fn from(t: CausalTensor<f64>) -> Self {
        EffectValue::Tensor(t)
    }
}

impl From<Complex<f64>> for EffectValue {
    fn from(c: Complex<f64>) -> Self {
        EffectValue::Complex(c)
    }
}

impl From<CausalTensor<Complex<f64>>> for EffectValue {
    fn from(t: CausalTensor<Complex<f64>>) -> Self {
        EffectValue::ComplexTensor(t)
    }
}

impl From<Quaternion<f64>> for EffectValue {
    fn from(q: Quaternion<f64>) -> Self {
        EffectValue::Quaternion(q)
    }
}

impl From<CausalTensor<Quaternion<f64>>> for EffectValue {
    fn from(t: CausalTensor<Quaternion<f64>>) -> Self {
        EffectValue::QuaternionTensor(t)
    }
}

impl From<UncertainBool> for EffectValue {
    fn from(ub: UncertainBool) -> Self {
        EffectValue::UncertainBool(ub)
    }
}

impl From<UncertainF64> for EffectValue {
    fn from(uf: UncertainF64) -> Self {
        EffectValue::UncertainFloat(uf)
    }
}

impl From<MaybeUncertainBool> for EffectValue {
    fn from(mub: MaybeUncertainBool) -> Self {
        EffectValue::MaybeUncertainBool(mub)
    }
}

impl From<MaybeUncertainF64> for EffectValue {
    fn from(muf: MaybeUncertainF64) -> Self {
        EffectValue::MaybeUncertainFloat(muf)
    }
}
