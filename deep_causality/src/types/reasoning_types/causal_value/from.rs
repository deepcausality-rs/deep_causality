/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{CausalValue, NumericValue};
use deep_causality_num::{Complex, Quaternion};
use deep_causality_tensor::CausalTensor;
use deep_causality_uncertain::{
    MaybeUncertainBool, MaybeUncertainF64, UncertainBool, UncertainF64,
};

impl From<bool> for CausalValue {
    fn from(b: bool) -> Self {
        CausalValue::Deterministic(b)
    }
}

impl From<NumericValue> for CausalValue {
    fn from(n: NumericValue) -> Self {
        CausalValue::Numerical(n)
    }
}

impl From<f64> for CausalValue {
    fn from(f: f64) -> Self {
        CausalValue::Probabilistic(f)
    }
}

impl From<CausalTensor<f64>> for CausalValue {
    fn from(t: CausalTensor<f64>) -> Self {
        CausalValue::Tensor(t)
    }
}

impl From<Complex<f64>> for CausalValue {
    fn from(c: Complex<f64>) -> Self {
        CausalValue::Complex(c)
    }
}

impl From<CausalTensor<Complex<f64>>> for CausalValue {
    fn from(t: CausalTensor<Complex<f64>>) -> Self {
        CausalValue::ComplexTensor(t)
    }
}

impl From<Quaternion<f64>> for CausalValue {
    fn from(q: Quaternion<f64>) -> Self {
        CausalValue::Quaternion(q)
    }
}

impl From<CausalTensor<Quaternion<f64>>> for CausalValue {
    fn from(t: CausalTensor<Quaternion<f64>>) -> Self {
        CausalValue::QuaternionTensor(t)
    }
}

impl From<UncertainBool> for CausalValue {
    fn from(ub: UncertainBool) -> Self {
        CausalValue::UncertainBool(ub)
    }
}

impl From<UncertainF64> for CausalValue {
    fn from(uf: UncertainF64) -> Self {
        CausalValue::UncertainFloat(uf)
    }
}

impl From<MaybeUncertainBool> for CausalValue {
    fn from(mub: MaybeUncertainBool) -> Self {
        CausalValue::MaybeUncertainBool(mub)
    }
}

impl From<MaybeUncertainF64> for CausalValue {
    fn from(muf: MaybeUncertainF64) -> Self {
        CausalValue::MaybeUncertainFloat(muf)
    }
}
