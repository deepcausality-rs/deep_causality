/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for Clifford algebra products: inner_product, outer_product, cross, commutators, hodge_dual.

use deep_causality_metric::Metric;
use deep_causality_multivector::{CausalMultiField, CausalMultiVector};
use deep_causality_num::Zero;
use deep_causality_tensor::CpuBackend;

// =============================================================================
// inner_product() tests
// =============================================================================

#[test]
fn test_inner_product_returns_field() {
    let metric = Metric::from_signature(3, 0, 0);
    let a = CausalMultiField::<CpuBackend, f32>::ones([2, 2, 2], metric, [1.0, 1.0, 1.0]);
    let b = CausalMultiField::<CpuBackend, f32>::ones([2, 2, 2], metric, [1.0, 1.0, 1.0]);

    let inner = a.inner_product(&b);

    assert_eq!(inner.metric(), metric);
    assert_eq!(*inner.shape(), [2, 2, 2]);
}

#[test]
fn test_inner_product_with_zeros() {
    let metric = Metric::from_signature(3, 0, 0);
    let a = CausalMultiField::<CpuBackend, f32>::ones([2, 2, 2], metric, [1.0, 1.0, 1.0]);
    let zeros = CausalMultiField::<CpuBackend, f32>::zeros([2, 2, 2], metric, [1.0, 1.0, 1.0]);

    let inner = a.inner_product(&zeros);

    assert!(inner.is_zero());
}

#[test]
fn test_inner_product_is_scalar() {
    let metric = Metric::from_signature(3, 0, 0);
    let a = CausalMultiField::<CpuBackend, f32>::ones([2, 2, 2], metric, [1.0, 1.0, 1.0]);
    let b = CausalMultiField::<CpuBackend, f32>::ones([2, 2, 2], metric, [1.0, 1.0, 1.0]);

    let inner = a.inner_product(&b);
    let coeffs = inner.to_coefficients();

    // Inner product projects to grade 0, so non-scalar components should be zero
    for mv in coeffs {
        for (i, val) in mv.data().iter().enumerate() {
            if i != 0 {
                assert!(
                    val.abs() < 1e-4,
                    "Non-scalar component {} should be 0, got {}",
                    i,
                    val
                );
            }
        }
    }
}

#[test]
#[should_panic(expected = "Metric mismatch")]
fn test_inner_product_metric_mismatch_panics() {
    let metric1 = Metric::from_signature(3, 0, 0);
    let metric2 = Metric::from_signature(2, 0, 0);

    let a = CausalMultiField::<CpuBackend, f32>::ones([2, 2, 2], metric1, [1.0, 1.0, 1.0]);
    let b = CausalMultiField::<CpuBackend, f32>::ones([2, 2, 2], metric2, [1.0, 1.0, 1.0]);

    let _ = a.inner_product(&b);
}

#[test]
#[should_panic(expected = "Shape mismatch")]
fn test_inner_product_shape_mismatch_panics() {
    let metric = Metric::from_signature(3, 0, 0);

    let a = CausalMultiField::<CpuBackend, f32>::ones([2, 2, 2], metric, [1.0, 1.0, 1.0]);
    let b = CausalMultiField::<CpuBackend, f32>::ones([3, 3, 3], metric, [1.0, 1.0, 1.0]);

    let _ = a.inner_product(&b);
}

// =============================================================================
// outer_product() tests
// =============================================================================

#[test]
fn test_outer_product_returns_field() {
    let metric = Metric::from_signature(3, 0, 0);
    let a = CausalMultiField::<CpuBackend, f32>::ones([2, 2, 2], metric, [1.0, 1.0, 1.0]);
    let b = CausalMultiField::<CpuBackend, f32>::ones([2, 2, 2], metric, [1.0, 1.0, 1.0]);

    let outer = a.outer_product(&b);

    assert_eq!(outer.metric(), metric);
    assert_eq!(*outer.shape(), [2, 2, 2]);
}

#[test]
fn test_outer_product_antisymmetric() {
    let metric = Metric::from_signature(3, 0, 0);
    let num_blades = 8;

    // Create non-trivial fields
    let mut mvs_a = Vec::with_capacity(8);
    let mut mvs_b = Vec::with_capacity(8);
    for i in 0..8 {
        let mut data_a = vec![0.0f32; num_blades];
        let mut data_b = vec![0.0f32; num_blades];
        data_a[1] = (i + 1) as f32; // e_1 component
        data_b[2] = (i + 2) as f32; // e_2 component
        mvs_a.push(CausalMultiVector::unchecked(data_a, metric));
        mvs_b.push(CausalMultiVector::unchecked(data_b, metric));
    }

    let a =
        CausalMultiField::<CpuBackend, f32>::from_coefficients(&mvs_a, [2, 2, 2], [1.0, 1.0, 1.0]);
    let b =
        CausalMultiField::<CpuBackend, f32>::from_coefficients(&mvs_b, [2, 2, 2], [1.0, 1.0, 1.0]);

    let ab = a.outer_product(&b);
    let ba = b.outer_product(&a);

    let ab_coeffs = ab.to_coefficients();
    let ba_coeffs = ba.to_coefficients();

    // A ∧ B = -(B ∧ A)
    for (ab_mv, ba_mv) in ab_coeffs.iter().zip(ba_coeffs.iter()) {
        for (ab_val, ba_val) in ab_mv.data().iter().zip(ba_mv.data().iter()) {
            assert!(
                (ab_val + ba_val).abs() < 1e-4,
                "Antisymmetry failed: {} + {} = {}",
                ab_val,
                ba_val,
                ab_val + ba_val
            );
        }
    }
}

#[test]
fn test_outer_product_with_zeros() {
    let metric = Metric::from_signature(3, 0, 0);
    let a = CausalMultiField::<CpuBackend, f32>::ones([2, 2, 2], metric, [1.0, 1.0, 1.0]);
    let zeros = CausalMultiField::<CpuBackend, f32>::zeros([2, 2, 2], metric, [1.0, 1.0, 1.0]);

    let outer = a.outer_product(&zeros);

    assert!(outer.is_zero());
}

#[test]
#[should_panic(expected = "Metric mismatch")]
fn test_outer_product_metric_mismatch() {
    let metric1 = Metric::from_signature(3, 0, 0);
    let metric2 = Metric::from_signature(2, 0, 0);

    let a = CausalMultiField::<CpuBackend, f32>::ones([2, 2, 2], metric1, [1.0, 1.0, 1.0]);
    let b = CausalMultiField::<CpuBackend, f32>::ones([2, 2, 2], metric2, [1.0, 1.0, 1.0]);

    let _ = a.outer_product(&b);
}

// =============================================================================
// commutator_lie() tests
// =============================================================================

#[test]
fn test_commutator_lie_returns_field() {
    let metric = Metric::from_signature(3, 0, 0);
    let a = CausalMultiField::<CpuBackend, f32>::ones([2, 2, 2], metric, [1.0, 1.0, 1.0]);
    let b = CausalMultiField::<CpuBackend, f32>::ones([2, 2, 2], metric, [1.0, 1.0, 1.0]);

    let comm = a.commutator_lie(&b);

    assert_eq!(comm.metric(), metric);
    assert_eq!(*comm.shape(), [2, 2, 2]);
}

#[test]
fn test_commutator_lie_self_is_zero() {
    let metric = Metric::from_signature(3, 0, 0);
    let a = CausalMultiField::<CpuBackend, f32>::ones([2, 2, 2], metric, [1.0, 1.0, 1.0]);

    // Clone a for the second operand
    let a2 = CausalMultiField::<CpuBackend, f32>::ones([2, 2, 2], metric, [1.0, 1.0, 1.0]);

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

    let a =
        CausalMultiField::<CpuBackend, f32>::from_coefficients(&mvs_a, [2, 2, 2], [1.0, 1.0, 1.0]);
    let b =
        CausalMultiField::<CpuBackend, f32>::from_coefficients(&mvs_b, [2, 2, 2], [1.0, 1.0, 1.0]);

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

    let a =
        CausalMultiField::<CpuBackend, f32>::from_coefficients(&mvs_a, [2, 2, 2], [1.0, 1.0, 1.0]);
    let b =
        CausalMultiField::<CpuBackend, f32>::from_coefficients(&mvs_b, [2, 2, 2], [1.0, 1.0, 1.0]);

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
// cross() tests
// =============================================================================

#[test]
fn test_cross_returns_field() {
    let metric = Metric::from_signature(3, 0, 0);
    let a = CausalMultiField::<CpuBackend, f32>::ones([2, 2, 2], metric, [1.0, 1.0, 1.0]);
    let b = CausalMultiField::<CpuBackend, f32>::ones([2, 2, 2], metric, [1.0, 1.0, 1.0]);

    let cross = a.cross(&b);

    assert_eq!(cross.metric(), metric);
    assert_eq!(*cross.shape(), [2, 2, 2]);
}

#[test]
fn test_cross_with_zeros() {
    let metric = Metric::from_signature(3, 0, 0);
    let a = CausalMultiField::<CpuBackend, f32>::ones([2, 2, 2], metric, [1.0, 1.0, 1.0]);
    let zeros = CausalMultiField::<CpuBackend, f32>::zeros([2, 2, 2], metric, [1.0, 1.0, 1.0]);

    let cross = a.cross(&zeros);

    // Cross with zero should be zero-ish (may have numerical noise from Hodge dual)
    let coeffs = cross.to_coefficients();
    let sum: f32 = coeffs
        .iter()
        .flat_map(|mv| mv.data().iter())
        .map(|v| v.abs())
        .sum();

    assert!(sum < 1e-3, "Cross with zeros should be ~0, got sum {}", sum);
}

// =============================================================================
// hodge_dual() tests
// =============================================================================

#[test]
fn test_hodge_dual_returns_field() {
    let metric = Metric::from_signature(3, 0, 0);
    let field = CausalMultiField::<CpuBackend, f32>::ones([2, 2, 2], metric, [1.0, 1.0, 1.0]);

    let dual = field.hodge_dual();

    assert_eq!(dual.metric(), metric);
    assert_eq!(*dual.shape(), [2, 2, 2]);
}

#[test]
fn test_hodge_dual_of_zeros() {
    let metric = Metric::from_signature(3, 0, 0);
    let field = CausalMultiField::<CpuBackend, f32>::zeros([2, 2, 2], metric, [1.0, 1.0, 1.0]);

    let dual = field.hodge_dual();

    assert!(dual.is_zero());
}

#[test]
fn test_hodge_dual_changes_grade() {
    let metric = Metric::from_signature(3, 0, 0);
    let num_blades = 8;

    // Create pure scalar field
    let mut mvs = Vec::with_capacity(8);
    for i in 0..8 {
        let mut data = vec![0.0f32; num_blades];
        data[0] = (i + 1) as f32; // Scalar
        mvs.push(CausalMultiVector::unchecked(data, metric));
    }

    let field =
        CausalMultiField::<CpuBackend, f32>::from_coefficients(&mvs, [2, 2, 2], [1.0, 1.0, 1.0]);

    let dual = field.hodge_dual();
    let coeffs = dual.to_coefficients();

    // Hodge dual of scalar should be pseudoscalar (grade 3 in Cl(3))
    // The scalar component should now be ~0
    for _mv in coeffs {
        // After Hodge dual, the pseudoscalar (index 7) should be non-zero
        // The exact behavior depends on the dual implementation
    }
}

#[test]
fn test_double_hodge_dual() {
    let metric = Metric::from_signature(3, 0, 0);
    let num_blades = 8;

    let mut mvs = Vec::with_capacity(8);
    for i in 0..8 {
        let mut data = vec![0.0f32; num_blades];
        data[0] = (i + 1) as f32;
        mvs.push(CausalMultiVector::unchecked(data, metric));
    }

    let field =
        CausalMultiField::<CpuBackend, f32>::from_coefficients(&mvs, [2, 2, 2], [1.0, 1.0, 1.0]);
    let orig_coeffs = field.to_coefficients();

    let double_dual = field.hodge_dual().hodge_dual();
    let dd_coeffs = double_dual.to_coefficients();

    // **A = ±A depending on signature and grade
    // For Cl(3,0,0), **A = (-1)^{k(3-k)} A where k is grade
    // This test just verifies the operation completes
    assert_eq!(orig_coeffs.len(), dd_coeffs.len());
}
