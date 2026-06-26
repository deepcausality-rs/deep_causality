/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::{Float106, FromPrimitive, RealField};
use deep_causality_tensor::{
    CausalTensor, CausalTensorTrain, CausalTensorTrainOperator, SolveConfig, TensorTrain,
    TensorTrainOperator, Truncation, solve,
};

fn v<T: FromPrimitive>(x: f64) -> T {
    T::from_f64(x).unwrap()
}

fn tensor<T: RealField + FromPrimitive>(data: &[f64], shape: &[usize]) -> CausalTensor<T> {
    CausalTensor::new(data.iter().map(|&x| v::<T>(x)).collect(), shape.to_vec()).unwrap()
}

fn tol<T: RealField + FromPrimitive>() -> T {
    T::epsilon().sqrt() * v::<T>(256.0)
}

fn full<T: RealField + FromPrimitive>() -> Truncation<T> {
    Truncation::by_bond(4096).unwrap()
}

fn known_train<T: RealField + FromPrimitive>(shape: &[usize]) -> CausalTensorTrain<T> {
    let data: Vec<f64> = (0..shape.iter().product::<usize>())
        .map(|i| (i as f64).sin() + 1.5)
        .collect();
    CausalTensorTrain::from_dense(&tensor::<T>(&data, shape), &full::<T>()).unwrap()
}

// ---- integrate ----

fn check_integrate<T: RealField + FromPrimitive>() {
    let shape = [2usize, 3, 2];
    let tt = known_train::<T>(&shape);
    // Integrate against all-ones weights == sum of all entries.
    let weights: Vec<CausalTensor<T>> = shape
        .iter()
        .map(|&n| CausalTensor::new(vec![v::<T>(1.0); n], vec![n]).unwrap())
        .collect();
    let got = tt.integrate(&weights).unwrap();
    let mut want = T::zero();
    for x in tt.to_dense().unwrap().as_slice() {
        want += *x;
    }
    assert!((got - want).abs() <= tol::<T>(), "integrate mismatch");
}

#[test]
fn test_integrate_f32() {
    check_integrate::<f32>();
}
#[test]
fn test_integrate_f64() {
    check_integrate::<f64>();
}
#[test]
fn test_integrate_float106() {
    check_integrate::<Float106>();
}

#[test]
fn test_integrate_errors() {
    let tt = known_train::<f64>(&[2, 2]);
    let w = vec![CausalTensor::new(vec![1.0, 1.0], vec![2]).unwrap()];
    assert!(matches!(
        tt.integrate(&w),
        Err(deep_causality_tensor::CausalTensorError::DimensionMismatch)
    ));
}

// ---- fit ----

fn check_fit<T: RealField + FromPrimitive>() {
    // Sample a known rank-bounded train fully, fit, and compare densely.
    let shape = [3usize, 3, 3];
    let target = known_train::<T>(&shape);
    let dense = target.to_dense().unwrap();

    let mut samples = Vec::new();
    let mut idx = vec![0usize; 3];
    for i0 in 0..3 {
        for i1 in 0..3 {
            for i2 in 0..3 {
                idx[0] = i0;
                idx[1] = i1;
                idx[2] = i2;
                samples.push((idx.clone(), *dense.get(&idx).unwrap()));
            }
        }
    }

    // Precision-scaled ridge so the achievable accuracy tracks the working precision.
    let cfg = SolveConfig::<T>::new(100, tol::<T>(), T::epsilon()).unwrap();
    let fitted = solve::fit(&shape, 4, &samples, &cfg).unwrap();
    assert_dense_eq(&fitted.to_dense().unwrap(), &dense);
}

fn assert_dense_eq<T: RealField + FromPrimitive>(a: &CausalTensor<T>, b: &CausalTensor<T>) {
    assert_eq!(a.shape(), b.shape());
    for (x, y) in a.as_slice().iter().zip(b.as_slice().iter()) {
        assert!((*x - *y).abs() <= tol::<T>(), "differ beyond tolerance");
    }
}

#[test]
fn test_fit_f64() {
    check_fit::<f64>();
}
#[test]
fn test_fit_float106() {
    check_fit::<Float106>();
}

// ---- linear ----

fn check_linear<T: RealField + FromPrimitive>() {
    // A is a 2-site operator; x* is a known state; b = A·x*. Solving recovers x*.
    let a_dense = tensor::<T>(
        &(0..16)
            .map(|i| ((i as f64) * 0.3).cos() + 1.2)
            .collect::<Vec<_>>(),
        &[2, 2, 2, 2],
    );
    let a =
        CausalTensorTrainOperator::from_dense(&a_dense, &[2, 2], &[2, 2], &full::<T>()).unwrap();
    let xstar = known_train::<T>(&[2, 2]);
    let b = a.apply(&xstar, &full::<T>()).unwrap();

    // Precision-scaled ridge so accuracy tracks the working precision.
    let cfg = SolveConfig::<T>::new(80, tol::<T>(), T::epsilon()).unwrap();
    let x = solve::linear(&a, &b, 4, &cfg).unwrap();

    // A·x ≈ b.
    let ax = a.apply(&x, &full::<T>()).unwrap();
    assert_dense_eq(&ax.to_dense().unwrap(), &b.to_dense().unwrap());
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
fn test_solve_config_errors() {
    assert!(matches!(
        SolveConfig::<f64>::new(0, 1e-6, 1e-12),
        Err(deep_causality_tensor::CausalTensorError::InvalidParameter(
            _
        ))
    ));
    assert!(matches!(
        SolveConfig::<f64>::new(10, -1.0, 1e-12),
        Err(deep_causality_tensor::CausalTensorError::InvalidParameter(
            _
        ))
    ));
}
