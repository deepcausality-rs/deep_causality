/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_tensor::{CausalTensor, CausalTensorError};

// --- group.rs: zero / add / sub / neg ---

#[test]
fn test_group_zero() {
    let z = CausalTensor::<f64>::zero(&[2, 3]);
    assert_eq!(z.shape(), &[2, 3]);
    assert_eq!(z.as_slice(), &[0.0; 6]);
}

#[test]
fn test_group_zero_scalar() {
    let z = CausalTensor::<f64>::zero(&[]);
    assert_eq!(z.shape(), &[] as &[usize]);
    assert_eq!(z.as_slice(), &[0.0]);
}

#[test]
fn test_group_add() {
    let a = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
    let b = CausalTensor::new(vec![10.0, 20.0, 30.0, 40.0], vec![2, 2]).unwrap();
    let c = a.add(&b);
    assert_eq!(c.shape(), &[2, 2]);
    assert_eq!(c.as_slice(), &[11.0, 22.0, 33.0, 44.0]);
}

#[test]
#[should_panic(expected = "Shape mismatch in addition")]
fn test_group_add_shape_mismatch_panics() {
    let a = CausalTensor::new(vec![1.0, 2.0], vec![2]).unwrap();
    let b = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let _ = a.add(&b);
}

#[test]
fn test_group_sub() {
    let a = CausalTensor::new(vec![10.0, 20.0, 30.0, 40.0], vec![2, 2]).unwrap();
    let b = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
    let c = a.sub(&b);
    assert_eq!(c.as_slice(), &[9.0, 18.0, 27.0, 36.0]);
}

#[test]
#[should_panic(expected = "Shape mismatch in subtraction")]
fn test_group_sub_shape_mismatch_panics() {
    let a = CausalTensor::new(vec![1.0, 2.0], vec![2]).unwrap();
    let b = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let _ = a.sub(&b);
}

#[test]
fn test_group_neg() {
    let a = CausalTensor::new(vec![1.0, -2.0, 3.0], vec![3]).unwrap();
    let n = a.neg();
    assert_eq!(n.as_slice(), &[-1.0, 2.0, -3.0]);
    assert_eq!(n.shape(), &[3]);
}

// --- module.rs: scale ---

#[test]
fn test_module_scale_f64() {
    let a = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
    let scaled = a.scale(2.0_f64);
    assert_eq!(scaled.as_slice(), &[2.0, 4.0, 6.0, 8.0]);
    assert_eq!(scaled.shape(), &[2, 2]);
}

#[test]
fn test_module_scale_i64() {
    let a = CausalTensor::new(vec![1_i64, 2, 3], vec![3]).unwrap();
    let scaled = a.scale(3_i64);
    assert_eq!(scaled.as_slice(), &[3, 6, 9]);
}

// --- ring.rs: mul / one / ones / identity ---

#[test]
fn test_ring_mul() {
    let a = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
    let b = CausalTensor::new(vec![5.0, 6.0, 7.0, 8.0], vec![2, 2]).unwrap();
    let c = a.mul(&b);
    assert_eq!(c.as_slice(), &[5.0, 12.0, 21.0, 32.0]);
    assert_eq!(c.shape(), &[2, 2]);
}

#[test]
#[should_panic(expected = "Shape mismatch in multiplication")]
fn test_ring_mul_shape_mismatch_panics() {
    let a = CausalTensor::new(vec![1.0, 2.0], vec![2]).unwrap();
    let b = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let _ = a.mul(&b);
}

#[test]
fn test_ring_one() {
    let o = CausalTensor::<f64>::one(&[2, 2]);
    assert_eq!(o.shape(), &[2, 2]);
    assert_eq!(o.as_slice(), &[1.0; 4]);
}

#[test]
fn test_ring_ones_alias() {
    let o = CausalTensor::<f64>::ones(&[3]);
    assert_eq!(o.shape(), &[3]);
    assert_eq!(o.as_slice(), &[1.0, 1.0, 1.0]);
}

#[test]
fn test_ring_identity_success() {
    let id = CausalTensor::<f64>::identity(&[3, 3]).unwrap();
    assert_eq!(id.shape(), &[3, 3]);
    assert_eq!(
        id.as_slice(),
        &[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]
    );
}

#[test]
fn test_ring_identity_not_2d() {
    let err = CausalTensor::<f64>::identity(&[3]).unwrap_err();
    assert_eq!(err, CausalTensorError::DimensionMismatch);

    let err3 = CausalTensor::<f64>::identity(&[2, 2, 2]).unwrap_err();
    assert_eq!(err3, CausalTensorError::DimensionMismatch);
}

#[test]
fn test_ring_identity_not_square() {
    let err = CausalTensor::<f64>::identity(&[2, 3]).unwrap_err();
    assert_eq!(err, CausalTensorError::ShapeMismatch);
}

// --- Blanket-implemented marker traits (AbelianGroup / Ring via deep_causality_num) ---

#[test]
fn test_blanket_add_group_via_num_trait() {
    use deep_causality_num::AddGroup;
    // Exercises the blanket AddGroup impl: Zero + Add + Sub + Neg.
    let a = CausalTensor::new(vec![1.0, 2.0], vec![2]).unwrap();
    let b = CausalTensor::new(vec![3.0, 4.0], vec![2]).unwrap();
    fn use_add_group<G: AddGroup + Clone>(x: &G, y: &G) -> G {
        x.clone() + y.clone()
    }
    let c = use_add_group(&a, &b);
    assert_eq!(c.as_slice(), &[4.0, 6.0]);
}
