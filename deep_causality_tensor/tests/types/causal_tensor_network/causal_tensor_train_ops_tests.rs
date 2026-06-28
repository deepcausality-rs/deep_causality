/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::{ConjugateScalar, Float106, FromPrimitive, RealField};
use deep_causality_tensor::{
    CausalTensor, CausalTensorError, CausalTensorTrain, Tensor, TensorTrain, Truncation,
};

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

fn close<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>(a: T, b: T) {
    assert!(
        (a - b).abs() <= tol::<T>(),
        "values differ beyond tolerance"
    );
}

fn assert_dense_eq<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>(
    a: &CausalTensor<T>,
    b: &CausalTensor<T>,
) {
    assert_eq!(a.shape(), b.shape());
    for (x, y) in a.as_slice().iter().zip(b.as_slice().iter()) {
        close(*x, *y);
    }
}

fn dense_a<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>() -> CausalTensor<T> {
    let data: Vec<f64> = (0..12).map(|i| (i as f64) * 0.5 - 1.0).collect();
    tensor::<T>(&data, &[2, 3, 2])
}
fn dense_b<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>() -> CausalTensor<T> {
    let data: Vec<f64> = (0..12).map(|i| 2.0 - (i as f64) * 0.3).collect();
    tensor::<T>(&data, &[2, 3, 2])
}

fn tt<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>(
    dense: &CausalTensor<T>,
) -> CausalTensorTrain<T> {
    let full = Truncation::<T>::by_bond(4096).unwrap();
    CausalTensorTrain::from_dense(dense, &full).unwrap()
}

fn check_inner_norm<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>() {
    let (da, db) = (dense_a::<T>(), dense_b::<T>());
    let (ta, tb) = (tt(&da), tt(&db));

    // inner matches the dense dot product.
    let mut want = T::zero();
    for (x, y) in da.as_slice().iter().zip(db.as_slice().iter()) {
        want += *x * *y;
    }
    close(ta.inner(&tb).unwrap(), want);

    // norm matches the dense Frobenius norm.
    let mut sq = T::zero();
    for x in da.as_slice() {
        sq += *x * *x;
    }
    close(ta.norm().unwrap(), sq.sqrt());
}

fn check_linear<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>() {
    let (da, db) = (dense_a::<T>(), dense_b::<T>());
    let (ta, tb) = (tt(&da), tt(&db));

    // add
    let sum = ta.add(&tb).unwrap().to_dense().unwrap();
    let want = tensor_from::<T>(
        da.as_slice()
            .iter()
            .zip(db.as_slice())
            .map(|(x, y)| *x + *y)
            .collect(),
        &[2, 3, 2],
    );
    assert_dense_eq(&sum, &want);

    // scale
    let s = v::<T>(2.5);
    let scaled = ta.scale(s).to_dense().unwrap();
    let want = tensor_from::<T>(da.as_slice().iter().map(|x| *x * s).collect(), &[2, 3, 2]);
    assert_dense_eq(&scaled, &want);

    // add_scalar
    let c = v::<T>(-0.75);
    let added = ta.add_scalar(c).unwrap().to_dense().unwrap();
    let want = tensor_from::<T>(da.as_slice().iter().map(|x| *x + c).collect(), &[2, 3, 2]);
    assert_dense_eq(&added, &want);

    // hadamard
    let had = ta.hadamard(&tb).unwrap().to_dense().unwrap();
    let want = tensor_from::<T>(
        da.as_slice()
            .iter()
            .zip(db.as_slice())
            .map(|(x, y)| *x * *y)
            .collect(),
        &[2, 3, 2],
    );
    assert_dense_eq(&had, &want);
}

fn check_marginalize<T: RealField + FromPrimitive + ConjugateScalar<Real = T> + Default>() {
    let da = dense_a::<T>();
    let ta = tt(&da);

    // Sum out the middle site → compare to dense sum_axes.
    let got = ta.marginalize(&[1]).unwrap().to_dense().unwrap();
    let want = da.sum_axes(&[1]).unwrap();
    assert_dense_eq(&got, &want);

    // Sum out the first site.
    let got = ta.marginalize(&[0]).unwrap().to_dense().unwrap();
    let want = da.sum_axes(&[0]).unwrap();
    assert_dense_eq(&got, &want);

    // Sum out two sites (first and last).
    let got = ta.marginalize(&[0, 2]).unwrap().to_dense().unwrap();
    let want = da.sum_axes(&[0, 2]).unwrap();
    assert_dense_eq(&got, &want);
}

fn tensor_from<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>(
    data: Vec<T>,
    shape: &[usize],
) -> CausalTensor<T> {
    CausalTensor::new(data, shape.to_vec()).unwrap()
}

#[test]
fn test_inner_norm_f32() {
    check_inner_norm::<f32>();
}
#[test]
fn test_inner_norm_f64() {
    check_inner_norm::<f64>();
}
#[test]
fn test_inner_norm_float106() {
    check_inner_norm::<Float106>();
}

#[test]
fn test_linear_f32() {
    check_linear::<f32>();
}
#[test]
fn test_linear_f64() {
    check_linear::<f64>();
}
#[test]
fn test_linear_float106() {
    check_linear::<Float106>();
}

#[test]
fn test_marginalize_f32() {
    check_marginalize::<f32>();
}
#[test]
fn test_marginalize_f64() {
    check_marginalize::<f64>();
}
#[test]
fn test_marginalize_float106() {
    check_marginalize::<Float106>();
}

/// Fused `hadamard_rounded` must equal `hadamard(other).round(trunc)` to tolerance, at every
/// supported real precision.
fn check_hadamard_rounded_fused<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>() {
    let (da, db) = (dense_a::<T>(), dense_b::<T>());
    let (ta, tb) = (tt(&da), tt(&db));
    for trunc in [
        Truncation::<T>::by_bond(4096).unwrap(),
        Truncation::<T>::by_tol(v::<T>(1e-10)).unwrap(),
    ] {
        let fused = ta
            .hadamard_rounded(&tb, &trunc)
            .unwrap()
            .to_dense()
            .unwrap();
        let build_then_round = ta
            .hadamard(&tb)
            .unwrap()
            .round(&trunc)
            .unwrap()
            .to_dense()
            .unwrap();
        assert_dense_eq(&fused, &build_then_round);
        // ...and both equal the exact elementwise product.
        let want = tensor_from::<T>(
            da.as_slice()
                .iter()
                .zip(db.as_slice())
                .map(|(x, y)| *x * *y)
                .collect(),
            &[2, 3, 2],
        );
        assert_dense_eq(&fused, &want);
    }
}
#[test]
fn test_hadamard_rounded_fused_f64() {
    check_hadamard_rounded_fused::<f64>();
}
#[test]
fn test_hadamard_rounded_fused_float106() {
    check_hadamard_rounded_fused::<Float106>();
}

#[test]
fn test_shape_mismatch_errors() {
    let a = tt(&dense_a::<f64>());
    let other = CausalTensorTrain::from_dense(
        &tensor::<f64>(&[1.0, 2.0, 3.0, 4.0], &[2, 2]),
        &Truncation::<f64>::by_bond(8).unwrap(),
    )
    .unwrap();
    assert!(matches!(
        a.inner(&other),
        Err(CausalTensorError::ShapeMismatch)
    ));
    assert!(matches!(
        a.add(&other),
        Err(CausalTensorError::ShapeMismatch)
    ));
    assert!(matches!(
        a.hadamard(&other),
        Err(CausalTensorError::ShapeMismatch)
    ));
    // marginalize errors.
    assert!(matches!(
        a.marginalize(&[9]),
        Err(CausalTensorError::IndexOutOfBounds)
    ));
    assert!(matches!(
        a.marginalize(&[0, 1, 2]),
        Err(CausalTensorError::InvalidParameter(_))
    ));
}
