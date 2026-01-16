/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for CausalMultiField algebra operations.
//!
//! These tests cover:
//! - scale() - Scalar multiplication
//! - normalize() - Unit normalization
//! - inverse() - Multiplicative inversion
//! - reversion() - Grade reversion
//! - squared_magnitude() - Magnitude computation
//! - commutator_lie() - Lie bracket [A,B] = AB - BA
//! - commutator_geometric() - GA commutator (AB - BA)/2
use deep_causality_metric::Metric;
use deep_causality_multivector::{CausalMultiField, CausalMultiVector};
use deep_causality_num::Zero;

// =============================================================================
// scale() tests
// =============================================================================

#[test]
fn test_scale_by_zero() {
    let metric = Metric::from_signature(3, 0, 0);
    let field = CausalMultiField::<f32>::ones([2, 2, 2], metric, [1.0, 1.0, 1.0]);

    let scaled = field.scale(0.0);

    assert!(scaled.is_zero());
}

#[test]
fn test_scale_by_one() {
    let metric = Metric::from_signature(3, 0, 0);
    let num_blades = 8;

    let mut mvs = Vec::with_capacity(8);
    for i in 0..8 {
        let mut data = vec![0.0f32; num_blades];
        data[0] = (i + 1) as f32;
        mvs.push(CausalMultiVector::unchecked(data, metric));
    }

    let field = CausalMultiField::<f32>::from_coefficients(&mvs, [2, 2, 2], [1.0, 1.0, 1.0]);
    let scaled = field.scale(1.0);

    let orig_coeffs = field.to_coefficients();
    let scaled_coeffs = scaled.to_coefficients();

    for (orig, scaled_mv) in orig_coeffs.iter().zip(scaled_coeffs.iter()) {
        for (o, s) in orig.data().iter().zip(scaled_mv.data().iter()) {
            assert!((o - s).abs() < 1e-6, "Scale by 1 should preserve values");
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

    let field = CausalMultiField::<f32>::from_coefficients(&mvs, [2, 2, 2], [1.0, 1.0, 1.0]);
    let scaled = field.scale(2.0);

    let orig_coeffs = field.to_coefficients();
    let scaled_coeffs = scaled.to_coefficients();

    for (orig, scaled_mv) in orig_coeffs.iter().zip(scaled_coeffs.iter()) {
        for (o, s) in orig.data().iter().zip(scaled_mv.data().iter()) {
            assert!(
                (2.0 * o - s).abs() < 1e-5,
                "Scale by 2 failed: {} * 2 != {}",
                o,
                s
            );
        }
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

    let field = CausalMultiField::<f32>::from_coefficients(&mvs, [2, 2, 2], [1.0, 1.0, 1.0]);
    let scaled = field.scale(-1.0);

    let orig_coeffs = field.to_coefficients();
    let scaled_coeffs = scaled.to_coefficients();

    for (orig, scaled_mv) in orig_coeffs.iter().zip(scaled_coeffs.iter()) {
        for (o, s) in orig.data().iter().zip(scaled_mv.data().iter()) {
            assert!(
                (o + s).abs() < 1e-5,
                "Scale by -1 failed: {} + {} != 0",
                o,
                s
            );
        }
    }
}

// =============================================================================
// commutator_lie() tests
// =============================================================================

#[test]
fn test_commutator_lie_returns_field() {
    let metric = Metric::from_signature(3, 0, 0);
    let a = CausalMultiField::<f32>::ones([2, 2, 2], metric, [1.0, 1.0, 1.0]);
    let b = CausalMultiField::<f32>::ones([2, 2, 2], metric, [1.0, 1.0, 1.0]);

    let comm = a.commutator_lie(&b);

    assert_eq!(comm.metric(), metric);
    assert_eq!(*comm.shape(), [2, 2, 2]);
}

#[test]
fn test_commutator_lie_self_is_zero() {
    let metric = Metric::from_signature(3, 0, 0);
    let a = CausalMultiField::<f32>::ones([2, 2, 2], metric, [1.0, 1.0, 1.0]);
    let a2 = CausalMultiField::<f32>::ones([2, 2, 2], metric, [1.0, 1.0, 1.0]);

    let comm = a.commutator_lie(&a2);

    // [A, A] = AA - AA = 0
    assert!(comm.is_zero());
}

#[test]
fn test_commutator_lie_antisymmetric() {
    let metric = Metric::from_signature(3, 0, 0);
    let num_blades = 8;

    let mut mvs_a = Vec::with_capacity(8);
    let mut mvs_b = Vec::with_capacity(8);
    for i in 0..8 {
        let mut data_a = vec![0.0f32; num_blades];
        let mut data_b = vec![0.0f32; num_blades];
        data_a[0] = (i + 1) as f32;
        data_b[1] = (i + 2) as f32;
        mvs_a.push(CausalMultiVector::unchecked(data_a, metric));
        mvs_b.push(CausalMultiVector::unchecked(data_b, metric));
    }

    let a = CausalMultiField::<f32>::from_coefficients(&mvs_a, [2, 2, 2], [1.0, 1.0, 1.0]);
    let b = CausalMultiField::<f32>::from_coefficients(&mvs_b, [2, 2, 2], [1.0, 1.0, 1.0]);

    let ab = a.commutator_lie(&b);
    let ba = b.commutator_lie(&a);

    let ab_coeffs = ab.to_coefficients();
    let ba_coeffs = ba.to_coefficients();

    // [A,B] = -[B,A]
    for (ab_mv, ba_mv) in ab_coeffs.iter().zip(ba_coeffs.iter()) {
        for (ab_val, ba_val) in ab_mv.data().iter().zip(ba_mv.data().iter()) {
            assert!(
                (ab_val + ba_val).abs() < 1e-4,
                "Antisymmetry failed: {} + {} != 0",
                ab_val,
                ba_val
            );
        }
    }
}

#[test]
#[should_panic(expected = "Metric mismatch")]
fn test_commutator_lie_metric_mismatch() {
    let metric1 = Metric::from_signature(3, 0, 0);
    let metric2 = Metric::from_signature(2, 0, 0);

    let a = CausalMultiField::<f32>::ones([2, 2, 2], metric1, [1.0, 1.0, 1.0]);
    let b = CausalMultiField::<f32>::ones([2, 2, 2], metric2, [1.0, 1.0, 1.0]);

    let _ = a.commutator_lie(&b);
}

// =============================================================================
// commutator_geometric() tests
// =============================================================================

#[test]
fn test_commutator_geometric_is_half_lie() {
    let metric = Metric::from_signature(3, 0, 0);
    let num_blades = 8;

    let mut mvs_a = Vec::with_capacity(8);
    let mut mvs_b = Vec::with_capacity(8);
    for i in 0..8 {
        let mut data_a = vec![0.0f32; num_blades];
        let mut data_b = vec![0.0f32; num_blades];
        data_a[1] = (i + 1) as f32;
        data_b[2] = (i + 2) as f32;
        mvs_a.push(CausalMultiVector::unchecked(data_a, metric));
        mvs_b.push(CausalMultiVector::unchecked(data_b, metric));
    }

    let a = CausalMultiField::<f32>::from_coefficients(&mvs_a, [2, 2, 2], [1.0, 1.0, 1.0]);
    let b = CausalMultiField::<f32>::from_coefficients(&mvs_b, [2, 2, 2], [1.0, 1.0, 1.0]);

    let lie = a.commutator_lie(&b);
    let geo = a.commutator_geometric(&b);

    let lie_coeffs = lie.to_coefficients();
    let geo_coeffs = geo.to_coefficients();

    // geo = lie / 2
    for (lie_mv, geo_mv) in lie_coeffs.iter().zip(geo_coeffs.iter()) {
        for (lie_val, geo_val) in lie_mv.data().iter().zip(geo_mv.data().iter()) {
            let expected = lie_val / 2.0;
            assert!(
                (expected - geo_val).abs() < 1e-4,
                "Geometric should be Lie/2: {} vs {}",
                expected,
                geo_val
            );
        }
    }
}

// =============================================================================
// normalize() tests
// =============================================================================

#[test]
fn test_normalize_returns_field() {
    let metric = Metric::from_signature(3, 0, 0);
    let num_blades = 8;

    // Create field with non-zero entries
    let mut mvs = Vec::with_capacity(8);
    for i in 0..8 {
        let mut data = vec![0.0f32; num_blades];
        data[0] = (i + 1) as f32; // Scalar component
        mvs.push(CausalMultiVector::unchecked(data, metric));
    }

    let field = CausalMultiField::<f32>::from_coefficients(&mvs, [2, 2, 2], [1.0, 1.0, 1.0]);
    let normalized = field.normalize();

    assert_eq!(normalized.metric(), metric);
    assert_eq!(*normalized.shape(), [2, 2, 2]);
}

#[test]
fn test_normalize_unit_magnitude() {
    let metric = Metric::from_signature(2, 0, 0); // Euclidean 2D
    let num_blades = 4;

    // Create field with vectors of varying magnitudes
    let mut mvs = Vec::with_capacity(8);
    for i in 0..8 {
        let mut data = vec![0.0f32; num_blades];
        // Pure e1 component with magnitude (i+1)
        data[1] = (i + 1) as f32;
        mvs.push(CausalMultiVector::unchecked(data, metric));
    }

    let field = CausalMultiField::<f32>::from_coefficients(&mvs, [2, 2, 2], [1.0, 1.0, 1.0]);
    let normalized = field.normalize();

    // Verify global normalization
    // squared_magnitude() sums over all elements
    let mag_sq = normalized.squared_magnitude();
    assert!(
        (mag_sq - 1.0).abs() < 1e-4,
        "Normalized field should have unit global magnitude, got {}",
        mag_sq
    );
}

// =============================================================================
// inverse() tests
// =============================================================================

#[test]
fn test_inverse_returns_field() {
    let metric = Metric::from_signature(2, 0, 0);
    let num_blades = 4;

    // Create field with non-zero scalars
    let mut mvs = Vec::with_capacity(8);
    for i in 0..8 {
        let mut data = vec![0.0f32; num_blades];
        data[0] = (i + 1) as f32; // Scalar
        mvs.push(CausalMultiVector::unchecked(data, metric));
    }

    let field = CausalMultiField::<f32>::from_coefficients(&mvs, [2, 2, 2], [1.0, 1.0, 1.0]);
    let inverse = field.inverse();

    assert_eq!(inverse.metric(), metric);
    assert_eq!(*inverse.shape(), [2, 2, 2]);
}

#[test]
fn test_inverse_of_scalar() {
    let metric = Metric::from_signature(2, 0, 0);
    let num_blades = 4;

    // Create field with scalar = 2
    let mut mvs = Vec::with_capacity(8);
    for _ in 0..8 {
        let mut data = vec![0.0f32; num_blades];
        data[0] = 2.0;
        mvs.push(CausalMultiVector::unchecked(data, metric));
    }

    let field = CausalMultiField::<f32>::from_coefficients(&mvs, [2, 2, 2], [1.0, 1.0, 1.0]);
    let inverse = field.inverse();
    let inv_coeffs = inverse.to_coefficients();

    // Inverse of scalar 2 should be 0.5
    for mv in &inv_coeffs {
        let scalar = mv.data()[0];
        assert!(
            (scalar - 0.5).abs() < 1e-4,
            "Inverse of 2 should be 0.5, got {}",
            scalar
        );
    }
}

#[test]
fn test_inverse_product_is_identity() {
    let metric = Metric::from_signature(2, 0, 0);
    let num_blades = 4;

    // Create field with a proper versor (unit vector e1)
    // In Euclidean space, e1 is its own inverse: e1 * e1 = 1
    let mut mvs = Vec::with_capacity(8);
    for _ in 0..8 {
        let mut data = vec![0.0f32; num_blades];
        // Use unit vector e1 which is a versor (invertible)
        data[1] = 1.0; // e1 component
        mvs.push(CausalMultiVector::unchecked(data, metric));
    }

    let field = CausalMultiField::<f32>::from_coefficients(&mvs, [2, 2, 2], [1.0, 1.0, 1.0]);
    let inverse = field.inverse();

    // A * A^{-1} should be identity (scalar = 1)
    let product = field * inverse;
    let prod_coeffs = product.to_coefficients();

    for mv in &prod_coeffs {
        let scalar = mv.data()[0];
        assert!(
            (scalar - 1.0).abs() < 1e-3,
            "A * A^-1 scalar part should be 1, got {}",
            scalar
        );

        // Check that other components are near zero
        for (i, &val) in mv.data().iter().enumerate().skip(1) {
            assert!(
                val.abs() < 1e-3,
                "Non-scalar part of identity should be zero at index {}, got {}",
                i,
                val
            );
        }
    }
}

// =============================================================================
// reversion() tests
// =============================================================================

#[test]
fn test_reversion_returns_field() {
    let metric = Metric::from_signature(3, 0, 0);
    let field = CausalMultiField::<f32>::ones([2, 2, 2], metric, [1.0, 1.0, 1.0]);

    let rev = field.reversion();

    assert_eq!(rev.metric(), metric);
    assert_eq!(*rev.shape(), [2, 2, 2]);
}

#[test]
fn test_reversion_of_scalar_unchanged() {
    let metric = Metric::from_signature(2, 0, 0);
    let num_blades = 4;

    let mut mvs = Vec::with_capacity(8);
    for i in 0..8 {
        let mut data = vec![0.0f32; num_blades];
        data[0] = (i + 1) as f32; // Pure scalar
        mvs.push(CausalMultiVector::unchecked(data, metric));
    }

    let field = CausalMultiField::<f32>::from_coefficients(&mvs, [2, 2, 2], [1.0, 1.0, 1.0]);
    let rev = field.reversion();

    let orig_coeffs = field.to_coefficients();
    let rev_coeffs = rev.to_coefficients();

    // Scalars are unchanged by reversion
    for (orig, rev_mv) in orig_coeffs.iter().zip(rev_coeffs.iter()) {
        assert!(
            (orig.data()[0] - rev_mv.data()[0]).abs() < 1e-6,
            "Scalar should be unchanged by reversion"
        );
    }
}

#[test]
fn test_reversion_of_vector_unchanged() {
    let metric = Metric::from_signature(2, 0, 0);
    let num_blades = 4;

    let mut mvs = Vec::with_capacity(8);
    for i in 0..8 {
        let mut data = vec![0.0f32; num_blades];
        data[1] = (i + 1) as f32; // Pure e1
        mvs.push(CausalMultiVector::unchecked(data, metric));
    }

    let field = CausalMultiField::<f32>::from_coefficients(&mvs, [2, 2, 2], [1.0, 1.0, 1.0]);
    let rev = field.reversion();

    let orig_coeffs = field.to_coefficients();
    let rev_coeffs = rev.to_coefficients();

    // Grade-1 elements are unchanged by reversion
    for (orig, rev_mv) in orig_coeffs.iter().zip(rev_coeffs.iter()) {
        assert!(
            (orig.data()[1] - rev_mv.data()[1]).abs() < 1e-6,
            "Vector should be unchanged by reversion"
        );
    }
}

#[test]
fn test_reversion_of_bivector_negated() {
    let metric = Metric::from_signature(2, 0, 0);
    let num_blades = 4;

    let mut mvs = Vec::with_capacity(8);
    for i in 0..8 {
        let mut data = vec![0.0f32; num_blades];
        data[3] = (i + 1) as f32; // Pure e12 (bivector, index 3 = 0b11)
        mvs.push(CausalMultiVector::unchecked(data, metric));
    }

    let field = CausalMultiField::<f32>::from_coefficients(&mvs, [2, 2, 2], [1.0, 1.0, 1.0]);
    let rev = field.reversion();

    let orig_coeffs = field.to_coefficients();
    let rev_coeffs = rev.to_coefficients();

    // Grade-2 elements are negated by reversion: ~(e1e2) = e2e1 = -e1e2
    for (orig, rev_mv) in orig_coeffs.iter().zip(rev_coeffs.iter()) {
        assert!(
            (orig.data()[3] + rev_mv.data()[3]).abs() < 1e-6,
            "Bivector should be negated by reversion: {} vs {}",
            orig.data()[3],
            rev_mv.data()[3]
        );
    }
}

// =============================================================================
// squared_magnitude() tests
// =============================================================================

#[test]
fn test_squared_magnitude_of_zeros() {
    let metric = Metric::from_signature(3, 0, 0);
    let field = CausalMultiField::<f32>::zeros([2, 2, 2], metric, [1.0, 1.0, 1.0]);

    assert_eq!(field.squared_magnitude(), 0.0);
}

#[test]
fn test_squared_magnitude_single_element() {
    let metric = Metric::from_signature(2, 0, 0);
    let num_blades = 4;
    let shape = [1, 1, 1]; // Single cell

    let mut data = vec![0.0f32; num_blades];
    data[0] = 3.0; // Scalar = 3
    let mv = CausalMultiVector::unchecked(data, metric);

    let field = CausalMultiField::<f32>::from_coefficients(&[mv], shape, [1.0, 1.0, 1.0]);

    // Matrix representation of scalar 3 in Cl(2) is 3*I (2x2)
    // Frobenius norm squared: 3^2 + 0 + 0 + 3^2 = 9 + 9 = 18
    assert!((field.squared_magnitude() - 18.0).abs() < 1e-5);
}

#[test]
fn test_squared_magnitude_scaling() {
    let metric = Metric::from_signature(2, 0, 0);
    let shape = [1, 1, 1];
    let num_blades = 4;
    let mut data = vec![0.0f32; num_blades];
    data[0] = 1.0;

    let mv = CausalMultiVector::unchecked(data, metric);
    let field = CausalMultiField::<f32>::from_coefficients(&[mv], shape, [1.0, 1.0, 1.0]);

    let mag_1 = field.squared_magnitude();

    let field_2 = field.scale(2.0);
    let mag_2 = field_2.squared_magnitude();

    // ||2x||^2 = 4 * ||x||^2
    assert!((mag_2 - 4.0 * mag_1).abs() < 1e-5);
}

#[test]
fn test_squared_magnitude_field_sum() {
    let metric = Metric::from_signature(2, 0, 0);
    // 2 cells
    let shape = [2, 1, 1];
    let num_blades = 4;

    let mut mvs = Vec::with_capacity(2);
    // Cell 0: Scalar 1 -> MagSq = 1^2+1^2 = 2
    let mut d0 = vec![0.0f32; num_blades];
    d0[0] = 1.0;
    mvs.push(CausalMultiVector::unchecked(d0, metric));

    // Cell 1: Scalar 2 -> MagSq = 2^2+2^2 = 8
    let mut d1 = vec![0.0f32; num_blades];
    d1[0] = 2.0;
    mvs.push(CausalMultiVector::unchecked(d1, metric));

    let field = CausalMultiField::<f32>::from_coefficients(&mvs, shape, [1.0, 1.0, 1.0]);

    let mag = field.squared_magnitude();

    // Total = 2 + 8 = 10
    assert!((mag - 10.0).abs() < 1e-5, "Expected 10.0, got {}", mag);
}
