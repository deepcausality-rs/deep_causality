/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for grade projection operations: grade_project, scalar_part, vector_part, etc.

use deep_causality_metric::Metric;
use deep_causality_multivector::{CausalMultiField, CausalMultiVector};
use deep_causality_tensor::CpuBackend;

// =============================================================================
// grade_project() tests
// =============================================================================

#[test]
fn test_grade_project_0_extracts_scalar() {
    let metric = Metric::from_signature(3, 0, 0);
    let num_blades = 8;

    // Create field with mixed grades
    let mut mvs = Vec::with_capacity(8);
    for i in 0..8 {
        let mut data = vec![0.0f32; num_blades];
        data[0] = i as f32; // Scalar
        data[1] = 1.0; // Vector e_1
        data[3] = 2.0; // Bivector e_12
        mvs.push(CausalMultiVector::unchecked(data, metric));
    }

    let field =
        CausalMultiField::<CpuBackend, f32>::from_coefficients(&mvs, [2, 2, 2], [1.0, 1.0, 1.0]);

    let projected = field.grade_project(0);
    let coeffs = projected.to_coefficients();

    for (i, mv) in coeffs.iter().enumerate() {
        // Scalar should be preserved
        assert!(
            (mv.data()[0] - i as f32).abs() < 1e-4,
            "Scalar should be {}, got {}",
            i,
            mv.data()[0]
        );
        // Vector component should be zero
        assert!(
            mv.data()[1].abs() < 1e-4,
            "Vector should be 0, got {}",
            mv.data()[1]
        );
        // Bivector component should be zero
        assert!(
            mv.data()[3].abs() < 1e-4,
            "Bivector should be 0, got {}",
            mv.data()[3]
        );
    }
}

#[test]
fn test_grade_project_1_extracts_vector() {
    let metric = Metric::from_signature(3, 0, 0);
    let num_blades = 8;

    let mut mvs = Vec::with_capacity(8);
    for i in 0..8 {
        let mut data = vec![0.0f32; num_blades];
        data[0] = 5.0; // Scalar
        data[1] = (i + 1) as f32; // Vector e_1
        data[2] = 2.0; // Vector e_2
        data[3] = 3.0; // Bivector e_12
        mvs.push(CausalMultiVector::unchecked(data, metric));
    }

    let field =
        CausalMultiField::<CpuBackend, f32>::from_coefficients(&mvs, [2, 2, 2], [1.0, 1.0, 1.0]);

    let projected = field.grade_project(1);
    let coeffs = projected.to_coefficients();

    for (i, mv) in coeffs.iter().enumerate() {
        // Scalar should be zero
        assert!(mv.data()[0].abs() < 1e-4);
        // Vector e_1 should be preserved
        assert!(
            (mv.data()[1] - (i + 1) as f32).abs() < 1e-4,
            "e_1 should be {}, got {}",
            i + 1,
            mv.data()[1]
        );
        // Vector e_2 should be preserved
        assert!((mv.data()[2] - 2.0).abs() < 1e-4);
        // Bivector should be zero
        assert!(mv.data()[3].abs() < 1e-4);
    }
}

#[test]
fn test_grade_project_2_extracts_bivector() {
    let metric = Metric::from_signature(3, 0, 0);
    let num_blades = 8;

    let mut mvs = Vec::with_capacity(8);
    for i in 0..8 {
        let mut data = vec![0.0f32; num_blades];
        data[0] = 1.0; // Scalar
        data[1] = 2.0; // Vector e_1
        data[3] = (i + 1) as f32; // Bivector e_12
        data[5] = 4.0; // Bivector e_13
        data[7] = 5.0; // Trivector e_123
        mvs.push(CausalMultiVector::unchecked(data, metric));
    }

    let field =
        CausalMultiField::<CpuBackend, f32>::from_coefficients(&mvs, [2, 2, 2], [1.0, 1.0, 1.0]);

    let projected = field.grade_project(2);
    let coeffs = projected.to_coefficients();

    for (i, mv) in coeffs.iter().enumerate() {
        // Scalar, vector, trivector should be zero
        assert!(mv.data()[0].abs() < 1e-4);
        assert!(mv.data()[1].abs() < 1e-4);
        assert!(mv.data()[7].abs() < 1e-4);

        // Bivectors should be preserved
        assert!(
            (mv.data()[3] - (i + 1) as f32).abs() < 1e-4,
            "e_12 should be {}, got {}",
            i + 1,
            mv.data()[3]
        );
        assert!((mv.data()[5] - 4.0).abs() < 1e-4);
    }
}

#[test]
fn test_grade_project_preserves_field_properties() {
    let metric = Metric::from_signature(3, 0, 0);
    let dx = [0.5, 1.0, 1.5];
    let shape = [2, 2, 2];
    let field = CausalMultiField::<CpuBackend, f32>::ones(shape, metric, dx);

    let projected = field.grade_project(0);

    assert_eq!(projected.metric(), metric);
    assert_eq!(*projected.shape(), shape);
    assert_eq!(*projected.dx(), dx);
}

// =============================================================================
// scalar_part() tests
// =============================================================================

#[test]
fn test_scalar_part_equivalent_to_grade_project_0() {
    let metric = Metric::from_signature(3, 0, 0);
    let num_blades = 8;

    let mut mvs = Vec::with_capacity(8);
    for i in 0..8 {
        let mut data = vec![0.0f32; num_blades];
        data[0] = i as f32;
        data[1] = 1.0;
        mvs.push(CausalMultiVector::unchecked(data, metric));
    }

    let field =
        CausalMultiField::<CpuBackend, f32>::from_coefficients(&mvs, [2, 2, 2], [1.0, 1.0, 1.0]);

    let scalar_part = field.scalar_part();
    let coeffs = scalar_part.to_coefficients();

    for mv in coeffs {
        // Only scalar should remain
        assert!(mv.data()[1].abs() < 1e-4); // Vector zeroed
    }
}

#[test]
fn test_scalar_part_of_identity() {
    let metric = Metric::from_signature(3, 0, 0);
    let field = CausalMultiField::<CpuBackend, f32>::ones([2, 2, 2], metric, [1.0, 1.0, 1.0]);

    let scalar_part = field.scalar_part();
    let coeffs = scalar_part.to_coefficients();

    for mv in coeffs {
        // Identity matrix = scalar 1
        assert!((mv.data()[0] - 1.0).abs() < 1e-4);
    }
}

// =============================================================================
// vector_part() tests
// =============================================================================

#[test]
fn test_vector_part_equivalent_to_grade_project_1() {
    let metric = Metric::from_signature(3, 0, 0);
    let num_blades = 8;

    let mut mvs = Vec::with_capacity(8);
    for _ in 0..8 {
        let mut data = vec![0.0f32; num_blades];
        data[0] = 5.0;
        data[1] = 3.0;
        data[2] = 4.0;
        mvs.push(CausalMultiVector::unchecked(data, metric));
    }

    let field =
        CausalMultiField::<CpuBackend, f32>::from_coefficients(&mvs, [2, 2, 2], [1.0, 1.0, 1.0]);

    let vector_part = field.vector_part();
    let coeffs = vector_part.to_coefficients();

    for mv in coeffs {
        assert!(mv.data()[0].abs() < 1e-4); // Scalar zeroed
        assert!((mv.data()[1] - 3.0).abs() < 1e-4);
        assert!((mv.data()[2] - 4.0).abs() < 1e-4);
    }
}

// =============================================================================
// bivector_part() tests
// =============================================================================

#[test]
fn test_bivector_part_equivalent_to_grade_project_2() {
    let metric = Metric::from_signature(3, 0, 0);
    let num_blades = 8;

    let mut mvs = Vec::with_capacity(8);
    for _ in 0..8 {
        let mut data = vec![0.0f32; num_blades];
        data[0] = 1.0;
        data[3] = 7.0; // e_12
        data[5] = 8.0; // e_13
        data[6] = 9.0; // e_23
        mvs.push(CausalMultiVector::unchecked(data, metric));
    }

    let field =
        CausalMultiField::<CpuBackend, f32>::from_coefficients(&mvs, [2, 2, 2], [1.0, 1.0, 1.0]);

    let bivector_part = field.bivector_part();
    let coeffs = bivector_part.to_coefficients();

    for mv in coeffs {
        assert!(mv.data()[0].abs() < 1e-4); // Scalar zeroed
        assert!((mv.data()[3] - 7.0).abs() < 1e-4);
        assert!((mv.data()[5] - 8.0).abs() < 1e-4);
        assert!((mv.data()[6] - 9.0).abs() < 1e-4);
    }
}

// =============================================================================
// trivector_part() tests
// =============================================================================

#[test]
fn test_trivector_part_equivalent_to_grade_project_3() {
    let metric = Metric::from_signature(3, 0, 0);
    let num_blades = 8;

    let mut mvs = Vec::with_capacity(8);
    for i in 0..8 {
        let mut data = vec![0.0f32; num_blades];
        data[0] = 1.0;
        data[7] = (i + 1) as f32; // e_123 (trivector for Cl(3))
        mvs.push(CausalMultiVector::unchecked(data, metric));
    }

    let field =
        CausalMultiField::<CpuBackend, f32>::from_coefficients(&mvs, [2, 2, 2], [1.0, 1.0, 1.0]);

    let trivector_part = field.trivector_part();
    let coeffs = trivector_part.to_coefficients();

    for (i, mv) in coeffs.iter().enumerate() {
        assert!(mv.data()[0].abs() < 1e-4); // Scalar zeroed
        assert!(
            (mv.data()[7] - (i + 1) as f32).abs() < 1e-4,
            "Trivector should be {}, got {}",
            i + 1,
            mv.data()[7]
        );
    }
}

// =============================================================================
// pseudoscalar_part() tests
// =============================================================================

#[test]
fn test_pseudoscalar_part_extracts_highest_grade() {
    let metric = Metric::from_signature(3, 0, 0);
    let num_blades = 8;

    // For Cl(3), pseudoscalar is grade 3 (e_123)
    let mut mvs = Vec::with_capacity(8);
    for i in 0..8 {
        let mut data = vec![0.0f32; num_blades];
        data[0] = 1.0;
        data[1] = 2.0;
        data[7] = (i + 1) as f32; // Pseudoscalar
        mvs.push(CausalMultiVector::unchecked(data, metric));
    }

    let field =
        CausalMultiField::<CpuBackend, f32>::from_coefficients(&mvs, [2, 2, 2], [1.0, 1.0, 1.0]);

    let pseudo_part = field.pseudoscalar_part();
    let coeffs = pseudo_part.to_coefficients();

    for (i, mv) in coeffs.iter().enumerate() {
        assert!(mv.data()[0].abs() < 1e-4); // Scalar zeroed
        assert!(mv.data()[1].abs() < 1e-4); // Vector zeroed
        assert!(
            (mv.data()[7] - (i + 1) as f32).abs() < 1e-4,
            "Pseudoscalar should be {}, got {}",
            i + 1,
            mv.data()[7]
        );
    }
}

#[test]
fn test_pseudoscalar_dimension_varies_with_metric() {
    // For Cl(2), pseudoscalar is grade 2 (e_12, index 3)
    let metric = Metric::from_signature(2, 0, 0);
    let num_blades = 4; // 2^2

    let mut mvs = Vec::with_capacity(8);
    for i in 0..8 {
        let mut data = vec![0.0f32; num_blades];
        data[0] = 1.0;
        data[3] = (i + 1) as f32; // e_12 is pseudoscalar for Cl(2)
        mvs.push(CausalMultiVector::unchecked(data, metric));
    }

    let field =
        CausalMultiField::<CpuBackend, f32>::from_coefficients(&mvs, [2, 2, 2], [1.0, 1.0, 1.0]);

    let pseudo_part = field.pseudoscalar_part();
    let coeffs = pseudo_part.to_coefficients();

    for (i, mv) in coeffs.iter().enumerate() {
        assert!(mv.data()[0].abs() < 1e-4); // Scalar zeroed
        assert!(
            (mv.data()[3] - (i + 1) as f32).abs() < 1e-4,
            "Pseudoscalar (e12) should be {}, got {}",
            i + 1,
            mv.data()[3]
        );
    }
}

// =============================================================================
// Edge cases
// =============================================================================

#[test]
fn test_grade_project_on_zeros() {
    let metric = Metric::from_signature(3, 0, 0);
    let field = CausalMultiField::<CpuBackend, f32>::zeros([2, 2, 2], metric, [1.0, 1.0, 1.0]);

    for k in 0..=3 {
        let projected = field.grade_project(k);
        let coeffs = projected.to_coefficients();

        for mv in coeffs {
            for val in mv.data() {
                assert!(val.abs() < 1e-5);
            }
        }
    }
}

#[test]
fn test_grade_project_beyond_dimension_gives_zeros() {
    let metric = Metric::from_signature(3, 0, 0);
    let num_blades = 8;

    let mut mvs = Vec::with_capacity(8);
    for _ in 0..8 {
        let data = vec![1.0f32; num_blades];
        mvs.push(CausalMultiVector::unchecked(data, metric));
    }

    let field =
        CausalMultiField::<CpuBackend, f32>::from_coefficients(&mvs, [2, 2, 2], [1.0, 1.0, 1.0]);

    // Grade 4 doesn't exist in Cl(3)
    let projected = field.grade_project(4);
    let coeffs = projected.to_coefficients();

    for mv in coeffs {
        for val in mv.data() {
            assert!(val.abs() < 1e-4);
        }
    }
}
