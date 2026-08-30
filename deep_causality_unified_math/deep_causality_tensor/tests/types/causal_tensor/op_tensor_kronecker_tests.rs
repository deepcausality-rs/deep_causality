/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num_complex::Complex;
use deep_causality_tensor::{CausalTensor, CausalTensorError, Tensor};

#[test]
fn test_kronecker_pauli_x_with_identity() {
    // σx ⊗ I = [[0,0,1,0],[0,0,0,1],[1,0,0,0],[0,1,0,0]]
    let sx = CausalTensor::new(vec![0.0, 1.0, 1.0, 0.0], vec![2, 2]).unwrap();
    let id = CausalTensor::new(vec![1.0, 0.0, 0.0, 1.0], vec![2, 2]).unwrap();
    let k = sx.kronecker(&id).unwrap();
    assert_eq!(k.shape(), &[4, 4]);
    #[rustfmt::skip]
    let expected = vec![
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
    ];
    assert_eq!(k.as_slice(), expected.as_slice());
}

#[test]
fn test_kronecker_rectangular_shapes() {
    // (1x2) ⊗ (2x1) = 2x2
    let a = CausalTensor::new(vec![2.0, 3.0], vec![1, 2]).unwrap();
    let b = CausalTensor::new(vec![5.0, 7.0], vec![2, 1]).unwrap();
    let k = a.kronecker(&b).unwrap();
    assert_eq!(k.shape(), &[2, 2]);
    // Block layout: [ 2*b | 3*b ] stacked as columns
    assert_eq!(k.as_slice(), &[10.0, 15.0, 14.0, 21.0]);
}

#[test]
fn test_kronecker_complex_entries() {
    let i = Complex::new(0.0, 1.0);
    let one = Complex::new(1.0, 0.0);
    let a = CausalTensor::new(vec![i, Complex::new(0.0, 0.0)], vec![1, 2]).unwrap();
    let b = CausalTensor::new(vec![one, i], vec![1, 2]).unwrap();
    let k = a.kronecker(&b).unwrap();
    assert_eq!(k.shape(), &[1, 4]);
    assert_eq!(k.as_slice()[0], i); // i * 1
    assert_eq!(k.as_slice()[1], Complex::new(-1.0, 0.0)); // i * i
    assert_eq!(k.as_slice()[2], Complex::new(0.0, 0.0));
    assert_eq!(k.as_slice()[3], Complex::new(0.0, 0.0));
}

#[test]
fn test_kronecker_mixed_product_identity() {
    // (A ⊗ B) · (C ⊗ D) = (A·C) ⊗ (B·D)
    let a: CausalTensor<f64> = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
    let b = CausalTensor::new(vec![0.0, 1.0, 1.0, 0.0], vec![2, 2]).unwrap();
    let c = CausalTensor::new(vec![2.0, 0.0, 0.0, 2.0], vec![2, 2]).unwrap();
    let d = CausalTensor::new(vec![1.0, 1.0, 0.0, 1.0], vec![2, 2]).unwrap();

    let lhs = a
        .kronecker(&b)
        .unwrap()
        .matmul(&c.kronecker(&d).unwrap())
        .unwrap();
    let rhs = a
        .matmul(&c)
        .unwrap()
        .kronecker(&b.matmul(&d).unwrap())
        .unwrap();
    assert_eq!(lhs.shape(), rhs.shape());
    for (l, r) in lhs.as_slice().iter().zip(rhs.as_slice()) {
        assert!((l - r).abs() < 1e-12);
    }
}

#[test]
fn test_kronecker_rejects_non_matrix() {
    let a = CausalTensor::new(vec![1.0, 2.0], vec![2]).unwrap();
    let b = CausalTensor::new(vec![1.0, 0.0, 0.0, 1.0], vec![2, 2]).unwrap();
    assert_eq!(
        a.kronecker(&b).unwrap_err(),
        CausalTensorError::DimensionMismatch
    );
    assert_eq!(
        b.kronecker(&a).unwrap_err(),
        CausalTensorError::DimensionMismatch
    );
}
