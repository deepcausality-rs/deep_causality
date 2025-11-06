/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{CausalEffectValue, NumericValue};
use deep_causality_num::{Complex, Quaternion};
use deep_causality_tensor::CausalTensor;
use deep_causality_uncertain::{
    MaybeUncertainBool, MaybeUncertainF64, UncertainBool, UncertainF64,
};

impl From<bool> for CausalEffectValue {
    fn from(b: bool) -> Self {
        CausalEffectValue::Deterministic(b)
    }
}

impl From<NumericValue> for CausalEffectValue {
    fn from(n: NumericValue) -> Self {
        CausalEffectValue::Numeric(n)
    }
}

impl From<f64> for CausalEffectValue {
    fn from(f: f64) -> Self {
        CausalEffectValue::Probabilistic(f)
    }
}

impl From<CausalTensor<f64>> for CausalEffectValue {
    fn from(t: CausalTensor<f64>) -> Self {
        CausalEffectValue::Tensor(t)
    }
}

impl From<Complex<f64>> for CausalEffectValue {
    fn from(c: Complex<f64>) -> Self {
        CausalEffectValue::Complex(c)
    }
}

impl From<CausalTensor<Complex<f64>>> for CausalEffectValue {
    fn from(t: CausalTensor<Complex<f64>>) -> Self {
        CausalEffectValue::ComplexTensor(t)
    }
}

impl From<Quaternion<f64>> for CausalEffectValue {
    fn from(q: Quaternion<f64>) -> Self {
        CausalEffectValue::Quaternion(q)
    }
}

impl From<CausalTensor<Quaternion<f64>>> for CausalEffectValue {
    fn from(t: CausalTensor<Quaternion<f64>>) -> Self {
        CausalEffectValue::QuaternionTensor(t)
    }
}

impl From<UncertainBool> for CausalEffectValue {
    fn from(ub: UncertainBool) -> Self {
        CausalEffectValue::UncertainBool(ub)
    }
}

impl From<UncertainF64> for CausalEffectValue {
    fn from(uf: UncertainF64) -> Self {
        CausalEffectValue::UncertainFloat(uf)
    }
}

impl From<MaybeUncertainBool> for CausalEffectValue {
    fn from(mub: MaybeUncertainBool) -> Self {
        CausalEffectValue::MaybeUncertainBool(mub)
    }
}

impl From<MaybeUncertainF64> for CausalEffectValue {
    fn from(muf: MaybeUncertainF64) -> Self {
        CausalEffectValue::MaybeUncertainFloat(muf)
    }
}
