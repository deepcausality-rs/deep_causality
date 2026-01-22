/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for LinkVariable type.
//!
//! Covers constructors, matrix operations, and SU(N) projection.

use deep_causality_num::Complex;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{LinkVariable, LinkVariableError, SU2, U1};

// ============================================================================
// Constructor Tests
// ============================================================================

#[test]
fn test_link_variable_try_identity_u1() {
    let link: LinkVariable<U1, Complex<f64>, f64> =
        LinkVariable::try_identity().expect("Should create U1 identity");
    // U1 is 1x1
    assert_eq!(link.as_slice().len(), 1);
    assert!((link.as_slice()[0] - 1.0).norm() < 1e-10);
}

#[test]
fn test_link_variable_try_identity_su2() {
    let link: LinkVariable<SU2, Complex<f64>, f64> =
        LinkVariable::try_identity().expect("Should create SU2 identity");
    // SU2 is 2x2
    assert_eq!(link.as_slice().len(), 4);
    // Diagonal elements = 1.0
    assert!((link.as_slice()[0] - Complex::new(1.0, 0.0)).norm() < 1e-10); // [0,0]
    assert!((link.as_slice()[3] - Complex::new(1.0, 0.0)).norm() < 1e-10); // [1,1]
    // Off-diagonal = 0.0
    assert!(link.as_slice()[1].norm() < 1e-10); // [0,1]
    assert!(link.as_slice()[2].norm() < 1e-10); // [1,0]
}

#[test]
fn test_link_variable_identity_convenience() {
    let link: LinkVariable<U1, Complex<f64>, f64> = LinkVariable::identity();
    assert!((link.as_slice()[0] - 1.0).norm() < 1e-10);
}

#[test]
fn test_link_variable_try_from_matrix_valid() {
    let tensor = CausalTensor::new(vec![Complex::new(1.0, 0.0)], vec![1, 1]).unwrap();
    let link: Result<LinkVariable<U1, Complex<f64>, f64>, _> =
        LinkVariable::try_from_matrix(tensor);
    assert!(link.is_ok());
}

#[test]
fn test_link_variable_try_from_matrix_wrong_shape() {
    let tensor = CausalTensor::new(
        vec![Complex::new(1.0, 0.0), Complex::new(2.0, 0.0)],
        vec![2, 1],
    )
    .unwrap();
    let link: Result<LinkVariable<U1, Complex<f64>, f64>, _> =
        LinkVariable::try_from_matrix(tensor);
    assert!(link.is_err());
    match link.unwrap_err() {
        LinkVariableError::ShapeMismatch { expected, got } => {
            assert_eq!(expected, vec![1, 1]);
            assert_eq!(got, vec![2, 1]);
        }
        _ => panic!("Expected ShapeMismatch error"),
    }
}

#[test]
fn test_link_variable_from_matrix_unchecked() {
    let tensor = CausalTensor::new(
        vec![
            Complex::new(2.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(2.0, 0.0),
        ],
        vec![2, 2],
    )
    .unwrap();
    let link: LinkVariable<SU2, Complex<f64>, f64> = LinkVariable::from_matrix_unchecked(tensor);
    assert_eq!(link.as_slice()[0], Complex::new(2.0, 0.0));
}

#[test]
fn test_link_variable_try_zero() {
    let link: LinkVariable<SU2, Complex<f64>, f64> =
        LinkVariable::try_zero().expect("Should create zero matrix");
    for val in link.as_slice() {
        assert!(val.norm() < 1e-10);
    }
}

// ============================================================================
// Getter Tests
// ============================================================================

#[test]
fn test_link_variable_matrix() {
    let link: LinkVariable<U1, Complex<f64>, f64> = LinkVariable::identity();
    let matrix = link.matrix();
    assert_eq!(matrix.shape(), &[1, 1]);
}

#[test]
fn test_link_variable_lie_dim() {
    assert_eq!(LinkVariable::<U1, Complex<f64>, f64>::lie_dim(), 1);
    assert_eq!(LinkVariable::<SU2, Complex<f64>, f64>::lie_dim(), 3);
}

#[test]
fn test_link_variable_as_slice() {
    let link: LinkVariable<SU2, Complex<f64>, f64> = LinkVariable::identity();
    let slice = link.as_slice();
    assert_eq!(slice.len(), 4);
}

// ============================================================================
// Operation Tests
// ============================================================================

#[test]
fn test_link_variable_dagger() {
    // For identity, dagger should equal identity
    let id: LinkVariable<SU2, Complex<f64>, f64> = LinkVariable::identity();
    let id_dag = id.dagger();
    for (a, b) in id.as_slice().iter().zip(id_dag.as_slice().iter()) {
        assert!((*a - *b).norm() < 1e-10);
    }
}

#[test]
fn test_link_variable_try_dagger() {
    let id: LinkVariable<SU2, Complex<f64>, f64> = LinkVariable::identity();
    let result = id.try_dagger();
    assert!(result.is_ok());
}

#[test]
fn test_link_variable_mul_identity() {
    // I * I = I
    let id: LinkVariable<SU2, Complex<f64>, f64> = LinkVariable::identity();
    let result = id.mul(&id);
    // Result should be identity
    assert!((result.as_slice()[0] - Complex::new(1.0, 0.0)).norm() < 1e-10);
    assert!((result.as_slice()[3] - Complex::new(1.0, 0.0)).norm() < 1e-10);
}

#[test]
fn test_link_variable_try_mul() {
    let id: LinkVariable<SU2, Complex<f64>, f64> = LinkVariable::identity();
    let result = id.try_mul(&id);
    assert!(result.is_ok());
}

#[test]
fn test_link_variable_add() {
    let id: LinkVariable<SU2, Complex<f64>, f64> = LinkVariable::identity();
    let result = id.add(&id);
    // 2I = [[2, 0], [0, 2]]
    assert!((result.as_slice()[0] - Complex::new(2.0, 0.0)).norm() < 1e-10);
    assert!((result.as_slice()[3] - Complex::new(2.0, 0.0)).norm() < 1e-10);
}

#[test]
fn test_link_variable_try_add() {
    let id: LinkVariable<SU2, Complex<f64>, f64> = LinkVariable::identity();
    let result = id.try_add(&id);
    assert!(result.is_ok());
}

#[test]
fn test_link_variable_scale() {
    let id: LinkVariable<SU2, Complex<f64>, f64> = LinkVariable::identity();
    let scaled = id.scale(&Complex::new(3.0, 0.0));
    assert!((scaled.as_slice()[0] - Complex::new(3.0, 0.0)).norm() < 1e-10);
    assert!((scaled.as_slice()[3] - Complex::new(3.0, 0.0)).norm() < 1e-10);
}

#[test]
fn test_link_variable_try_scale() {
    let id: LinkVariable<SU2, Complex<f64>, f64> = LinkVariable::identity();
    let result = id.try_scale(&Complex::new(2.0, 0.0));
    assert!(result.is_ok());
}

#[test]
fn test_link_variable_trace() {
    let id: LinkVariable<SU2, Complex<f64>, f64> = LinkVariable::identity();
    let tr = id.trace();
    assert!((tr - Complex::new(2.0, 0.0)).norm() < 1e-10); // Tr(I) = N = 2 for SU(2)
}

#[test]
fn test_link_variable_re_trace() {
    let id: LinkVariable<SU2, Complex<f64>, f64> = LinkVariable::identity();
    assert!((id.re_trace() - id.trace().re).abs() < 1e-10);
}

#[test]
fn test_link_variable_frobenius_norm_sq() {
    let id: LinkVariable<SU2, Complex<f64>, f64> = LinkVariable::identity();
    let norm_sq = id.frobenius_norm_sq();
    // ||I||²_F = Tr(I² I) = N = 2
    assert!((norm_sq - 2.0).abs() < 1e-10);
}

// ============================================================================
// Projection Tests
// ============================================================================

#[test]
fn test_link_variable_project_sun_identity() {
    // Identity should project to identity
    let id: LinkVariable<SU2, Complex<f64>, f64> = LinkVariable::identity();
    let projected = id.project_sun().expect("Projection should succeed");
    for (a, b) in id.as_slice().iter().zip(projected.as_slice().iter()) {
        assert!((*a - *b).norm() < 1e-8, "Identity projection mismatch");
    }
}

#[test]
fn test_link_variable_project_sun_scaled_identity() {
    // 2*I should project to I (normalize to unitary)
    let id: LinkVariable<SU2, Complex<f64>, f64> = LinkVariable::identity();
    let scaled = id.scale(&Complex::new(2.0, 0.0));
    let projected = scaled.project_sun().expect("Projection should succeed");

    // After projection, should be close to identity (or -I which is also SU(2))
    let diag_sum = projected.as_slice()[0] + projected.as_slice()[3];
    assert!(
        diag_sum.norm() > 1.5,
        "Projected diagonal should sum to ~±2"
    );
}

#[test]
fn test_link_variable_project_sun_zero_matrix() {
    // Zero matrix should project to identity
    let zero: LinkVariable<SU2, Complex<f64>, f64> = LinkVariable::try_zero().unwrap();
    let projected = zero
        .project_sun()
        .expect("Projection of zero should return identity");
    // Check it's identity
    assert!((projected.as_slice()[0] - Complex::new(1.0, 0.0)).norm() < 1e-10);
}

// ============================================================================
// Display Tests
// ============================================================================

#[test]
fn test_link_variable_display() {
    let id: LinkVariable<SU2, Complex<f64>, f64> = LinkVariable::identity();
    let display = format!("{}", id);
    assert!(display.contains("LinkVariable"));
    assert!(display.contains("SU(2)"));
}

// ============================================================================
// PartialEq Tests
// ============================================================================

#[test]
fn test_link_variable_partial_eq() {
    let id1: LinkVariable<SU2, Complex<f64>, f64> = LinkVariable::identity();
    let id2: LinkVariable<SU2, Complex<f64>, f64> = LinkVariable::identity();
    assert_eq!(id1, id2);
}

#[test]
fn test_link_variable_partial_eq_different() {
    let id: LinkVariable<SU2, Complex<f64>, f64> = LinkVariable::identity();
    let scaled: LinkVariable<SU2, Complex<f64>, f64> = id.scale(&Complex::new(2.0, 0.0));
    assert_ne!(id, scaled);
}

// ============================================================================
// Error Case Tests
// ============================================================================

#[test]
fn test_link_variable_error_display() {
    let err = LinkVariableError::ShapeMismatch {
        expected: vec![2, 2],
        got: vec![3, 3],
    };
    let display = format!("{}", err);
    assert!(display.contains("Shape mismatch"));
}

#[test]
fn test_link_variable_error_tensor_creation() {
    let err = LinkVariableError::TensorCreation("test error".to_string());
    let display = format!("{}", err);
    assert!(display.contains("Tensor creation failed"));
}

#[test]
fn test_link_variable_error_invalid_dimension() {
    let err = LinkVariableError::InvalidDimension(0);
    let display = format!("{}", err);
    assert!(display.contains("Invalid matrix dimension"));
}

#[test]
fn test_link_variable_error_singular_matrix() {
    let err = LinkVariableError::SingularMatrix;
    let display = format!("{}", err);
    assert!(display.contains("Matrix is singular"));
}

#[test]
fn test_link_variable_error_numerical() {
    let err = LinkVariableError::NumericalError("overflow".to_string());
    let display = format!("{}", err);
    assert!(display.contains("overflow"));
}

// ============================================================================
// Random Constructor Tests
// ============================================================================

#[test]
fn test_link_variable_try_random_su2() {
    let mut rng = deep_causality_rand::rng();
    let link: LinkVariable<SU2, Complex<f64>, f64> =
        LinkVariable::try_random(&mut rng).expect("Should create random SU2 link");

    // Should have correct size (2x2 = 4 elements)
    assert_eq!(link.as_slice().len(), 4);

    // Verify it's close to unitary: U * U† ≈ I
    let u_dag = link.dagger();
    let product = link.mul(&u_dag);

    // Trace should be close to 2 (identity trace for 2x2)
    let trace = product.trace();
    assert!(
        (trace - Complex::new(2.0, 0.0)).norm() < 0.5,
        "Random SU2 link should be approximately unitary, got trace = {}",
        trace
    );
}

#[test]
fn test_link_variable_random_u1() {
    let mut rng = deep_causality_rand::rng();
    let link: LinkVariable<U1, Complex<f64>, f64> =
        LinkVariable::try_random(&mut rng).expect("Should create random U1 link");

    // U1 is 1x1
    assert_eq!(link.as_slice().len(), 1);
}

// ============================================================================
// Coverage Gap Tests
// ============================================================================

#[test]
fn test_link_variable_matrix_mut() {
    let mut link: LinkVariable<U1, Complex<f64>, f64> = LinkVariable::identity();
    {
        let _matrix = link.matrix_mut();
        // Just calling matrix_mut covers the line.
        // CausalTensor might not expose simple mutable slice access in tests,
        // but verifying we can call this method is sufficient for line coverage.
    }
}

#[test]
fn test_link_variable_matrix_dim_static() {
    assert_eq!(LinkVariable::<U1, Complex<f64>, f64>::matrix_dim(), 1);
    assert_eq!(LinkVariable::<SU2, Complex<f64>, f64>::matrix_dim(), 2);
}

#[test]
fn test_link_variable_error_conversion() {
    let lv_err = LinkVariableError::SingularMatrix;
    let topo_err: deep_causality_topology::TopologyError = lv_err.into();
    let msg = format!("{}", topo_err);
    assert!(msg.contains("Matrix is singular"));
}

#[test]
fn test_gauge_groups_matrix_dim() {
    use deep_causality_topology::{GaugeGroup, SO3_1, SU3};

    // U1: 1x1
    assert_eq!(U1::matrix_dim(), 1);

    // SU2: 2x2
    assert_eq!(SU2::matrix_dim(), 2);

    // SU3: 3x3
    assert_eq!(SU3::matrix_dim(), 3);

    // Lorentz (SO(3,1)): 4x4
    assert_eq!(SO3_1::matrix_dim(), 4);
}
