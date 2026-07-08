/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algebra::{ConjugateScalar, Module, RealField, Ring};
use deep_causality_num::{Float106, FromPrimitive, One, Zero};
use deep_causality_tensor::{CausalTensor, CausalTensorTrain, TensorTrain, Truncation};

fn v<T: FromPrimitive>(x: f64) -> T {
    T::from_f64(x).unwrap()
}

fn tensor<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>(
    data: &[f64],
    shape: &[usize],
) -> CausalTensor<T> {
    CausalTensor::new(data.iter().map(|&x| v::<T>(x)).collect(), shape.to_vec()).unwrap()
}

fn tol<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>() -> T {
    T::epsilon().sqrt() * v::<T>(64.0)
}

fn assert_dense_eq<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>(
    a: &CausalTensor<T>,
    b: &CausalTensor<T>,
) {
    assert_eq!(a.shape(), b.shape());
    for (x, y) in a.as_slice().iter().zip(b.as_slice().iter()) {
        assert!((*x - *y).abs() <= tol::<T>(), "differ beyond tolerance");
    }
}

fn sample<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>(
    seed: f64,
) -> CausalTensorTrain<T> {
    let data: Vec<f64> = (0..12).map(|i| (i as f64) * 0.3 + seed).collect();
    CausalTensorTrain::from_dense(
        &tensor::<T>(&data, &[2, 3, 2]),
        &Truncation::<T>::by_bond(64).unwrap(),
    )
    .unwrap()
}

// Generic bounds that only compile if the markers actually engage.
fn assert_is_module<V: Module<f64>>(_: &V) {}
fn assert_is_ring<R: Ring>(_: &R) {}

fn check_additive_identity<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>() {
    let a = sample::<T>(1.0);
    let zero = CausalTensorTrain::<T>::zero();

    // a + 0 = a and 0 + a = a, for an `a` of arbitrary shape.
    assert_dense_eq(
        &(a.clone() + zero.clone()).to_dense().unwrap(),
        &a.to_dense().unwrap(),
    );
    assert_dense_eq(
        &(zero.clone() + a.clone()).to_dense().unwrap(),
        &a.to_dense().unwrap(),
    );

    // is_zero / densification of the bare identity.
    assert!(zero.is_zero());
    assert!(!a.is_zero());
    assert_eq!(zero.to_dense().unwrap().shape(), &[] as &[usize]);
    assert!(zero.to_dense().unwrap().as_slice()[0].abs() <= tol::<T>());

    // a - a is zero.
    let diff = a.clone() - a.clone();
    for x in diff.to_dense().unwrap().as_slice() {
        assert!(x.abs() <= tol::<T>());
    }
}

fn check_multiplicative_identity<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>() {
    let a = sample::<T>(0.5);
    let one = CausalTensorTrain::<T>::one();

    // a * 1 = a and 1 * a = a under the Hadamard product.
    assert_dense_eq(
        &(a.clone() * one.clone()).to_dense().unwrap(),
        &a.to_dense().unwrap(),
    );
    assert_dense_eq(
        &(one.clone() * a.clone()).to_dense().unwrap(),
        &a.to_dense().unwrap(),
    );
    assert!(one.is_one());
    assert!(!a.is_one());
    let od = one.to_dense().unwrap();
    assert_eq!(od.shape(), &[] as &[usize]);
    assert!((od.as_slice()[0] - v::<T>(1.0)).abs() <= tol::<T>());
}

fn check_operators<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>() {
    let a = sample::<T>(1.0);
    let b = sample::<T>(-2.0);

    // + matches the trait add.
    assert_dense_eq(
        &(a.clone() + b.clone()).to_dense().unwrap(),
        &a.add(&b).unwrap().to_dense().unwrap(),
    );
    // scalar Mul<T> matches scale.
    let s = v::<T>(3.0);
    assert_dense_eq(
        &(a.clone() * s).to_dense().unwrap(),
        &a.scale(s).to_dense().unwrap(),
    );
    // Hadamard Mul<Self> matches the trait.
    assert_dense_eq(
        &(a.clone() * b.clone()).to_dense().unwrap(),
        &a.hadamard(&b).unwrap().to_dense().unwrap(),
    );
    // Module::scale provided method.
    let scaled = Module::scale(&a, s);
    assert_dense_eq(&scaled.to_dense().unwrap(), &a.scale(s).to_dense().unwrap());
}

#[test]
fn test_markers_engage() {
    let a = sample::<f64>(1.0);
    assert_is_module(&a);
    assert_is_ring(&a);
}

#[test]
fn test_additive_identity_f32() {
    check_additive_identity::<f32>();
}
#[test]
fn test_additive_identity_f64() {
    check_additive_identity::<f64>();
}
#[test]
fn test_additive_identity_float106() {
    check_additive_identity::<Float106>();
}

#[test]
fn test_multiplicative_identity_f32() {
    check_multiplicative_identity::<f32>();
}
#[test]
fn test_multiplicative_identity_f64() {
    check_multiplicative_identity::<f64>();
}
#[test]
fn test_multiplicative_identity_float106() {
    check_multiplicative_identity::<Float106>();
}

#[test]
fn test_operators_f32() {
    check_operators::<f32>();
}
#[test]
fn test_operators_f64() {
    check_operators::<f64>();
}
#[test]
fn test_operators_float106() {
    check_operators::<Float106>();
}
