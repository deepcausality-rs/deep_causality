/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{EffectValue, IntoEffectValue, NumericValue};
use deep_causality_num::{Complex, Quaternion};
use deep_causality_tensor::CausalTensor;
use deep_causality_uncertain::{
    MaybeUncertain, MaybeUncertainBool, MaybeUncertainF64, Uncertain, UncertainBool, UncertainF64,
};

// Test for bool
#[test]
fn test_bool_into_effect_value() {
    let b = true;
    let ev: EffectValue = b.into_effect_value();
    assert_eq!(ev, EffectValue::Boolean(true));
}

#[test]
fn test_bool_try_from_effect_value_success() {
    let ev = EffectValue::Boolean(false);
    let b: bool = <bool as IntoEffectValue>::try_from_effect_value(ev).unwrap();
    assert!(!b);
}

#[test]
fn test_bool_try_from_effect_value_error() {
    let ev = EffectValue::Numerical(1.0);
    let err = <bool as IntoEffectValue>::try_from_effect_value(ev).unwrap_err();
    assert!(
        err.to_string()
            .contains("Expected Deterministic(bool), found Numerical(1.0)")
    );
}

#[test]
fn test_bool_dyn_eq() {
    let a = true;
    let av: EffectValue = a.into_effect_value();

    let b = true;
    let bv: EffectValue = b.into_effect_value();

    assert_eq!(av, bv);

    let c = false;
    let cv: EffectValue = c.into_effect_value();
    assert_ne!(av, cv);
    assert_ne!(bv, cv);
}

// Test for f64
#[test]
fn test_f64_into_effect_value() {
    let f = 1.23;
    let ev: EffectValue = f.into_effect_value();
    assert_eq!(ev, EffectValue::Numerical(1.23));
}

#[test]
fn test_f64_try_from_effect_value_success() {
    let ev = EffectValue::Numerical(4.56);
    let f: f64 = IntoEffectValue::try_from_effect_value(ev).unwrap();
    assert_eq!(f, 4.56);
}

#[test]
fn test_f64_try_from_effect_value_error() {
    let ev = EffectValue::Boolean(true);
    let err = <f64 as IntoEffectValue>::try_from_effect_value(ev).unwrap_err();
    assert!(
        err.to_string()
            .contains("Expected Numerical(f64), found Deterministic(true)")
    );
}

// Test for u64
#[test]
fn test_u64_into_effect_value() {
    let u = 123u64;
    let ev: EffectValue = u.into_effect_value();
    assert_eq!(ev, EffectValue::Number(NumericValue::U64(123)));
}

#[test]
fn test_u64_try_from_effect_value_success() {
    let ev = EffectValue::Number(NumericValue::U64(456));
    let u: u64 = IntoEffectValue::try_from_effect_value(ev).unwrap();
    assert_eq!(u, 456);
}

#[test]
fn test_u64_try_from_effect_value_error_wrong_numeric_type() {
    let ev = EffectValue::Number(NumericValue::I64(123));
    let err = <u64 as IntoEffectValue>::try_from_effect_value(ev).unwrap_err();
    assert!(
        err.to_string()
            .contains("Expected NumericValue::U64, found I64(123)")
    );
}

#[test]
fn test_u64_try_from_effect_value_error_wrong_effect_value_type() {
    let ev = EffectValue::Boolean(true);
    let err = <u64 as IntoEffectValue>::try_from_effect_value(ev).unwrap_err();
    assert!(
        err.to_string()
            .contains("Expected EffectValue::Number(NumericValue::U64), found Deterministic(true)")
    );
}

// Test for NumericValue
#[test]
fn test_numeric_value_into_effect_value() {
    let nv = NumericValue::F64(10.0);
    let ev: EffectValue = nv.into_effect_value();
    assert_eq!(ev, EffectValue::Number(NumericValue::F64(10.0)));
}

#[test]
fn test_numeric_value_try_from_effect_value_success() {
    let ev = EffectValue::Number(NumericValue::F64(20.0));
    let nv: NumericValue = IntoEffectValue::try_from_effect_value(ev).unwrap();
    assert_eq!(nv, NumericValue::F64(20.0));
}

#[test]
fn test_numeric_value_try_from_effect_value_error() {
    let ev = EffectValue::Boolean(true);
    let err = <NumericValue as IntoEffectValue>::try_from_effect_value(ev).unwrap_err();
    assert!(
        err.to_string()
            .contains("Expected Number(NumericValue), found Deterministic(true)")
    );
}

// Test for CausalTensor<f64>
#[test]
fn test_causal_tensor_f64_into_effect_value() {
    let tensor = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let ev: EffectValue = tensor.into_effect_value();
    assert_eq!(
        ev,
        EffectValue::Tensor(CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap())
    );
}

#[test]
fn test_causal_tensor_f64_try_from_effect_value_success() {
    let tensor = CausalTensor::new(vec![4.0, 5.0], vec![2]).unwrap();
    let ev = EffectValue::Tensor(tensor.clone());
    let t: CausalTensor<f64> = IntoEffectValue::try_from_effect_value(ev).unwrap();
    assert_eq!(t, tensor);
}

#[test]
fn test_causal_tensor_f64_try_from_effect_value_error() {
    let ev = EffectValue::Boolean(true);
    let err = <CausalTensor<f64> as IntoEffectValue>::try_from_effect_value(ev).unwrap_err();
    assert!(
        err.to_string()
            .contains("Expected Tensor(CausalTensor<f64>), found Deterministic(true)")
    );
}

// Test for Complex<f64>
#[test]
fn test_complex_f64_into_effect_value() {
    let c = Complex::new(1.0, 2.0);
    let ev: EffectValue = c.into_effect_value();
    assert_eq!(ev, EffectValue::Complex(Complex::new(1.0, 2.0)));
}

#[test]
fn test_complex_f64_try_from_effect_value_success() {
    let c = Complex::new(3.0, 4.0);
    let ev = EffectValue::Complex(c);
    let complex: Complex<f64> = IntoEffectValue::try_from_effect_value(ev).unwrap();
    assert_eq!(complex, c);
}

#[test]
fn test_complex_f64_try_from_effect_value_error() {
    let ev = EffectValue::Boolean(true);
    let err = <Complex<f64> as IntoEffectValue>::try_from_effect_value(ev).unwrap_err();
    assert!(
        err.to_string()
            .contains("Expected Complex(Complex<f64>), found Deterministic(true)")
    );
}

// Test for CausalTensor<Complex<f64>>
#[test]
fn test_causal_tensor_complex_f64_into_effect_value() {
    let tensor = CausalTensor::new(
        vec![Complex::new(1.0, 1.0), Complex::new(2.0, 2.0)],
        vec![2],
    )
    .unwrap();
    let ev: EffectValue = tensor.into_effect_value();
    assert_eq!(
        ev,
        EffectValue::ComplexTensor(
            CausalTensor::new(
                vec![Complex::new(1.0, 1.0), Complex::new(2.0, 2.0)],
                vec![2]
            )
            .unwrap()
        )
    );
}

#[test]
fn test_causal_tensor_complex_f64_try_from_effect_value_success() {
    let tensor = CausalTensor::new(vec![Complex::new(3.0, 3.0)], vec![1]).unwrap();
    let ev = EffectValue::ComplexTensor(tensor.clone());
    let t: CausalTensor<Complex<f64>> = IntoEffectValue::try_from_effect_value(ev).unwrap();
    assert_eq!(t, tensor);
}

#[test]
fn test_causal_tensor_complex_f64_try_from_effect_value_error() {
    let ev = EffectValue::Boolean(true);
    let err =
        <CausalTensor<Complex<f64>> as IntoEffectValue>::try_from_effect_value(ev).unwrap_err();
    assert!(
        err.to_string().contains(
            "Expected ComplexTensor(CausalTensor<Complex<f64>>), found Deterministic(true)"
        )
    );
}

// Test for Quaternion<f64>
#[test]
fn test_quaternion_f64_into_effect_value() {
    let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    let ev: EffectValue = q.into_effect_value();
    assert_eq!(
        ev,
        EffectValue::Quaternion(Quaternion::new(1.0, 2.0, 3.0, 4.0))
    );
}

#[test]
fn test_quaternion_f64_try_from_effect_value_success() {
    let q = Quaternion::new(5.0, 6.0, 7.0, 8.0);
    let ev = EffectValue::Quaternion(q);
    let quaternion: Quaternion<f64> = IntoEffectValue::try_from_effect_value(ev).unwrap();
    assert_eq!(quaternion, q);
}

#[test]
fn test_quaternion_f64_try_from_effect_value_error() {
    let ev = EffectValue::Boolean(true);
    let err = <Quaternion<f64> as IntoEffectValue>::try_from_effect_value(ev).unwrap_err();
    assert!(
        err.to_string()
            .contains("Expected Quaternion(Quaternion<f64>), found Deterministic(true)")
    );
}

// Test for CausalTensor<Quaternion<f64>>
#[test]
fn test_causal_tensor_quaternion_f64_into_effect_value() {
    let tensor = CausalTensor::new(vec![Quaternion::new(1.0, 1.0, 1.0, 1.0)], vec![1]).unwrap();
    let ev: EffectValue = tensor.into_effect_value();
    assert_eq!(
        ev,
        EffectValue::QuaternionTensor(
            CausalTensor::new(vec![Quaternion::new(1.0, 1.0, 1.0, 1.0)], vec![1]).unwrap()
        )
    );
}

#[test]
fn test_causal_tensor_quaternion_f64_try_from_effect_value_success() {
    let tensor = CausalTensor::new(vec![Quaternion::new(2.0, 2.0, 2.0, 2.0)], vec![1]).unwrap();
    let ev = EffectValue::QuaternionTensor(tensor.clone());
    let t: CausalTensor<Quaternion<f64>> = IntoEffectValue::try_from_effect_value(ev).unwrap();
    assert_eq!(t, tensor);
}

#[test]
fn test_causal_tensor_quaternion_f64_try_from_effect_value_error() {
    let ev = EffectValue::Boolean(true);
    let err =
        <CausalTensor<Quaternion<f64>> as IntoEffectValue>::try_from_effect_value(ev).unwrap_err();
    assert!(err.to_string().contains(
        "Expected QuaternionTensor(CausalTensor<Quaternion<f64>>), found Deterministic(true)"
    ));
}

// Test for UncertainBool
#[test]
fn test_uncertain_bool_into_effect_value() {
    let ub = Uncertain::<bool>::point(true);
    let ev: EffectValue = ub.into_effect_value();
    assert_eq!(
        ev,
        EffectValue::UncertainBool(Uncertain::<bool>::point(true))
    );
}

#[test]
fn test_uncertain_bool_try_from_effect_value_success() {
    let ub = Uncertain::<bool>::point(false);
    let ev = EffectValue::UncertainBool(ub.clone());
    let uncertain_bool: UncertainBool = IntoEffectValue::try_from_effect_value(ev).unwrap();
    assert_eq!(uncertain_bool, ub);
}

#[test]
fn test_uncertain_bool_try_from_effect_value_error() {
    let ev = EffectValue::Boolean(true);
    let err = <UncertainBool as IntoEffectValue>::try_from_effect_value(ev).unwrap_err();
    assert!(
        err.to_string()
            .contains("Expected UncertainBool(UncertainBool), found Deterministic(true)")
    );
}

// Test for UncertainF64
#[test]
fn test_uncertain_f64_into_effect_value() {
    let uf = Uncertain::<f64>::point(1.0);
    let ev: EffectValue = uf.into_effect_value();
    assert_eq!(
        ev,
        EffectValue::UncertainFloat(Uncertain::<f64>::point(1.0))
    );
}

#[test]
fn test_uncertain_f64_try_from_effect_value_success() {
    let uf = Uncertain::<f64>::point(2.0);
    let ev = EffectValue::UncertainFloat(uf.clone());
    let uncertain_f64: UncertainF64 = IntoEffectValue::try_from_effect_value(ev).unwrap();
    assert_eq!(uncertain_f64, uf);
}

#[test]
fn test_uncertain_f64_try_from_effect_value_error() {
    let ev = EffectValue::Boolean(true);
    let err = <UncertainF64 as IntoEffectValue>::try_from_effect_value(ev).unwrap_err();
    assert!(
        err.to_string()
            .contains("Expected UncertainFloat(UncertainF64), found Deterministic(true)")
    );
}

// Test for MaybeUncertainBool
#[test]
fn test_maybe_uncertain_bool_into_effect_value() {
    let mub = MaybeUncertain::<bool>::from_value(true);
    let ev: EffectValue = mub.into_effect_value();
    assert_eq!(
        ev,
        EffectValue::MaybeUncertainBool(MaybeUncertain::<bool>::from_value(true))
    );
}

#[test]
fn test_maybe_uncertain_bool_try_from_effect_value_success() {
    let mub = MaybeUncertainBool::from_value(false);
    let ev = EffectValue::MaybeUncertainBool(mub.clone());
    let maybe_uncertain_bool: MaybeUncertain<bool> =
        IntoEffectValue::try_from_effect_value(ev).unwrap();
    assert_eq!(maybe_uncertain_bool, mub);
}

#[test]
fn test_maybe_uncertain_bool_try_from_effect_value_error() {
    let ev = EffectValue::Boolean(true);
    let err = <MaybeUncertainBool as IntoEffectValue>::try_from_effect_value(ev).unwrap_err();
    assert!(
        err.to_string()
            .contains("Expected MaybeUncertainBool(MaybeUncertainBool), found Deterministic(true)")
    );
}

// Test for MaybeUncertainF64
#[test]
fn test_maybe_uncertain_f64_into_effect_value() {
    let muf = MaybeUncertainF64::from_value(1.0);
    let ev: EffectValue = muf.into_effect_value();
    assert_eq!(
        ev,
        EffectValue::MaybeUncertainFloat(MaybeUncertain::<f64>::from_value(1.0))
    );
}

#[test]
fn test_maybe_uncertain_f64_try_from_effect_value_success() {
    let muf = MaybeUncertainF64::from_value(2.0);
    let ev = EffectValue::MaybeUncertainFloat(muf.clone());
    let maybe_uncertain_f64: MaybeUncertain<f64> =
        IntoEffectValue::try_from_effect_value(ev).unwrap();
    assert_eq!(maybe_uncertain_f64, muf);
}

#[test]
fn test_maybe_uncertain_f64_try_from_effect_value_error() {
    let ev = EffectValue::Boolean(true);
    let err = <MaybeUncertainF64 as IntoEffectValue>::try_from_effect_value(ev).unwrap_err();
    assert!(
        err.to_string()
            .contains("Expected MaybeUncertainFloat(MaybeUncertainF64), found Deterministic(true)")
    );
}

// Test for () (Unit type)
#[test]
fn test_unit_into_effect_value() {
    let u = ();
    let ev: EffectValue = u.into_effect_value();
    assert_eq!(ev, EffectValue::None);
}

#[test]
fn test_unit_try_from_effect_value_success() {
    let ev = EffectValue::None;
    let u: () = IntoEffectValue::try_from_effect_value(ev).unwrap();
    assert_eq!(u, ());
}

#[test]
fn test_unit_try_from_effect_value_error() {
    let ev = EffectValue::Boolean(true);
    let err = <() as IntoEffectValue>::try_from_effect_value(ev).unwrap_err();
    assert!(
        err.to_string()
            .contains("Expected None, found Deterministic(true)")
    );
}

// Test for EffectValue
#[test]
fn test_effect_value_into_effect_value() {
    let ev_original = EffectValue::Boolean(true);
    let ev_converted: EffectValue = ev_original.clone().into_effect_value();
    assert_eq!(ev_converted, ev_original);
}

#[test]
fn test_effect_value_try_from_effect_value_success() {
    let ev_original = EffectValue::Numerical(123.45);
    let ev_converted: EffectValue =
        IntoEffectValue::try_from_effect_value(ev_original.clone()).unwrap();
    assert_eq!(ev_converted, ev_original);
}
