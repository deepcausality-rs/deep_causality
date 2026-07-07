/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Scalar-generality tests (Stage 4): the tensor-train stack is bound on `Scalar`
//! (`Real + Div + FromPrimitive`), which `Dual<f64>` satisfies, so forward-mode automatic
//! differentiation flows through the network with no change to the library code. Derivatives are
//! checked against central finite differences.

use deep_causality_algebra::{ConjugateScalar, Scalar};
use deep_causality_num::FromPrimitive;
use deep_causality_num_dual::Dual;
use deep_causality_tensor::{CausalTensor, CausalTensorTrain, TensorTrain, Truncation};

fn c<T: FromPrimitive>(x: f64) -> T {
    T::from_f64(x).unwrap()
}

/// Norm of a small tensor train whose core entries depend smoothly (affinely) on `t`. Built via
/// `from_cores` (pure arithmetic, no SVD), so the result is a smooth function of `t`.
fn param_norm<T: Scalar + ConjugateScalar<Real = T>>(t: T) -> T {
    let a = [1.0, -0.5, 0.7, 0.3, -0.2, 0.9, 0.4, -0.6];
    let b = [0.5, 0.2, -0.3, 0.8, 0.1, -0.4, 0.6, 0.25];
    let core0: Vec<T> = (0..4).map(|j| c::<T>(a[j]) + c::<T>(b[j]) * t).collect();
    let core1: Vec<T> = (0..4)
        .map(|j| c::<T>(a[j + 4]) + c::<T>(b[j + 4]) * t)
        .collect();
    let c0 = CausalTensor::new(core0, vec![1, 2, 2]).unwrap();
    let c1 = CausalTensor::new(core1, vec![2, 2, 1]).unwrap();
    let tt = CausalTensorTrain::from_cores(vec![c0, c1]).unwrap();
    tt.norm().unwrap()
}

/// A single entry of a tensor train built from a `t`-dependent dense tensor via TT-SVD, exercising
/// automatic differentiation *through the SVD kernel* (full rank, so reconstruction is exact and the
/// derivative must survive the factorization).
fn param_eval<T: Scalar + ConjugateScalar<Real = T>>(t: T, idx: &[usize]) -> T {
    let a = [1.3, -0.4, 0.6, 0.9];
    let b = [0.3, 0.7, -0.5, 0.2];
    let q = [0.1, -0.2, 0.15, 0.05];
    let data: Vec<T> = (0..4)
        .map(|j| c::<T>(a[j]) + c::<T>(b[j]) * t + c::<T>(q[j]) * t * t)
        .collect();
    let dense = CausalTensor::new(data, vec![2, 2]).unwrap();
    let trunc = Truncation::<T>::by_bond(4096).unwrap();
    let tt = CausalTensorTrain::from_dense(&dense, &trunc).unwrap();
    tt.eval(idx).unwrap()
}

#[test]
fn test_dual_ad_norm_vs_finite_difference() {
    let t0 = 0.37f64;
    // Forward-mode AD: seed the parameter with a unit derivative.
    let nd = param_norm::<Dual<f64>>(Dual::new(t0, 1.0));
    let ad = nd.derivative();
    // Central finite difference in plain f64.
    let h = 1e-6;
    let fd = (param_norm::<f64>(t0 + h) - param_norm::<f64>(t0 - h)) / (2.0 * h);
    assert!((ad - fd).abs() <= 1e-6, "AD {ad} vs FD {fd}");
    // The value channel matches the plain-f64 computation.
    assert!((nd.value() - param_norm::<f64>(t0)).abs() <= 1e-12);
}

#[test]
fn test_dual_ad_through_svd_vs_finite_difference() {
    let t0 = 0.21f64;
    let idx = [1usize, 0];
    let yd = param_eval::<Dual<f64>>(Dual::new(t0, 1.0), &idx);
    let ad = yd.derivative();
    let h = 1e-6;
    let fd = (param_eval::<f64>(t0 + h, &idx) - param_eval::<f64>(t0 - h, &idx)) / (2.0 * h);
    assert!((ad - fd).abs() <= 1e-5, "AD-through-SVD {ad} vs FD {fd}");
    assert!((yd.value() - param_eval::<f64>(t0, &idx)).abs() <= 1e-12);
}
