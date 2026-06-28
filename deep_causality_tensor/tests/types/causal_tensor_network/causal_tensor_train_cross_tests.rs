/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::{ConjugateScalar, Float106, FromPrimitive, RealField};
use deep_causality_tensor::{
    CausalTensor, CausalTensorError, CausalTensorTrain, CrossConfig, TensorTrain,
};

fn v<T: FromPrimitive>(x: f64) -> T {
    T::from_f64(x).unwrap()
}

fn tol<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>() -> T {
    T::epsilon().sqrt() * v::<T>(64.0)
}

/// Densifies an oracle for a reference comparison.
fn dense_of<T: RealField + FromPrimitive + ConjugateScalar<Real = T>, F: FnMut(&[usize]) -> T>(
    shape: &[usize],
    f: F,
) -> CausalTensor<T> {
    CausalTensor::from_shape_fn(shape, f)
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

// Rank-1 separable oracle: f(i) = ∏_k (i_k + k + 1).
fn rank1<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>(idx: &[usize]) -> T {
    let mut p = v::<T>(1.0);
    for (k, &i) in idx.iter().enumerate() {
        p *= v::<T>((i + k + 1) as f64);
    }
    p
}

// Rank-2 oracle: f(i) = ∏ (i_k+1) + ∏ (-1)^? ... use two distinct separable terms.
fn rank2<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>(idx: &[usize]) -> T {
    let mut a = v::<T>(1.0);
    let mut b = v::<T>(1.0);
    for (k, &i) in idx.iter().enumerate() {
        a *= v::<T>((i + 1) as f64);
        b *= v::<T>(((i as f64) - (k as f64)).cos());
    }
    a + b
}

fn check_rank1<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>() {
    let shape = [3usize, 3, 3];
    let cfg = CrossConfig::<T>::with_rank_cap(4, v::<T>(1e-10)).unwrap();
    let (tt, residual) = CausalTensorTrain::cross(&shape, rank1::<T>, &cfg).unwrap();

    assert!(residual <= tol::<T>(), "residual too large");
    // Rank-1 ⇒ all interior bonds are 1.
    assert_eq!(tt.bond_dims(), vec![1, 1]);
    assert_dense_eq(&tt.to_dense().unwrap(), &dense_of(&shape, rank1::<T>));
}

fn check_rank2<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>() {
    let shape = [4usize, 4, 4, 4];
    let cfg = CrossConfig::<T>::with_rank_cap(6, v::<T>(1e-10)).unwrap();
    let (tt, residual) = CausalTensorTrain::cross(&shape, rank2::<T>, &cfg).unwrap();

    assert!(residual <= tol::<T>(), "residual too large");
    // Rank ≤ 2 everywhere.
    assert!(tt.bond_dims().iter().all(|&r| r <= 2));
    assert_dense_eq(&tt.to_dense().unwrap(), &dense_of(&shape, rank2::<T>));
}

#[test]
fn test_cross_rank1_f32() {
    check_rank1::<f32>();
}
#[test]
fn test_cross_rank1_f64() {
    check_rank1::<f64>();
}
#[test]
fn test_cross_rank1_float106() {
    check_rank1::<Float106>();
}

#[test]
fn test_cross_rank2_f64() {
    check_rank2::<f64>();
}
#[test]
fn test_cross_rank2_float106() {
    check_rank2::<Float106>();
}

#[test]
fn test_apply_nonlinear_square() {
    // Squaring a rank-1 separable tensor stays rank-1, so the cross re-approximation is exact.
    let shape = [3usize, 3, 3];
    let cfg = CrossConfig::<f64>::with_rank_cap(4, 1e-10).unwrap();
    let (tt, _) = CausalTensorTrain::cross(&shape, rank1::<f64>, &cfg).unwrap();

    let (sq, residual) = tt.apply_nonlinear(|x| x * x, &cfg).unwrap();
    assert!(residual <= 1e-8, "nonlinear residual too large");
    let want = dense_of(&shape, |idx| {
        let r = rank1::<f64>(idx);
        r * r
    });
    assert_dense_eq(&sq.to_dense().unwrap(), &want);
}

#[test]
fn test_apply_nonlinear_linear_matches_scale() {
    let shape = [3usize, 4];
    let cfg = CrossConfig::<f64>::with_rank_cap(4, 1e-10).unwrap();
    let (tt, _) = CausalTensorTrain::cross(&shape, rank2::<f64>, &cfg).unwrap();
    let (scaled, _) = tt.apply_nonlinear(|x| x * 2.0, &cfg).unwrap();
    assert_dense_eq(
        &scaled.to_dense().unwrap(),
        &tt.scale(2.0).to_dense().unwrap(),
    );
}

#[test]
fn test_cross_order_one() {
    let shape = [5usize];
    let cfg = CrossConfig::<f64>::with_rank_cap(2, 1e-12).unwrap();
    let (tt, residual) =
        CausalTensorTrain::cross(&shape, |i| (i[0] as f64) * 2.0 - 3.0, &cfg).unwrap();
    assert_eq!(tt.order(), 1);
    assert!(residual <= 1e-9);
    assert!((tt.eval(&[3]).unwrap() - 3.0).abs() <= 1e-9);
}

#[test]
fn test_cross_non_finite_oracle() {
    let shape = [3usize, 3];
    let cfg = CrossConfig::<f64>::with_rank_cap(2, 1e-10).unwrap();
    let res = CausalTensorTrain::cross(&shape, |_| f64::NAN, &cfg);
    assert!(matches!(res, Err(CausalTensorError::CrossSampleFailure)));
}

#[test]
fn test_cross_config_errors() {
    assert!(matches!(
        CrossConfig::<f64>::new(0, 2, 1e-6, 16, 1),
        Err(CausalTensorError::InvalidParameter(_))
    ));
    assert!(matches!(
        CrossConfig::<f64>::new(4, 0, 1e-6, 16, 1),
        Err(CausalTensorError::InvalidParameter(_))
    ));
    assert!(matches!(
        CrossConfig::<f64>::new(4, 2, -1.0, 16, 1),
        Err(CausalTensorError::InvalidParameter(_))
    ));
}
