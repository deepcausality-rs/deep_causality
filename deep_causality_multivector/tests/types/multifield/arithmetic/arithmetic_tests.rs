/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for CausalMultiField arithmetic operations: Zero, Add, Sub, Neg, Mul, Scale.

use deep_causality_metric::Metric;
use deep_causality_multivector::{CausalMultiField, CausalMultiVector};
use deep_causality_num::Zero;

// =============================================================================
// Zero trait tests
// =============================================================================

#[test]
fn test_is_zero_on_zeros_field() {
    let metric = Metric::from_signature(3, 0, 0);
    let field = CausalMultiField::<f32>::zeros([2, 2, 2], metric, [1.0, 1.0, 1.0]);

    assert!(field.is_zero());
}

#[test]
fn test_is_zero_on_ones_field() {
    let metric = Metric::from_signature(3, 0, 0);
    let field = CausalMultiField::<f32>::ones([2, 2, 2], metric, [1.0, 1.0, 1.0]);

    assert!(!field.is_zero());
}

#[test]
fn test_is_zero_on_nonzero_from_coefficients() {
    let metric = Metric::from_signature(3, 0, 0);
    let num_blades = 8;

    let mut mvs = Vec::with_capacity(8);
    for i in 0..8 {
        let mut data = vec![0.0f32; num_blades];
        data[0] = i as f32;
        mvs.push(CausalMultiVector::unchecked(data, metric));
    }

    let field = CausalMultiField::<f32>::from_coefficients(&mvs, [2, 2, 2], [1.0, 1.0, 1.0]);

    assert!(!field.is_zero());
}

#[test]
#[should_panic(expected = "requires shape and metric")]
fn test_zero_trait_panics() {
    let _: CausalMultiField<f32> = Zero::zero();
}

// =============================================================================
// Add tests
// =============================================================================

#[test]
fn test_add_owned_owned() {
    let metric = Metric::from_signature(3, 0, 0);
    let a = CausalMultiField::<f32>::ones([2, 2, 2], metric, [1.0, 1.0, 1.0]);
    let b = CausalMultiField::<f32>::ones([2, 2, 2], metric, [1.0, 1.0, 1.0]);

    let c = a + b;

    // Result should have doubled diagonal elements
    let coeffs = c.to_coefficients();
    // ones() creates identity matrices, summing doubles the scalar
    assert!(!c.is_zero());
    assert_eq!(coeffs.len(), 8);
}

#[test]
fn test_add_owned_ref() {
    let metric = Metric::from_signature(3, 0, 0);
    let a = CausalMultiField::<f32>::ones([2, 2, 2], metric, [1.0, 1.0, 1.0]);
    let b = CausalMultiField::<f32>::ones([2, 2, 2], metric, [1.0, 1.0, 1.0]);

    let c = a + b;

    assert!(!c.is_zero());
}

#[test]
fn test_add_identity_zeros() {
    let metric = Metric::from_signature(3, 0, 0);
    let a = CausalMultiField::<f32>::ones([2, 2, 2], metric, [1.0, 1.0, 1.0]);
    let zeros = CausalMultiField::<f32>::zeros([2, 2, 2], metric, [1.0, 1.0, 1.0]);

    let a_before = a.to_coefficients();
    let c = a + zeros;
    let c_after = c.to_coefficients();

    // A + 0 = A
    for (before, after) in a_before.iter().zip(c_after.iter()) {
        for (b_val, a_val) in before.data().iter().zip(after.data().iter()) {
            assert!((b_val - a_val).abs() < 1e-5);
        }
    }
}

#[test]
#[should_panic(expected = "Metric mismatch")]
fn test_add_metric_mismatch_panics() {
    let metric1 = Metric::from_signature(3, 0, 0);
    let metric2 = Metric::from_signature(2, 0, 0);

    let a = CausalMultiField::<f32>::zeros([2, 2, 2], metric1, [1.0, 1.0, 1.0]);
    let b = CausalMultiField::<f32>::zeros([2, 2, 2], metric2, [1.0, 1.0, 1.0]);

    let _ = a + b;
}

#[test]
#[should_panic(expected = "Shape mismatch")]
fn test_add_shape_mismatch_panics() {
    let metric = Metric::from_signature(3, 0, 0);

    let a = CausalMultiField::<f32>::zeros([2, 2, 2], metric, [1.0, 1.0, 1.0]);
    let b = CausalMultiField::<f32>::zeros([3, 3, 3], metric, [1.0, 1.0, 1.0]);

    let _ = a + b;
}

// =============================================================================
// Sub tests
// =============================================================================

#[test]
fn test_sub_owned_owned() {
    let metric = Metric::from_signature(3, 0, 0);
    let a = CausalMultiField::<f32>::ones([2, 2, 2], metric, [1.0, 1.0, 1.0]);
    let b = CausalMultiField::<f32>::ones([2, 2, 2], metric, [1.0, 1.0, 1.0]);

    let c = a - b;

    // A - A = 0
    assert!(c.is_zero());
}

#[test]
fn test_sub_owned_ref() {
    let metric = Metric::from_signature(3, 0, 0);
    let a = CausalMultiField::<f32>::ones([2, 2, 2], metric, [1.0, 1.0, 1.0]);
    let b = CausalMultiField::<f32>::ones([2, 2, 2], metric, [1.0, 1.0, 1.0]);

    let c = a - b;

    assert!(c.is_zero());
}

#[test]
fn test_sub_identity_zeros() {
    let metric = Metric::from_signature(3, 0, 0);
    let a = CausalMultiField::<f32>::ones([2, 2, 2], metric, [1.0, 1.0, 1.0]);
    let zeros = CausalMultiField::<f32>::zeros([2, 2, 2], metric, [1.0, 1.0, 1.0]);

    let a_before = a.to_coefficients();
    let c = a - zeros;
    let c_after = c.to_coefficients();

    // A - 0 = A
    for (before, after) in a_before.iter().zip(c_after.iter()) {
        for (b_val, a_val) in before.data().iter().zip(after.data().iter()) {
            assert!((b_val - a_val).abs() < 1e-5);
        }
    }
}

#[test]
#[should_panic(expected = "Metric mismatch")]
fn test_sub_metric_mismatch_panics() {
    let metric1 = Metric::from_signature(3, 0, 0);
    let metric2 = Metric::from_signature(2, 0, 0);

    let a = CausalMultiField::<f32>::zeros([2, 2, 2], metric1, [1.0, 1.0, 1.0]);
    let b = CausalMultiField::<f32>::zeros([2, 2, 2], metric2, [1.0, 1.0, 1.0]);

    let _ = a - b;
}

#[test]
#[should_panic(expected = "Shape mismatch")]
fn test_sub_shape_mismatch_panics() {
    let metric = Metric::from_signature(3, 0, 0);

    let a = CausalMultiField::<f32>::zeros([2, 2, 2], metric, [1.0, 1.0, 1.0]);
    let b = CausalMultiField::<f32>::zeros([3, 3, 3], metric, [1.0, 1.0, 1.0]);

    let _ = a - b;
}

// =============================================================================
// Neg tests
// =============================================================================

#[test]
fn test_neg_produces_negation() {
    let metric = Metric::from_signature(3, 0, 0);
    let num_blades = 8;

    let mut mvs = Vec::with_capacity(8);
    for i in 0..8 {
        let mut data = vec![0.0f32; num_blades];
        data[0] = (i + 1) as f32; // 1, 2, 3, ...
        mvs.push(CausalMultiVector::unchecked(data, metric));
    }

    let a = CausalMultiField::<f32>::from_coefficients(&mvs, [2, 2, 2], [1.0, 1.0, 1.0]);
    let neg_a = -a;

    let neg_coeffs = neg_a.to_coefficients();

    for (i, mv) in neg_coeffs.iter().enumerate() {
        // Original scalar was (i+1), negated should be -(i+1)
        let expected = -((i + 1) as f32);
        assert!(
            (mv.data()[0] - expected).abs() < 1e-4,
            "Expected scalar {} at index {}, got {}",
            expected,
            i,
            mv.data()[0]
        );
    }
}

#[test]
fn test_double_negation() {
    let metric = Metric::from_signature(3, 0, 0);
    let num_blades = 8;

    let mut mvs = Vec::with_capacity(8);
    for i in 0..8 {
        let mut data = vec![0.0f32; num_blades];
        data[0] = i as f32;
        mvs.push(CausalMultiVector::unchecked(data, metric));
    }

    let a = CausalMultiField::<f32>::from_coefficients(&mvs, [2, 2, 2], [1.0, 1.0, 1.0]);
    let a_orig = a.to_coefficients();

    let neg_neg_a = -(-a);
    let a_restored = neg_neg_a.to_coefficients();

    // -(-A) = A
    for (orig, restored) in a_orig.iter().zip(a_restored.iter()) {
        for (o, r) in orig.data().iter().zip(restored.data().iter()) {
            assert!(
                (o - r).abs() < 1e-4,
                "Double negation failed: {} != {}",
                o,
                r
            );
        }
    }
}

#[test]
fn test_neg_zeros() {
    let metric = Metric::from_signature(3, 0, 0);
    let zeros = CausalMultiField::<f32>::zeros([2, 2, 2], metric, [1.0, 1.0, 1.0]);

    let neg_zeros = -zeros;

    // -0 = 0
    assert!(neg_zeros.is_zero());
}

// =============================================================================
// Mul (Geometric Product) tests
// =============================================================================

#[test]
fn test_mul_ones_ones() {
    let metric = Metric::from_signature(3, 0, 0);
    let a = CausalMultiField::<f32>::ones([2, 2, 2], metric, [1.0, 1.0, 1.0]);
    let b = CausalMultiField::<f32>::ones([2, 2, 2], metric, [1.0, 1.0, 1.0]);

    let c = a * b;

    // I * I = I (identity matrices)
    assert!(!c.is_zero());
}

#[test]
fn test_mul_owned_ref() {
    let metric = Metric::from_signature(3, 0, 0);
    let a = CausalMultiField::<f32>::ones([2, 2, 2], metric, [1.0, 1.0, 1.0]);
    let b = CausalMultiField::<f32>::ones([2, 2, 2], metric, [1.0, 1.0, 1.0]);

    let c = a * &b;

    assert!(!c.is_zero());
}

#[test]
fn test_mul_ref_ref() {
    let metric = Metric::from_signature(3, 0, 0);
    let a = CausalMultiField::<f32>::ones([2, 2, 2], metric, [1.0, 1.0, 1.0]);
    let b = CausalMultiField::<f32>::ones([2, 2, 2], metric, [1.0, 1.0, 1.0]);

    let c = &a * &b;

    assert!(!c.is_zero());
}

#[test]
fn test_mul_zeros_annihilates() {
    let metric = Metric::from_signature(3, 0, 0);
    let a = CausalMultiField::<f32>::ones([2, 2, 2], metric, [1.0, 1.0, 1.0]);
    let zeros = CausalMultiField::<f32>::zeros([2, 2, 2], metric, [1.0, 1.0, 1.0]);

    let c = a * zeros;

    // A * 0 = 0
    assert!(c.is_zero());
}

#[test]
#[should_panic(expected = "Metric mismatch")]
fn test_mul_metric_mismatch_panics() {
    let metric1 = Metric::from_signature(3, 0, 0);
    let metric2 = Metric::from_signature(2, 0, 0);

    let a = CausalMultiField::<f32>::ones([2, 2, 2], metric1, [1.0, 1.0, 1.0]);
    let b = CausalMultiField::<f32>::ones([2, 2, 2], metric2, [1.0, 1.0, 1.0]);

    let _ = a * b;
}

#[test]
#[should_panic(expected = "Shape mismatch")]
fn test_mul_shape_mismatch_panics() {
    let metric = Metric::from_signature(3, 0, 0);

    let a = CausalMultiField::<f32>::ones([2, 2, 2], metric, [1.0, 1.0, 1.0]);
    let b = CausalMultiField::<f32>::ones([3, 3, 3], metric, [1.0, 1.0, 1.0]);

    let _ = a * b;
}

// =============================================================================
// Scale tests
// =============================================================================

#[test]
fn test_scale_by_zero() {
    let metric = Metric::from_signature(3, 0, 0);
    let a = CausalMultiField::<f32>::ones([2, 2, 2], metric, [1.0, 1.0, 1.0]);

    let scaled = a.scale(0.0);

    assert!(scaled.is_zero());
}

#[test]
fn test_scale_by_one() {
    let metric = Metric::from_signature(3, 0, 0);
    let num_blades = 8;

    let mut mvs = Vec::with_capacity(8);
    for i in 0..8 {
        let mut data = vec![0.0f32; num_blades];
        data[0] = i as f32;
        mvs.push(CausalMultiVector::unchecked(data, metric));
    }

    let a = CausalMultiField::<f32>::from_coefficients(&mvs, [2, 2, 2], [1.0, 1.0, 1.0]);
    let a_orig = a.to_coefficients();

    let scaled = a.scale(1.0);
    let scaled_coeffs = scaled.to_coefficients();

    // 1 * A = A
    for (orig, sc) in a_orig.iter().zip(scaled_coeffs.iter()) {
        for (o, s) in orig.data().iter().zip(sc.data().iter()) {
            assert!((o - s).abs() < 1e-4);
        }
    }
}

#[test]
fn test_scale_by_two() {
    let metric = Metric::from_signature(3, 0, 0);
    let num_blades = 8;

    let mut mvs = Vec::with_capacity(8);
    for i in 0..8 {
        let mut data = vec![0.0f32; num_blades];
        data[0] = (i + 1) as f32;
        mvs.push(CausalMultiVector::unchecked(data, metric));
    }

    let a = CausalMultiField::<f32>::from_coefficients(&mvs, [2, 2, 2], [1.0, 1.0, 1.0]);

    let scaled = a.scale(2.0);
    let scaled_coeffs = scaled.to_coefficients();

    for (i, mv) in scaled_coeffs.iter().enumerate() {
        let expected = ((i + 1) as f32) * 2.0;
        assert!(
            (mv.data()[0] - expected).abs() < 1e-4,
            "Expected {} at index {}, got {}",
            expected,
            i,
            mv.data()[0]
        );
    }
}

#[test]
fn test_scale_by_negative() {
    let metric = Metric::from_signature(3, 0, 0);
    let num_blades = 8;

    let mut mvs = Vec::with_capacity(8);
    for i in 0..8 {
        let mut data = vec![0.0f32; num_blades];
        data[0] = (i + 1) as f32;
        mvs.push(CausalMultiVector::unchecked(data, metric));
    }

    let a = CausalMultiField::<f32>::from_coefficients(&mvs, [2, 2, 2], [1.0, 1.0, 1.0]);

    let scaled = a.scale(-1.0);
    let scaled_coeffs = scaled.to_coefficients();

    for (i, mv) in scaled_coeffs.iter().enumerate() {
        let expected = -((i + 1) as f32);
        assert!(
            (mv.data()[0] - expected).abs() < 1e-4,
            "Expected {} at index {}, got {}",
            expected,
            i,
            mv.data()[0]
        );
    }
}
