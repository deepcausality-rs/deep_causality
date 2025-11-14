/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{BaseCausaloid, ContextoidId, NumericValue, RegisteredCausaloid};
use deep_causality_num::{Complex, Quaternion};
use deep_causality_tensor::CausalTensor;
use deep_causality_uncertain::{UncertainBool, UncertainF64};

// Helper `From` implementations to make registration ergonomic.
impl From<BaseCausaloid<bool, bool>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<bool, bool>) -> Self {
        RegisteredCausaloid::DeterministicToDeterministic(c)
    }
}

impl From<BaseCausaloid<bool, NumericValue>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<bool, NumericValue>) -> Self {
        RegisteredCausaloid::DeterministicToNumber(c)
    }
}

impl From<BaseCausaloid<bool, f64>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<bool, f64>) -> Self {
        RegisteredCausaloid::DeterministicToNumerical(c)
    }
}

impl From<BaseCausaloid<bool, CausalTensor<f64>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<bool, CausalTensor<f64>>) -> Self {
        RegisteredCausaloid::DeterministicToTensor(c)
    }
}

impl From<BaseCausaloid<bool, Complex<f64>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<bool, Complex<f64>>) -> Self {
        RegisteredCausaloid::DeterministicToComplex(c)
    }
}

impl From<BaseCausaloid<bool, CausalTensor<Complex<f64>>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<bool, CausalTensor<Complex<f64>>>) -> Self {
        RegisteredCausaloid::DeterministicToComplexTensor(c)
    }
}

impl From<BaseCausaloid<bool, Quaternion<f64>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<bool, Quaternion<f64>>) -> Self {
        RegisteredCausaloid::DeterministicToQuaternion(c)
    }
}

impl From<BaseCausaloid<bool, CausalTensor<Quaternion<f64>>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<bool, CausalTensor<Quaternion<f64>>>) -> Self {
        RegisteredCausaloid::DeterministicToQuaternionTensor(c)
    }
}

impl From<BaseCausaloid<bool, UncertainBool>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<bool, UncertainBool>) -> Self {
        RegisteredCausaloid::DeterministicToUncertainBool(c)
    }
}

impl From<BaseCausaloid<bool, UncertainF64>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<bool, UncertainF64>) -> Self {
        RegisteredCausaloid::DeterministicToUncertainFloat(c)
    }
}

impl From<BaseCausaloid<bool, ContextoidId>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<bool, ContextoidId>) -> Self {
        RegisteredCausaloid::DeterministicToContextualLink(c)
    }
}

impl From<BaseCausaloid<NumericValue, bool>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<NumericValue, bool>) -> Self {
        RegisteredCausaloid::NumberToDeterministic(c)
    }
}

impl From<BaseCausaloid<NumericValue, NumericValue>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<NumericValue, NumericValue>) -> Self {
        RegisteredCausaloid::NumberToNumber(c)
    }
}

impl From<BaseCausaloid<NumericValue, f64>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<NumericValue, f64>) -> Self {
        RegisteredCausaloid::NumberToNumerical(c)
    }
}

impl From<BaseCausaloid<NumericValue, CausalTensor<f64>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<NumericValue, CausalTensor<f64>>) -> Self {
        RegisteredCausaloid::NumberToTensor(c)
    }
}

impl From<BaseCausaloid<NumericValue, Complex<f64>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<NumericValue, Complex<f64>>) -> Self {
        RegisteredCausaloid::NumberToComplex(c)
    }
}

impl From<BaseCausaloid<NumericValue, CausalTensor<Complex<f64>>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<NumericValue, CausalTensor<Complex<f64>>>) -> Self {
        RegisteredCausaloid::NumberToComplexTensor(c)
    }
}

impl From<BaseCausaloid<NumericValue, Quaternion<f64>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<NumericValue, Quaternion<f64>>) -> Self {
        RegisteredCausaloid::NumberToQuaternion(c)
    }
}

impl From<BaseCausaloid<NumericValue, CausalTensor<Quaternion<f64>>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<NumericValue, CausalTensor<Quaternion<f64>>>) -> Self {
        RegisteredCausaloid::NumberToQuaternionTensor(c)
    }
}

impl From<BaseCausaloid<NumericValue, UncertainBool>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<NumericValue, UncertainBool>) -> Self {
        RegisteredCausaloid::NumberToUncertainBool(c)
    }
}

impl From<BaseCausaloid<NumericValue, UncertainF64>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<NumericValue, UncertainF64>) -> Self {
        RegisteredCausaloid::NumberToUncertainFloat(c)
    }
}

impl From<BaseCausaloid<NumericValue, ContextoidId>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<NumericValue, ContextoidId>) -> Self {
        RegisteredCausaloid::NumberToContextualLink(c)
    }
}

impl From<BaseCausaloid<f64, bool>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<f64, bool>) -> Self {
        RegisteredCausaloid::NumericalToDeterministic(c)
    }
}

impl From<BaseCausaloid<f64, NumericValue>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<f64, NumericValue>) -> Self {
        RegisteredCausaloid::NumericalToNumber(c)
    }
}

impl From<BaseCausaloid<f64, f64>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<f64, f64>) -> Self {
        RegisteredCausaloid::NumericalToNumerical(c)
    }
}

impl From<BaseCausaloid<f64, CausalTensor<f64>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<f64, CausalTensor<f64>>) -> Self {
        RegisteredCausaloid::NumericalToTensor(c)
    }
}

impl From<BaseCausaloid<f64, Complex<f64>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<f64, Complex<f64>>) -> Self {
        RegisteredCausaloid::NumericalToComplex(c)
    }
}

impl From<BaseCausaloid<f64, CausalTensor<Complex<f64>>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<f64, CausalTensor<Complex<f64>>>) -> Self {
        RegisteredCausaloid::NumericalToComplexTensor(c)
    }
}

impl From<BaseCausaloid<f64, Quaternion<f64>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<f64, Quaternion<f64>>) -> Self {
        RegisteredCausaloid::NumericalToQuaternion(c)
    }
}

impl From<BaseCausaloid<f64, CausalTensor<Quaternion<f64>>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<f64, CausalTensor<Quaternion<f64>>>) -> Self {
        RegisteredCausaloid::NumericalToQuaternionTensor(c)
    }
}

impl From<BaseCausaloid<f64, UncertainBool>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<f64, UncertainBool>) -> Self {
        RegisteredCausaloid::NumericalToUncertainBool(c)
    }
}

impl From<BaseCausaloid<f64, UncertainF64>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<f64, UncertainF64>) -> Self {
        RegisteredCausaloid::NumericalToUncertainFloat(c)
    }
}

impl From<BaseCausaloid<f64, ContextoidId>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<f64, ContextoidId>) -> Self {
        RegisteredCausaloid::NumericalToContextualLink(c)
    }
}

impl From<BaseCausaloid<CausalTensor<f64>, bool>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<CausalTensor<f64>, bool>) -> Self {
        RegisteredCausaloid::TensorToDeterministic(c)
    }
}

impl From<BaseCausaloid<CausalTensor<f64>, NumericValue>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<CausalTensor<f64>, NumericValue>) -> Self {
        RegisteredCausaloid::TensorToNumber(c)
    }
}

impl From<BaseCausaloid<CausalTensor<f64>, f64>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<CausalTensor<f64>, f64>) -> Self {
        RegisteredCausaloid::TensorToNumerical(c)
    }
}

impl From<BaseCausaloid<CausalTensor<f64>, CausalTensor<f64>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<CausalTensor<f64>, CausalTensor<f64>>) -> Self {
        RegisteredCausaloid::TensorToTensor(c)
    }
}

impl From<BaseCausaloid<CausalTensor<f64>, Complex<f64>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<CausalTensor<f64>, Complex<f64>>) -> Self {
        RegisteredCausaloid::TensorToComplex(c)
    }
}

impl From<BaseCausaloid<CausalTensor<f64>, CausalTensor<Complex<f64>>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<CausalTensor<f64>, CausalTensor<Complex<f64>>>) -> Self {
        RegisteredCausaloid::TensorToComplexTensor(c)
    }
}

impl From<BaseCausaloid<CausalTensor<f64>, Quaternion<f64>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<CausalTensor<f64>, Quaternion<f64>>) -> Self {
        RegisteredCausaloid::TensorToQuaternion(c)
    }
}

impl From<BaseCausaloid<CausalTensor<f64>, CausalTensor<Quaternion<f64>>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<CausalTensor<f64>, CausalTensor<Quaternion<f64>>>) -> Self {
        RegisteredCausaloid::TensorToQuaternionTensor(c)
    }
}

impl From<BaseCausaloid<CausalTensor<f64>, UncertainBool>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<CausalTensor<f64>, UncertainBool>) -> Self {
        RegisteredCausaloid::TensorToUncertainBool(c)
    }
}

impl From<BaseCausaloid<CausalTensor<f64>, UncertainF64>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<CausalTensor<f64>, UncertainF64>) -> Self {
        RegisteredCausaloid::TensorToUncertainFloat(c)
    }
}

impl From<BaseCausaloid<CausalTensor<f64>, ContextoidId>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<CausalTensor<f64>, ContextoidId>) -> Self {
        RegisteredCausaloid::TensorToContextualLink(c)
    }
}

impl From<BaseCausaloid<Complex<f64>, bool>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<Complex<f64>, bool>) -> Self {
        RegisteredCausaloid::ComplexToDeterministic(c)
    }
}

impl From<BaseCausaloid<Complex<f64>, NumericValue>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<Complex<f64>, NumericValue>) -> Self {
        RegisteredCausaloid::ComplexToNumber(c)
    }
}

impl From<BaseCausaloid<Complex<f64>, f64>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<Complex<f64>, f64>) -> Self {
        RegisteredCausaloid::ComplexToNumerical(c)
    }
}

impl From<BaseCausaloid<Complex<f64>, CausalTensor<f64>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<Complex<f64>, CausalTensor<f64>>) -> Self {
        RegisteredCausaloid::ComplexToTensor(c)
    }
}

impl From<BaseCausaloid<Complex<f64>, Complex<f64>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<Complex<f64>, Complex<f64>>) -> Self {
        RegisteredCausaloid::ComplexToComplex(c)
    }
}

impl From<BaseCausaloid<Complex<f64>, CausalTensor<Complex<f64>>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<Complex<f64>, CausalTensor<Complex<f64>>>) -> Self {
        RegisteredCausaloid::ComplexToComplexTensor(c)
    }
}

impl From<BaseCausaloid<Complex<f64>, Quaternion<f64>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<Complex<f64>, Quaternion<f64>>) -> Self {
        RegisteredCausaloid::ComplexToQuaternion(c)
    }
}

impl From<BaseCausaloid<Complex<f64>, CausalTensor<Quaternion<f64>>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<Complex<f64>, CausalTensor<Quaternion<f64>>>) -> Self {
        RegisteredCausaloid::ComplexToQuaternionTensor(c)
    }
}

impl From<BaseCausaloid<Complex<f64>, UncertainBool>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<Complex<f64>, UncertainBool>) -> Self {
        RegisteredCausaloid::ComplexToUncertainBool(c)
    }
}

impl From<BaseCausaloid<Complex<f64>, UncertainF64>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<Complex<f64>, UncertainF64>) -> Self {
        RegisteredCausaloid::ComplexToUncertainFloat(c)
    }
}

impl From<BaseCausaloid<Complex<f64>, ContextoidId>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<Complex<f64>, ContextoidId>) -> Self {
        RegisteredCausaloid::ComplexToContextualLink(c)
    }
}

impl From<BaseCausaloid<CausalTensor<Complex<f64>>, bool>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<CausalTensor<Complex<f64>>, bool>) -> Self {
        RegisteredCausaloid::ComplexTensorToDeterministic(c)
    }
}

impl From<BaseCausaloid<CausalTensor<Complex<f64>>, NumericValue>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<CausalTensor<Complex<f64>>, NumericValue>) -> Self {
        RegisteredCausaloid::ComplexTensorToNumber(c)
    }
}

impl From<BaseCausaloid<CausalTensor<Complex<f64>>, f64>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<CausalTensor<Complex<f64>>, f64>) -> Self {
        RegisteredCausaloid::ComplexTensorToNumerical(c)
    }
}

impl From<BaseCausaloid<CausalTensor<Complex<f64>>, CausalTensor<f64>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<CausalTensor<Complex<f64>>, CausalTensor<f64>>) -> Self {
        RegisteredCausaloid::ComplexTensorToTensor(c)
    }
}

impl From<BaseCausaloid<CausalTensor<Complex<f64>>, Complex<f64>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<CausalTensor<Complex<f64>>, Complex<f64>>) -> Self {
        RegisteredCausaloid::ComplexTensorToComplex(c)
    }
}

impl From<BaseCausaloid<CausalTensor<Complex<f64>>, CausalTensor<Complex<f64>>>>
    for RegisteredCausaloid
{
    fn from(c: BaseCausaloid<CausalTensor<Complex<f64>>, CausalTensor<Complex<f64>>>) -> Self {
        RegisteredCausaloid::ComplexTensorToComplexTensor(c)
    }
}

impl From<BaseCausaloid<CausalTensor<Complex<f64>>, Quaternion<f64>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<CausalTensor<Complex<f64>>, Quaternion<f64>>) -> Self {
        RegisteredCausaloid::ComplexTensorToQuaternion(c)
    }
}

impl From<BaseCausaloid<CausalTensor<Complex<f64>>, CausalTensor<Quaternion<f64>>>>
    for RegisteredCausaloid
{
    fn from(c: BaseCausaloid<CausalTensor<Complex<f64>>, CausalTensor<Quaternion<f64>>>) -> Self {
        RegisteredCausaloid::ComplexTensorToQuaternionTensor(c)
    }
}

impl From<BaseCausaloid<CausalTensor<Complex<f64>>, UncertainBool>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<CausalTensor<Complex<f64>>, UncertainBool>) -> Self {
        RegisteredCausaloid::ComplexTensorToUncertainBool(c)
    }
}

impl From<BaseCausaloid<CausalTensor<Complex<f64>>, UncertainF64>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<CausalTensor<Complex<f64>>, UncertainF64>) -> Self {
        RegisteredCausaloid::ComplexTensorToUncertainFloat(c)
    }
}

impl From<BaseCausaloid<CausalTensor<Complex<f64>>, ContextoidId>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<CausalTensor<Complex<f64>>, ContextoidId>) -> Self {
        RegisteredCausaloid::ComplexTensorToContextualLink(c)
    }
}

impl From<BaseCausaloid<Quaternion<f64>, bool>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<Quaternion<f64>, bool>) -> Self {
        RegisteredCausaloid::QuaternionToDeterministic(c)
    }
}

impl From<BaseCausaloid<Quaternion<f64>, NumericValue>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<Quaternion<f64>, NumericValue>) -> Self {
        RegisteredCausaloid::QuaternionToNumber(c)
    }
}

impl From<BaseCausaloid<Quaternion<f64>, f64>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<Quaternion<f64>, f64>) -> Self {
        RegisteredCausaloid::QuaternionToNumerical(c)
    }
}

impl From<BaseCausaloid<Quaternion<f64>, CausalTensor<f64>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<Quaternion<f64>, CausalTensor<f64>>) -> Self {
        RegisteredCausaloid::QuaternionToTensor(c)
    }
}

impl From<BaseCausaloid<Quaternion<f64>, Complex<f64>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<Quaternion<f64>, Complex<f64>>) -> Self {
        RegisteredCausaloid::QuaternionToComplex(c)
    }
}

impl From<BaseCausaloid<Quaternion<f64>, CausalTensor<Complex<f64>>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<Quaternion<f64>, CausalTensor<Complex<f64>>>) -> Self {
        RegisteredCausaloid::QuaternionToComplexTensor(c)
    }
}

impl From<BaseCausaloid<Quaternion<f64>, Quaternion<f64>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<Quaternion<f64>, Quaternion<f64>>) -> Self {
        RegisteredCausaloid::QuaternionToQuaternion(c)
    }
}

impl From<BaseCausaloid<Quaternion<f64>, CausalTensor<Quaternion<f64>>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<Quaternion<f64>, CausalTensor<Quaternion<f64>>>) -> Self {
        RegisteredCausaloid::QuaternionToQuaternionTensor(c)
    }
}

impl From<BaseCausaloid<Quaternion<f64>, UncertainBool>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<Quaternion<f64>, UncertainBool>) -> Self {
        RegisteredCausaloid::QuaternionToUncertainBool(c)
    }
}

impl From<BaseCausaloid<Quaternion<f64>, UncertainF64>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<Quaternion<f64>, UncertainF64>) -> Self {
        RegisteredCausaloid::QuaternionToUncertainFloat(c)
    }
}

impl From<BaseCausaloid<Quaternion<f64>, ContextoidId>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<Quaternion<f64>, ContextoidId>) -> Self {
        RegisteredCausaloid::QuaternionToContextualLink(c)
    }
}

impl From<BaseCausaloid<CausalTensor<Quaternion<f64>>, bool>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<CausalTensor<Quaternion<f64>>, bool>) -> Self {
        RegisteredCausaloid::QuaternionTensorToDeterministic(c)
    }
}

impl From<BaseCausaloid<CausalTensor<Quaternion<f64>>, NumericValue>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<CausalTensor<Quaternion<f64>>, NumericValue>) -> Self {
        RegisteredCausaloid::QuaternionTensorToNumber(c)
    }
}

impl From<BaseCausaloid<CausalTensor<Quaternion<f64>>, f64>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<CausalTensor<Quaternion<f64>>, f64>) -> Self {
        RegisteredCausaloid::QuaternionTensorToNumerical(c)
    }
}

impl From<BaseCausaloid<CausalTensor<Quaternion<f64>>, CausalTensor<f64>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<CausalTensor<Quaternion<f64>>, CausalTensor<f64>>) -> Self {
        RegisteredCausaloid::QuaternionTensorToTensor(c)
    }
}

impl From<BaseCausaloid<CausalTensor<Quaternion<f64>>, Complex<f64>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<CausalTensor<Quaternion<f64>>, Complex<f64>>) -> Self {
        RegisteredCausaloid::QuaternionTensorToComplex(c)
    }
}

impl From<BaseCausaloid<CausalTensor<Quaternion<f64>>, CausalTensor<Complex<f64>>>>
    for RegisteredCausaloid
{
    fn from(c: BaseCausaloid<CausalTensor<Quaternion<f64>>, CausalTensor<Complex<f64>>>) -> Self {
        RegisteredCausaloid::QuaternionTensorToComplexTensor(c)
    }
}

impl From<BaseCausaloid<CausalTensor<Quaternion<f64>>, Quaternion<f64>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<CausalTensor<Quaternion<f64>>, Quaternion<f64>>) -> Self {
        RegisteredCausaloid::QuaternionTensorToQuaternion(c)
    }
}

impl From<BaseCausaloid<CausalTensor<Quaternion<f64>>, CausalTensor<Quaternion<f64>>>>
    for RegisteredCausaloid
{
    fn from(
        c: BaseCausaloid<CausalTensor<Quaternion<f64>>, CausalTensor<Quaternion<f64>>>,
    ) -> Self {
        RegisteredCausaloid::QuaternionTensorToQuaternionTensor(c)
    }
}

impl From<BaseCausaloid<CausalTensor<Quaternion<f64>>, UncertainBool>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<CausalTensor<Quaternion<f64>>, UncertainBool>) -> Self {
        RegisteredCausaloid::QuaternionTensorToUncertainBool(c)
    }
}

impl From<BaseCausaloid<CausalTensor<Quaternion<f64>>, UncertainF64>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<CausalTensor<Quaternion<f64>>, UncertainF64>) -> Self {
        RegisteredCausaloid::QuaternionTensorToUncertainFloat(c)
    }
}

impl From<BaseCausaloid<CausalTensor<Quaternion<f64>>, ContextoidId>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<CausalTensor<Quaternion<f64>>, ContextoidId>) -> Self {
        RegisteredCausaloid::QuaternionTensorToContextualLink(c)
    }
}

impl From<BaseCausaloid<UncertainBool, bool>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<UncertainBool, bool>) -> Self {
        RegisteredCausaloid::UncertainBoolToDeterministic(c)
    }
}

impl From<BaseCausaloid<UncertainBool, NumericValue>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<UncertainBool, NumericValue>) -> Self {
        RegisteredCausaloid::UncertainBoolToNumber(c)
    }
}

impl From<BaseCausaloid<UncertainBool, f64>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<UncertainBool, f64>) -> Self {
        RegisteredCausaloid::UncertainBoolToNumerical(c)
    }
}

impl From<BaseCausaloid<UncertainBool, CausalTensor<f64>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<UncertainBool, CausalTensor<f64>>) -> Self {
        RegisteredCausaloid::UncertainBoolToTensor(c)
    }
}

impl From<BaseCausaloid<UncertainBool, Complex<f64>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<UncertainBool, Complex<f64>>) -> Self {
        RegisteredCausaloid::UncertainBoolToComplex(c)
    }
}

impl From<BaseCausaloid<UncertainBool, CausalTensor<Complex<f64>>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<UncertainBool, CausalTensor<Complex<f64>>>) -> Self {
        RegisteredCausaloid::UncertainBoolToComplexTensor(c)
    }
}

impl From<BaseCausaloid<UncertainBool, Quaternion<f64>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<UncertainBool, Quaternion<f64>>) -> Self {
        RegisteredCausaloid::UncertainBoolToQuaternion(c)
    }
}

impl From<BaseCausaloid<UncertainBool, CausalTensor<Quaternion<f64>>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<UncertainBool, CausalTensor<Quaternion<f64>>>) -> Self {
        RegisteredCausaloid::UncertainBoolToQuaternionTensor(c)
    }
}

impl From<BaseCausaloid<UncertainBool, UncertainBool>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<UncertainBool, UncertainBool>) -> Self {
        RegisteredCausaloid::UncertainBoolToUncertainBool(c)
    }
}

impl From<BaseCausaloid<UncertainBool, UncertainF64>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<UncertainBool, UncertainF64>) -> Self {
        RegisteredCausaloid::UncertainBoolToUncertainFloat(c)
    }
}

impl From<BaseCausaloid<UncertainBool, ContextoidId>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<UncertainBool, ContextoidId>) -> Self {
        RegisteredCausaloid::UncertainBoolToContextualLink(c)
    }
}

impl From<BaseCausaloid<UncertainF64, bool>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<UncertainF64, bool>) -> Self {
        RegisteredCausaloid::UncertainFloatToDeterministic(c)
    }
}

impl From<BaseCausaloid<UncertainF64, NumericValue>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<UncertainF64, NumericValue>) -> Self {
        RegisteredCausaloid::UncertainFloatToNumber(c)
    }
}

impl From<BaseCausaloid<UncertainF64, f64>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<UncertainF64, f64>) -> Self {
        RegisteredCausaloid::UncertainFloatToNumerical(c)
    }
}

impl From<BaseCausaloid<UncertainF64, CausalTensor<f64>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<UncertainF64, CausalTensor<f64>>) -> Self {
        RegisteredCausaloid::UncertainFloatToTensor(c)
    }
}

impl From<BaseCausaloid<UncertainF64, Complex<f64>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<UncertainF64, Complex<f64>>) -> Self {
        RegisteredCausaloid::UncertainFloatToComplex(c)
    }
}

impl From<BaseCausaloid<UncertainF64, CausalTensor<Complex<f64>>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<UncertainF64, CausalTensor<Complex<f64>>>) -> Self {
        RegisteredCausaloid::UncertainFloatToComplexTensor(c)
    }
}

impl From<BaseCausaloid<UncertainF64, Quaternion<f64>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<UncertainF64, Quaternion<f64>>) -> Self {
        RegisteredCausaloid::UncertainFloatToQuaternion(c)
    }
}

impl From<BaseCausaloid<UncertainF64, CausalTensor<Quaternion<f64>>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<UncertainF64, CausalTensor<Quaternion<f64>>>) -> Self {
        RegisteredCausaloid::UncertainFloatToQuaternionTensor(c)
    }
}

impl From<BaseCausaloid<UncertainF64, UncertainBool>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<UncertainF64, UncertainBool>) -> Self {
        RegisteredCausaloid::UncertainFloatToUncertainBool(c)
    }
}

impl From<BaseCausaloid<UncertainF64, UncertainF64>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<UncertainF64, UncertainF64>) -> Self {
        RegisteredCausaloid::UncertainFloatToUncertainFloat(c)
    }
}

impl From<BaseCausaloid<UncertainF64, ContextoidId>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<UncertainF64, ContextoidId>) -> Self {
        RegisteredCausaloid::UncertainFloatToContextualLink(c)
    }
}

impl From<BaseCausaloid<ContextoidId, bool>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<ContextoidId, bool>) -> Self {
        RegisteredCausaloid::ContextualLinkToDeterministic(c)
    }
}

impl From<BaseCausaloid<ContextoidId, NumericValue>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<ContextoidId, NumericValue>) -> Self {
        RegisteredCausaloid::ContextualLinkToNumber(c)
    }
}

impl From<BaseCausaloid<ContextoidId, f64>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<ContextoidId, f64>) -> Self {
        RegisteredCausaloid::ContextualLinkToNumerical(c)
    }
}

impl From<BaseCausaloid<ContextoidId, CausalTensor<f64>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<ContextoidId, CausalTensor<f64>>) -> Self {
        RegisteredCausaloid::ContextualLinkToTensor(c)
    }
}

impl From<BaseCausaloid<ContextoidId, Complex<f64>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<ContextoidId, Complex<f64>>) -> Self {
        RegisteredCausaloid::ContextualLinkToComplex(c)
    }
}

impl From<BaseCausaloid<ContextoidId, CausalTensor<Complex<f64>>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<ContextoidId, CausalTensor<Complex<f64>>>) -> Self {
        RegisteredCausaloid::ContextualLinkToComplexTensor(c)
    }
}

impl From<BaseCausaloid<ContextoidId, Quaternion<f64>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<ContextoidId, Quaternion<f64>>) -> Self {
        RegisteredCausaloid::ContextualLinkToQuaternion(c)
    }
}

impl From<BaseCausaloid<ContextoidId, CausalTensor<Quaternion<f64>>>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<ContextoidId, CausalTensor<Quaternion<f64>>>) -> Self {
        RegisteredCausaloid::ContextualLinkToQuaternionTensor(c)
    }
}

impl From<BaseCausaloid<ContextoidId, UncertainBool>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<ContextoidId, UncertainBool>) -> Self {
        RegisteredCausaloid::ContextualLinkToUncertainBool(c)
    }
}

impl From<BaseCausaloid<ContextoidId, UncertainF64>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<ContextoidId, UncertainF64>) -> Self {
        RegisteredCausaloid::ContextualLinkToUncertainFloat(c)
    }
}

impl From<BaseCausaloid<ContextoidId, ContextoidId>> for RegisteredCausaloid {
    fn from(c: BaseCausaloid<ContextoidId, ContextoidId>) -> Self {
        RegisteredCausaloid::ContextualLinkToContextualLink(c)
    }
}
