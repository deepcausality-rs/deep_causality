/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_tensor::{BackendTensor, CpuBackend, TensorBackend};

#[test]
fn test_reduction_shape_ops() {
    let data = vec![1.0, 2.0, 3.0, 4.0];
    let tensor = <BackendTensor<f64, CpuBackend>>::from_slice(&data, &[2, 2]);

    // 1. Sum
    let s = tensor.sum(&[0]); // Sum along axis 0 -> [4, 6]
    let s_vec = <CpuBackend as TensorBackend>::to_vec(&s.clone().into_inner());
    assert_eq!(s_vec, vec![4.0, 6.0]);

    // 2. Max
    let m = tensor.max(&[0]); // Max along axis 0 -> [3, 4]
    let m_vec = <CpuBackend as TensorBackend>::to_vec(&m.clone().into_inner());
    assert_eq!(m_vec, vec![3.0, 4.0]);

    // 3. Mean
    let avg = tensor.mean(&[0]); // Mean along axis 0 -> [2, 3]
    let avg_vec = <CpuBackend as TensorBackend>::to_vec(&avg.clone().into_inner());
    assert_eq!(avg_vec, vec![2.0, 3.0]);

    // 4. ArgSort (BackendTensor::arg_sort)
    let tensor_1d = <BackendTensor<f64, CpuBackend>>::from_slice(&[4.0, 1.0, 3.0, 2.0], &[4]);
    let sorted_idx = tensor_1d.arg_sort();
    assert!(sorted_idx.is_ok());
    assert_eq!(sorted_idx.unwrap(), vec![1, 3, 2, 0]);

    // 5. Ravel
    use deep_causality_tensor::Tensor;
    let raveled = tensor.ravel();
    let r_vec = <CpuBackend as TensorBackend>::to_vec(&raveled.clone().into_inner());
    assert_eq!(r_vec, vec![1.0, 2.0, 3.0, 4.0]);
}
