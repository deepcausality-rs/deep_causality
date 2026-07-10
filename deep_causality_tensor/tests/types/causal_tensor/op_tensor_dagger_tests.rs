/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num_complex::Complex;
use deep_causality_tensor::{CausalTensor, CausalTensorError};

#[test]
fn test_dagger_real_is_transpose() {
    let a = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], vec![2, 3]).unwrap();
    let d = a.dagger().unwrap();
    assert_eq!(d.shape(), &[3, 2]);
    assert_eq!(d.as_slice(), &[1.0, 4.0, 2.0, 5.0, 3.0, 6.0]);
}

#[test]
fn test_dagger_complex_conjugates() {
    let a = CausalTensor::new(
        vec![
            Complex::new(1.0, 2.0),
            Complex::new(3.0, -4.0),
            Complex::new(0.0, 1.0),
            Complex::new(5.0, 0.0),
        ],
        vec![2, 2],
    )
    .unwrap();
    let d = a.dagger().unwrap();
    assert_eq!(d.as_slice()[0], Complex::new(1.0, -2.0)); // conj(a00)
    assert_eq!(d.as_slice()[1], Complex::new(0.0, -1.0)); // conj(a10)
    assert_eq!(d.as_slice()[2], Complex::new(3.0, 4.0)); // conj(a01)
    assert_eq!(d.as_slice()[3], Complex::new(5.0, 0.0)); // conj(a11)
}

#[test]
fn test_dagger_involution() {
    let a = CausalTensor::new(
        vec![
            Complex::new(1.0, 2.0),
            Complex::new(3.0, -4.0),
            Complex::new(0.0, 1.0),
            Complex::new(5.0, -6.0),
            Complex::new(7.0, 8.0),
            Complex::new(-9.0, 0.5),
        ],
        vec![2, 3],
    )
    .unwrap();
    let dd = a.dagger().unwrap().dagger().unwrap();
    assert_eq!(dd.shape(), a.shape());
    assert_eq!(dd.as_slice(), a.as_slice());
}

#[test]
fn test_dagger_rejects_non_matrix() {
    let a = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    assert_eq!(a.dagger().unwrap_err(), CausalTensorError::DimensionMismatch);

    let b = CausalTensor::new(vec![1.0; 8], vec![2, 2, 2]).unwrap();
    assert_eq!(b.dagger().unwrap_err(), CausalTensorError::DimensionMismatch);
}
