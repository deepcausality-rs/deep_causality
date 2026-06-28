/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::{ConjugateScalar, Float106, FromPrimitive, RealField};
use deep_causality_tensor::{
    CausalTensor, CausalTensorError, CausalTensorTrain, TensorTrain, Truncation,
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

fn assert_dense_eq<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>(
    a: &CausalTensor<T>,
    b: &CausalTensor<T>,
) {
    assert_eq!(a.shape(), b.shape());
    for (x, y) in a.as_slice().iter().zip(b.as_slice().iter()) {
        assert!(
            (*x - *y).abs() <= tol::<T>(),
            "tensors differ beyond tolerance"
        );
    }
}

fn sample_3d<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>() -> CausalTensor<T> {
    // A 2×3×2 tensor with distinct entries.
    let data: Vec<f64> = (0..12).map(|i| (i as f64) * 0.5 - 1.0).collect();
    tensor::<T>(&data, &[2, 3, 2])
}

fn check_roundtrip<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>() {
    let dense = sample_3d::<T>();
    let full = Truncation::<T>::by_bond(1024).unwrap();
    let tt = CausalTensorTrain::from_dense(&dense, &full).unwrap();

    assert_eq!(tt.order(), 3);
    assert_eq!(tt.phys_dims(), &[2, 3, 2]);

    // from_dense → to_dense reproduces the original.
    let back = tt.to_dense().unwrap();
    assert_dense_eq(&dense, &back);

    // eval matches every dense entry.
    let d = dense.shape().to_vec();
    for i0 in 0..d[0] {
        for i1 in 0..d[1] {
            for i2 in 0..d[2] {
                let got = tt.eval(&[i0, i1, i2]).unwrap();
                let want = *dense.get(&[i0, i1, i2]).unwrap();
                assert!((got - want).abs() <= tol::<T>());
            }
        }
    }
}

fn check_order_one<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>() {
    let dense = tensor::<T>(&[1.0, -2.0, 3.0, 4.0], &[4]);
    let full = Truncation::<T>::by_bond(8).unwrap();
    let tt = CausalTensorTrain::from_dense(&dense, &full).unwrap();
    assert_eq!(tt.order(), 1);
    assert_dense_eq(&dense, &tt.to_dense().unwrap());
    assert!((tt.eval(&[2]).unwrap() - v::<T>(3.0)).abs() <= tol::<T>());
}

fn check_constructors<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>() {
    // zeros / ones via to_dense.
    let z = CausalTensorTrain::<T>::zeros(&[2, 2]).to_dense().unwrap();
    assert!(z.as_slice().iter().all(|x| x.abs() <= tol::<T>()));
    let o = CausalTensorTrain::<T>::ones(&[2, 2]).to_dense().unwrap();
    assert!(
        o.as_slice()
            .iter()
            .all(|x| (*x - v::<T>(1.0)).abs() <= tol::<T>())
    );

    // from_fn equals an explicit dense build.
    let full = Truncation::<T>::by_bond(64).unwrap();
    let tt = CausalTensorTrain::<T>::from_fn(
        &[2, 2, 2],
        |idx| v::<T>((idx[0] * 4 + idx[1] * 2 + idx[2]) as f64),
        &full,
    )
    .unwrap();
    let dense = tensor::<T>(&[0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0], &[2, 2, 2]);
    assert_dense_eq(&dense, &tt.to_dense().unwrap());
}

#[test]
fn test_roundtrip_f32() {
    check_roundtrip::<f32>();
}
#[test]
fn test_roundtrip_f64() {
    check_roundtrip::<f64>();
}
#[test]
fn test_roundtrip_float106() {
    check_roundtrip::<Float106>();
}

#[test]
fn test_order_one_f32() {
    check_order_one::<f32>();
}
#[test]
fn test_order_one_f64() {
    check_order_one::<f64>();
}
#[test]
fn test_order_one_float106() {
    check_order_one::<Float106>();
}

#[test]
fn test_constructors_f32() {
    check_constructors::<f32>();
}
#[test]
fn test_constructors_f64() {
    check_constructors::<f64>();
}
#[test]
fn test_constructors_float106() {
    check_constructors::<Float106>();
}

#[test]
fn test_random_seeded_is_deterministic() {
    let a = CausalTensorTrain::<f64>::random_seeded(&[3, 3, 3], 2, 42);
    let b = CausalTensorTrain::<f64>::random_seeded(&[3, 3, 3], 2, 42);
    let c = CausalTensorTrain::<f64>::random_seeded(&[3, 3, 3], 2, 43);
    assert_eq!(a.to_dense().unwrap(), b.to_dense().unwrap());
    assert_ne!(a.to_dense().unwrap(), c.to_dense().unwrap());
}

#[test]
fn test_from_cores_errors() {
    // Empty.
    assert!(matches!(
        CausalTensorTrain::<f64>::from_cores(vec![]),
        Err(CausalTensorError::EmptyTensor)
    ));
    // Wrong rank core.
    let bad = CausalTensor::new(vec![1.0, 2.0], vec![2]).unwrap();
    assert!(matches!(
        CausalTensorTrain::<f64>::from_cores(vec![bad]),
        Err(CausalTensorError::DimensionMismatch)
    ));
    // Boundary bond not 1.
    let core = CausalTensor::new(vec![0.0; 2 * 2], vec![2, 2, 1]).unwrap();
    assert!(matches!(
        CausalTensorTrain::<f64>::from_cores(vec![core]),
        Err(CausalTensorError::BondDimensionMismatch)
    ));
    // Adjacent bond mismatch.
    let c0 = CausalTensor::new(vec![0.0; 2 * 3], vec![1, 2, 3]).unwrap();
    let c1 = CausalTensor::new(vec![0.0; 2 * 2], vec![2, 2, 1]).unwrap();
    assert!(matches!(
        CausalTensorTrain::<f64>::from_cores(vec![c0, c1]),
        Err(CausalTensorError::BondDimensionMismatch)
    ));
}

#[test]
fn test_to_dense_and_from_fn_guards() {
    // from_fn guard: 2^25 elements exceeds the cap.
    let full = Truncation::<f64>::by_bond(4).unwrap();
    let res = CausalTensorTrain::<f64>::from_fn(&[2; 25], |_| 0.0, &full);
    assert!(matches!(res, Err(CausalTensorError::RankExceeded)));

    // eval index errors.
    let tt = CausalTensorTrain::from_dense(&sample_3d::<f64>(), &full).unwrap();
    assert!(matches!(
        tt.eval(&[0, 0]),
        Err(CausalTensorError::DimensionMismatch)
    ));
    assert!(matches!(
        tt.eval(&[0, 5, 0]),
        Err(CausalTensorError::IndexOutOfBounds)
    ));
}
