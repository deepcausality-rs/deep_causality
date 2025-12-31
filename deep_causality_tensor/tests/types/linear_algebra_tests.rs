/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_tensor::{BackendTensor, CpuBackend, LinearAlgebraBackend, TensorBackend};

#[test]
fn test_linear_algebra_wrappers() {
    type Tensor = BackendTensor<f64, CpuBackend>;

    // 1. Cholesky
    let a_data = vec![4.0, 12.0, -16.0, 12.0, 37.0, -43.0, -16.0, -43.0, 98.0];
    let a = Tensor::from_slice(&a_data, &[3, 3]);

    // Call wrapper directly on CpuBackend to cover the trait impl
    let l = <CpuBackend as LinearAlgebraBackend>::cholesky_decomposition(&to_inner_clone(&a));
    let l_vec = <CpuBackend as TensorBackend>::to_vec(&l);
    // Check diagonal elements roughly
    assert!((l_vec[0] - 2.0).abs() < 1e-4); // L[0,0]

    // 2. Solve Least Squares Cholesky
    let id_data = vec![1.0, 0.0, 0.0, 1.0];
    let id = Tensor::from_slice(&id_data, &[2, 2]);
    let b = Tensor::from_slice(&[2.0, 3.0], &[2, 1]);

    let x = <CpuBackend as LinearAlgebraBackend>::solve_least_squares_cholsky(
        &to_inner_clone(&id),
        &to_inner_clone(&b),
    );
    let x_vec = <CpuBackend as TensorBackend>::to_vec(&x);
    assert!((x_vec[0] - 2.0).abs() < 1e-4);
    assert!((x_vec[1] - 3.0).abs() < 1e-4);

    // 3. Inverse
    let inv = <CpuBackend as LinearAlgebraBackend>::inverse(&to_inner_clone(&id));
    let inv_vec = <CpuBackend as TensorBackend>::to_vec(&inv);
    assert!((inv_vec[0] - 1.0).abs() < 1e-4);

    // 4. Tensor Product
    let tp = <CpuBackend as LinearAlgebraBackend>::tensor_product(
        &to_inner_clone(&id),
        &to_inner_clone(&id),
    );
    assert_eq!(
        type_name_of_val(&tp),
        "deep_causality_tensor::types::cpu_tensor::InternalCpuTensor<f64>"
    );
}

fn type_name_of_val<T>(_: &T) -> &'static str {
    std::any::type_name::<T>()
}

fn to_inner_clone<T: Clone, B: TensorBackend>(t: &BackendTensor<T, B>) -> B::Tensor<T>
where
    B::Tensor<T>: Clone,
{
    // The issue might be trait bounds on the function.
    // If we assume T: Clone, does B::Tensor<T> implement Clone?
    // We strictly need B::Tensor<T>: Clone for BackendTensor to be Clone.
    // So add that bound to the helper.
    let owned = <BackendTensor<T, B> as Clone>::clone(t);
    owned.into_inner()
}
