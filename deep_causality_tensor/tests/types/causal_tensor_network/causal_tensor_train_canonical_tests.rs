/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algebra::{ConjugateScalar, RealField};
use deep_causality_num::{Float106, FromPrimitive};
use deep_causality_tensor::{
    CanonicalForm, CausalTensor, CausalTensorError, CausalTensorTrain, TensorTrain, Truncation,
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

fn sample<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>() -> CausalTensor<T> {
    let data: Vec<f64> = (0..24).map(|i| (i as f64).sin()).collect();
    tensor::<T>(&data, &[2, 3, 4])
}

fn tt<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>(
    dense: &CausalTensor<T>,
) -> CausalTensorTrain<T> {
    CausalTensorTrain::from_dense(dense, &Truncation::<T>::by_bond(4096).unwrap()).unwrap()
}

/// Columns of each core (reshaped `[rl*n, rr]`) are orthonormal.
fn assert_left_orthonormal<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>(
    core: &CausalTensor<T>,
) {
    let (rl, n, rr) = (core.shape()[0], core.shape()[1], core.shape()[2]);
    let rows = rl * n;
    let data = core.as_slice();
    for a in 0..rr {
        for b in 0..rr {
            let mut dot = T::zero();
            for r in 0..rows {
                dot += data[r * rr + a] * data[r * rr + b];
            }
            close(dot, if a == b { v::<T>(1.0) } else { v::<T>(0.0) });
        }
    }
}

/// Rows of each core (reshaped `[rl, n*rr]`) are orthonormal.
fn assert_right_orthonormal<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>(
    core: &CausalTensor<T>,
) {
    let (rl, n, rr) = (core.shape()[0], core.shape()[1], core.shape()[2]);
    let cols = n * rr;
    let data = core.as_slice();
    for a in 0..rl {
        for b in 0..rl {
            let mut dot = T::zero();
            for c in 0..cols {
                dot += data[a * cols + c] * data[b * cols + c];
            }
            close(dot, if a == b { v::<T>(1.0) } else { v::<T>(0.0) });
        }
    }
}

fn check_left<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>() {
    let dense = sample::<T>();
    let lc = tt(&dense).left_canonicalize().unwrap();
    assert_eq!(lc.canonical_form(), CanonicalForm::LeftAt(2));
    assert_dense_eq(&dense, &lc.to_dense().unwrap());
    // All but the last core are left-orthonormal.
    for core in &lc.cores()[..lc.order() - 1] {
        assert_left_orthonormal(core);
    }
}

fn check_right<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>() {
    let dense = sample::<T>();
    let rc = tt(&dense).right_canonicalize().unwrap();
    assert_eq!(rc.canonical_form(), CanonicalForm::RightAt(0));
    assert_dense_eq(&dense, &rc.to_dense().unwrap());
    // All but the first core are right-orthonormal.
    for core in &rc.cores()[1..] {
        assert_right_orthonormal(core);
    }
}

fn check_mixed<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>() {
    let dense = sample::<T>();
    let mc = tt(&dense).canonicalize_at(1).unwrap();
    assert_eq!(mc.canonical_form(), CanonicalForm::Mixed(1));
    assert_dense_eq(&dense, &mc.to_dense().unwrap());
    assert_left_orthonormal(&mc.cores()[0]);
    assert_right_orthonormal(&mc.cores()[2]);
}

fn check_round<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>() {
    let dense = sample::<T>();
    let ta = tt(&dense);
    let full = Truncation::<T>::by_bond(4096).unwrap();

    // a + a has inflated bonds but represents 2·a; rounding recompresses losslessly.
    let doubled = ta.add(&ta).unwrap();
    let inflated_bond = doubled.max_bond();
    let rounded = doubled.round(&full).unwrap();
    assert_eq!(rounded.canonical_form(), CanonicalForm::Mixed(0));

    // Compare to 2·dense.
    let two_dense = CausalTensor::new(
        dense.as_slice().iter().map(|x| *x + *x).collect(),
        dense.shape().to_vec(),
    )
    .unwrap();
    assert_dense_eq(&rounded.to_dense().unwrap(), &two_dense);

    // Rounding compressed the bond back below the inflated value.
    assert!(rounded.max_bond() <= ta.max_bond());
    assert!(rounded.max_bond() < inflated_bond);
}

fn check_round_truncation<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>() {
    // A separable (rank-1) tensor: outer product, exact at bond 1.
    let a = [v::<T>(1.0), v::<T>(2.0)];
    let b = [v::<T>(1.0), v::<T>(-1.0), v::<T>(0.5)];
    let mut data = vec![T::zero(); 6];
    for i in 0..2 {
        for j in 0..3 {
            data[i * 3 + j] = a[i] * b[j];
        }
    }
    let dense = CausalTensor::new(data, vec![2, 3]).unwrap();
    let tt =
        CausalTensorTrain::from_dense(&dense, &Truncation::<T>::by_bond(4096).unwrap()).unwrap();
    let rounded = tt
        .round(&Truncation::<T>::by_tol(v::<T>(1e-6)).unwrap())
        .unwrap();
    // Rank-1 ⇒ a single interior bond of dimension 1.
    assert_eq!(rounded.bond_dims(), vec![1]);
    assert_dense_eq(&rounded.to_dense().unwrap(), &dense);
}

#[test]
fn test_left_f32() {
    check_left::<f32>();
}
#[test]
fn test_left_f64() {
    check_left::<f64>();
}
#[test]
fn test_left_float106() {
    check_left::<Float106>();
}

#[test]
fn test_right_f32() {
    check_right::<f32>();
}
#[test]
fn test_right_f64() {
    check_right::<f64>();
}
#[test]
fn test_right_float106() {
    check_right::<Float106>();
}

#[test]
fn test_mixed_f32() {
    check_mixed::<f32>();
}
#[test]
fn test_mixed_f64() {
    check_mixed::<f64>();
}
#[test]
fn test_mixed_float106() {
    check_mixed::<Float106>();
}

#[test]
fn test_round_f32() {
    check_round::<f32>();
}
#[test]
fn test_round_f64() {
    check_round::<f64>();
}
#[test]
fn test_round_float106() {
    check_round::<Float106>();
}

#[test]
fn test_round_truncation_f64() {
    check_round_truncation::<f64>();
}

#[test]
fn test_canonicalize_at_out_of_bounds() {
    let tt = tt(&sample::<f64>());
    assert!(matches!(
        tt.canonicalize_at(9),
        Err(CausalTensorError::IndexOutOfBounds)
    ));
}
