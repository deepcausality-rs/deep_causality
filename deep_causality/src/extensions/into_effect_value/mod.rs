/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::IntoEffectValue;
use crate::{CausalityError, EffectValue, NumericValue};
use deep_causality_num::{Complex, Quaternion};
use deep_causality_tensor::CausalTensor;
use deep_causality_uncertain::{
    MaybeUncertainBool, MaybeUncertainF64, UncertainBool, UncertainF64,
};

// Blanket implementation for types that are Debug, Clone, Default, and 'static.
// This ensures that any type meeting these basic requirements can be considered a PropagatingValue.
// impl<T: Debug + Clone + 'static> PropagatingValue for T {}

// Implementations for concrete types to be used as PropagatingEffect

impl IntoEffectValue for EffectValue {
    fn into_effect_value(self) -> EffectValue {
        self
    }

    fn try_from_effect_value(ev: EffectValue) -> Result<Self, CausalityError>
    where
        Self: Sized,
    {
        Ok(ev)
    }
}

// bool
impl IntoEffectValue for bool {
    fn into_effect_value(self) -> EffectValue {
        EffectValue::Deterministic(self)
    }

    fn try_from_effect_value(ev: EffectValue) -> Result<Self, CausalityError> {
        if let EffectValue::Deterministic(b) = ev {
            Ok(b)
        } else {
            Err(CausalityError(format!(
                "Expected Deterministic(bool), found {:?}",
                ev
            )))
        }
    }
}

// f64
impl IntoEffectValue for f64 {
    fn into_effect_value(self) -> EffectValue {
        EffectValue::Numerical(self)
    }

    fn try_from_effect_value(ev: EffectValue) -> Result<Self, CausalityError> {
        if let EffectValue::Numerical(f) = ev {
            Ok(f)
        } else {
            Err(CausalityError(format!(
                "Expected Numerical(f64), found {:?}",
                ev
            )))
        }
    }
}

impl IntoEffectValue for u64 {
    fn into_effect_value(self) -> EffectValue {
        EffectValue::Number(NumericValue::U64(self))
    }

    fn try_from_effect_value(ev: EffectValue) -> Result<Self, CausalityError>
    where
        Self: Sized,
    {
        if let EffectValue::Number(val) = ev {
            match val {
                NumericValue::U64(val) => Ok(val),
                _ => Err(CausalityError(format!(
                    "Expected U64(u64), found {:?}",
                    val
                ))),
            }
        } else {
            Err(CausalityError(format!("Expected U64(u64), found {:?}", ev)))
        }
    }
}

// NumericValue
impl IntoEffectValue for NumericValue {
    fn into_effect_value(self) -> EffectValue {
        EffectValue::Number(self)
    }

    fn try_from_effect_value(ev: EffectValue) -> Result<Self, CausalityError> {
        if let EffectValue::Number(n) = ev {
            Ok(n)
        } else {
            Err(CausalityError(format!(
                "Expected Number(NumericValue), found {:?}",
                ev
            )))
        }
    }
}

// CausalTensor<f64>
impl IntoEffectValue for CausalTensor<f64> {
    fn into_effect_value(self) -> EffectValue {
        EffectValue::Tensor(self)
    }

    fn try_from_effect_value(ev: EffectValue) -> Result<Self, CausalityError> {
        if let EffectValue::Tensor(t) = ev {
            Ok(t)
        } else {
            Err(CausalityError(format!(
                "Expected Tensor(CausalTensor<f64>), found {:?}",
                ev
            )))
        }
    }
}

// Complex<f64>
impl IntoEffectValue for Complex<f64> {
    fn into_effect_value(self) -> EffectValue {
        EffectValue::Complex(self)
    }

    fn try_from_effect_value(ev: EffectValue) -> Result<Self, CausalityError> {
        if let EffectValue::Complex(c) = ev {
            Ok(c)
        } else {
            Err(CausalityError(format!(
                "Expected Complex(Complex<f64>), found {:?}",
                ev
            )))
        }
    }
}

// CausalTensor<Complex<f64>>
impl IntoEffectValue for CausalTensor<Complex<f64>> {
    fn into_effect_value(self) -> EffectValue {
        EffectValue::ComplexTensor(self)
    }

    fn try_from_effect_value(ev: EffectValue) -> Result<Self, CausalityError> {
        if let EffectValue::ComplexTensor(ct) = ev {
            Ok(ct)
        } else {
            Err(CausalityError(format!(
                "Expected ComplexTensor(CausalTensor<Complex<f64>>), found {:?}",
                ev
            )))
        }
    }
}

// Quaternion<f64>
impl IntoEffectValue for Quaternion<f64> {
    fn into_effect_value(self) -> EffectValue {
        EffectValue::Quaternion(self)
    }

    fn try_from_effect_value(ev: EffectValue) -> Result<Self, CausalityError> {
        if let EffectValue::Quaternion(q) = ev {
            Ok(q)
        } else {
            Err(CausalityError(format!(
                "Expected Quaternion(Quaternion<f64>), found {:?}",
                ev
            )))
        }
    }
}

// CausalTensor<Quaternion<f64>>
impl IntoEffectValue for CausalTensor<Quaternion<f64>> {
    fn into_effect_value(self) -> EffectValue {
        EffectValue::QuaternionTensor(self)
    }

    fn try_from_effect_value(ev: EffectValue) -> Result<Self, CausalityError> {
        if let EffectValue::QuaternionTensor(qt) = ev {
            Ok(qt)
        } else {
            Err(CausalityError(format!(
                "Expected QuaternionTensor(CausalTensor<Quaternion<f64>>), found {:?}",
                ev
            )))
        }
    }
}

// UncertainBool
impl IntoEffectValue for UncertainBool {
    fn into_effect_value(self) -> EffectValue {
        EffectValue::UncertainBool(self)
    }

    fn try_from_effect_value(ev: EffectValue) -> Result<Self, CausalityError> {
        if let EffectValue::UncertainBool(ub) = ev {
            Ok(ub)
        } else {
            Err(CausalityError(format!(
                "Expected UncertainBool(UncertainBool), found {:?}",
                ev
            )))
        }
    }
}

// UncertainF64
impl IntoEffectValue for UncertainF64 {
    fn into_effect_value(self) -> EffectValue {
        EffectValue::UncertainFloat(self)
    }

    fn try_from_effect_value(ev: EffectValue) -> Result<Self, CausalityError> {
        if let EffectValue::UncertainFloat(uf) = ev {
            Ok(uf)
        } else {
            Err(CausalityError(format!(
                "Expected UncertainFloat(UncertainF64), found {:?}",
                ev
            )))
        }
    }
}

// MaybeUncertainBool
impl IntoEffectValue for MaybeUncertainBool {
    fn into_effect_value(self) -> EffectValue {
        EffectValue::MaybeUncertainBool(self)
    }

    fn try_from_effect_value(ev: EffectValue) -> Result<Self, CausalityError> {
        if let EffectValue::MaybeUncertainBool(mub) = ev {
            Ok(mub)
        } else {
            Err(CausalityError(format!(
                "Expected MaybeUncertainBool(MaybeUncertainBool), found {:?}",
                ev
            )))
        }
    }
}

// MaybeUncertainF64
impl IntoEffectValue for MaybeUncertainF64 {
    fn into_effect_value(self) -> EffectValue {
        EffectValue::MaybeUncertainFloat(self)
    }

    fn try_from_effect_value(ev: EffectValue) -> Result<Self, CausalityError> {
        if let EffectValue::MaybeUncertainFloat(muf) = ev {
            Ok(muf)
        } else {
            Err(CausalityError(format!(
                "Expected MaybeUncertainFloat(MaybeUncertainF64), found {:?}",
                ev
            )))
        }
    }
}

// Unit type for no effect
impl IntoEffectValue for () {
    fn into_effect_value(self) -> EffectValue {
        EffectValue::None
    }

    fn try_from_effect_value(ev: EffectValue) -> Result<Self, CausalityError> {
        if let EffectValue::None = ev {
            Ok(())
        } else {
            Err(CausalityError(format!("Expected None, found {:?}", ev)))
        }
    }
}
