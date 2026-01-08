/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_tensor::{BackendTensor, CpuBackend, TensorBackend};

#[test]
fn test_backend_tensor_arithmetic_ops() {
    let shape = [2, 2];
    let data_a = vec![1.0, 2.0, 3.0, 4.0];
    let data_b = vec![10.0, 20.0, 30.0, 40.0];

    // Create tensors via BackendTensor wrapper
    let a = <BackendTensor<f64, CpuBackend>>::from_slice(&data_a, &shape);
    let b = <BackendTensor<f64, CpuBackend>>::from_slice(&data_b, &shape);

    // 1. Add
    let c = &a + &b;
    let c_vec = CpuBackend::into_vec(c.into_inner());
    assert_eq!(c_vec, vec![11.0, 22.0, 33.0, 44.0]);

    // 2. Sub
    let d = &b - &a;
    let d_vec = CpuBackend::into_vec(d.into_inner());
    assert_eq!(d_vec, vec![9.0, 18.0, 27.0, 36.0]);

    // 3. Mul (Element-wise)
    let e = &a * &b;
    let e_vec = CpuBackend::into_vec(e.into_inner());
    assert_eq!(e_vec, vec![10.0, 40.0, 90.0, 160.0]);

    // 4. Div
    let f = &b / &a;
    let f_vec = CpuBackend::into_vec(f.into_inner());
    assert_eq!(f_vec, vec![10.0, 10.0, 10.0, 10.0]);
}

#[test]
fn test_backend_tensor_assign_ops() {
    let shape = [2];
    let data = vec![10.0, 20.0];
    let mut a = <BackendTensor<f64, CpuBackend>>::from_slice(&data, &shape);
    let b = <BackendTensor<f64, CpuBackend>>::from_slice(&[1.0, 2.0], &shape);

    // AddAssign
    a += &b;
    assert_eq!(
        CpuBackend::into_vec(a.clone().into_inner()),
        vec![11.0, 22.0]
    );

    // SubAssign
    a -= &b;
    assert_eq!(
        CpuBackend::into_vec(a.clone().into_inner()),
        vec![10.0, 20.0]
    );

    // MulAssign
    a *= &b;
    assert_eq!(
        CpuBackend::into_vec(a.clone().into_inner()),
        vec![10.0, 40.0]
    );

    // DivAssign
    a /= &b;
    assert_eq!(
        CpuBackend::into_vec(a.clone().into_inner()),
        vec![10.0, 20.0]
    );
}

#[test]
fn test_backend_tensor_scalar_ops() {
    let shape = [2];
    let mut a = <BackendTensor<f64, CpuBackend>>::from_slice(&[1.0, 2.0], &shape);

    // Scalar Add
    let b = &a + 1.0;
    assert_eq!(CpuBackend::into_vec(b.into_inner()), vec![2.0, 3.0]);

    // Scalar Mul
    let c = &a * 2.0;
    assert_eq!(CpuBackend::into_vec(c.into_inner()), vec![2.0, 4.0]);

    // Scalar Assign
    a += 10.0;
    assert_eq!(
        CpuBackend::into_vec(a.clone().into_inner()),
        vec![11.0, 12.0]
    );
}
