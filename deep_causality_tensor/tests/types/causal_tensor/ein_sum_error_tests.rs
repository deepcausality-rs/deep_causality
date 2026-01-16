/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for error paths in ein_sum_impl.rs

use deep_causality_tensor::{CausalTensor, CausalTensorError, EinSumAST, EinSumOp, EinSumValidationError, utils_tests, Tensor};

// ============================================================================
// Contraction error tests
// ============================================================================

#[test]
fn test_contraction_axes_length_mismatch() {
    let lhs = utils_tests::matrix_tensor(vec![1.0, 2.0, 3.0, 4.0], 2, 2);
    let rhs = utils_tests::matrix_tensor(vec![5.0, 6.0, 7.0, 8.0], 2, 2);

    // lhs_axes has 2 elements, rhs_axes has 1 - mismatch
    let ast = EinSumAST::with_children(
        EinSumOp::Contraction {
            lhs_axes: vec![0, 1],
            rhs_axes: vec![0],
        },
        vec![EinSumOp::tensor_source(lhs), EinSumOp::tensor_source(rhs)],
    );

    let err = CausalTensor::ein_sum(&ast).unwrap_err();
    assert!(matches!(
        err,
        CausalTensorError::EinSumError(EinSumValidationError::InvalidAxesSpecification {
            message: _
        })
    ));
}

#[test]
fn test_contraction_lhs_axis_out_of_bounds() {
    let lhs = utils_tests::matrix_tensor(vec![1.0, 2.0, 3.0, 4.0], 2, 2);
    let rhs = utils_tests::matrix_tensor(vec![5.0, 6.0, 7.0, 8.0], 2, 2);

    // lhs_axes = [5] is out of bounds for 2D tensor
    let ast = EinSumAST::with_children(
        EinSumOp::Contraction {
            lhs_axes: vec![5],
            rhs_axes: vec![0],
        },
        vec![EinSumOp::tensor_source(lhs), EinSumOp::tensor_source(rhs)],
    );

    let err = CausalTensor::ein_sum(&ast).unwrap_err();
    assert!(matches!(
        err,
        CausalTensorError::EinSumError(EinSumValidationError::InvalidAxesSpecification {
            message: _
        })
    ));
}

#[test]
fn test_contraction_rhs_axis_out_of_bounds() {
    let lhs = utils_tests::matrix_tensor(vec![1.0, 2.0, 3.0, 4.0], 2, 2);
    let rhs = utils_tests::matrix_tensor(vec![5.0, 6.0, 7.0, 8.0], 2, 2);

    // rhs_axes = [10] is out of bounds for 2D tensor
    let ast = EinSumAST::with_children(
        EinSumOp::Contraction {
            lhs_axes: vec![0],
            rhs_axes: vec![10],
        },
        vec![EinSumOp::tensor_source(lhs), EinSumOp::tensor_source(rhs)],
    );

    let err = CausalTensor::ein_sum(&ast).unwrap_err();
    assert!(matches!(
        err,
        CausalTensorError::EinSumError(EinSumValidationError::InvalidAxesSpecification {
            message: _
        })
    ));
}

#[test]
fn test_contraction_dimension_mismatch() {
    // lhs is 2x3, rhs is 2x2
    let lhs = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], vec![2, 3]).unwrap();
    let rhs = utils_tests::matrix_tensor(vec![5.0, 6.0, 7.0, 8.0], 2, 2);

    // Try to contract lhs axis 1 (size 3) with rhs axis 0 (size 2) - dimension mismatch
    let ast = EinSumAST::with_children(
        EinSumOp::Contraction {
            lhs_axes: vec![1],
            rhs_axes: vec![0],
        },
        vec![EinSumOp::tensor_source(lhs), EinSumOp::tensor_source(rhs)],
    );

    let err = CausalTensor::ein_sum(&ast).unwrap_err();
    assert!(matches!(
        err,
        CausalTensorError::EinSumError(EinSumValidationError::ShapeMismatch { message: _ })
    ));
}

// ============================================================================
// MatMul error tests
// ============================================================================

#[test]
fn test_mat_mul_rank_mismatch_rhs() {
    let lhs = utils_tests::matrix_tensor(vec![1.0, 2.0, 3.0, 4.0], 2, 2);
    let rhs = utils_tests::vector_tensor(vec![1.0, 2.0]);

    let ast = EinSumOp::<f64>::mat_mul(lhs, rhs);
    let err = CausalTensor::ein_sum(&ast).unwrap_err();
    assert!(matches!(
        err,
        CausalTensorError::EinSumError(EinSumValidationError::RankMismatch {
            expected: 2,
            found: 1
        })
    ));
}

#[test]
fn test_mat_mul_dimension_mismatch() {
    // lhs is 2x3, rhs is 2x2: lhs.shape[1]=3 != rhs.shape[0]=2
    let lhs = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], vec![2, 3]).unwrap();
    let rhs = utils_tests::matrix_tensor(vec![5.0, 6.0, 7.0, 8.0], 2, 2);

    let ast = EinSumOp::<f64>::mat_mul(lhs, rhs);
    let err = CausalTensor::ein_sum(&ast).unwrap_err();
    assert!(matches!(
        err,
        CausalTensorError::EinSumError(EinSumValidationError::ShapeMismatch { message: _ })
    ));
}

// ============================================================================
// Trace error tests
// ============================================================================

#[test]
fn test_trace_axis_out_of_bounds() {
    let operand = utils_tests::matrix_tensor(vec![1.0, 2.0, 3.0, 4.0], 2, 2);

    let ast = EinSumOp::<f64>::trace(operand, 0, 5); // axis2=5 is out of bounds
    let err = CausalTensor::ein_sum(&ast).unwrap_err();
    assert!(matches!(
        err,
        CausalTensorError::EinSumError(EinSumValidationError::InvalidAxesSpecification {
            message: _
        })
    ));
}

#[test]
fn test_trace_shape_mismatch() {
    // Create a 2x3 matrix - axes 0 and 1 have different sizes
    let operand = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], vec![2, 3]).unwrap();

    let ast = EinSumOp::<f64>::trace(operand, 0, 1);
    let err = CausalTensor::ein_sum(&ast).unwrap_err();
    assert!(matches!(
        err,
        CausalTensorError::EinSumError(EinSumValidationError::ShapeMismatch { message: _ })
    ));
}

// ============================================================================
// Diagonal error tests
// ============================================================================

#[test]
fn test_diagonal_axes_identical() {
    let operand = utils_tests::matrix_tensor(vec![1.0, 2.0, 3.0, 4.0], 2, 2);

    let ast = EinSumOp::<f64>::diagonal_extraction(operand, 0, 0); // Same axis
    let err = CausalTensor::ein_sum(&ast).unwrap_err();
    assert!(matches!(
        err,
        CausalTensorError::EinSumError(EinSumValidationError::InvalidAxesSpecification {
            message: _
        })
    ));
}

#[test]
fn test_diagonal_axis_out_of_bounds() {
    let operand = utils_tests::matrix_tensor(vec![1.0, 2.0, 3.0, 4.0], 2, 2);

    let ast = EinSumOp::<f64>::diagonal_extraction(operand, 0, 10); // axis2=10 out of bounds
    let err = CausalTensor::ein_sum(&ast).unwrap_err();
    assert!(matches!(
        err,
        CausalTensorError::EinSumError(EinSumValidationError::InvalidAxesSpecification {
            message: _
        })
    ));
}

#[test]
fn test_diagonal_shape_mismatch() {
    // Create a 2x3 matrix - axes 0 and 1 have different sizes
    let operand = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], vec![2, 3]).unwrap();

    let ast = EinSumOp::<f64>::diagonal_extraction(operand, 0, 1);
    let err = CausalTensor::ein_sum(&ast).unwrap_err();
    assert!(matches!(
        err,
        CausalTensorError::EinSumError(EinSumValidationError::ShapeMismatch { message: _ })
    ));
}

// ============================================================================
// BatchMatMul error tests
// ============================================================================

#[test]
fn test_batch_mat_mul_lhs_rank_too_low() {
    // lhs has only 2 dimensions, needs at least 3
    let lhs = utils_tests::matrix_tensor(vec![1.0, 2.0, 3.0, 4.0], 2, 2);
    let rhs = CausalTensor::new(vec![1.0; 8], vec![2, 2, 2]).unwrap();

    let ast = EinSumOp::<f64>::batch_mat_mul(lhs, rhs);
    let err = CausalTensor::ein_sum(&ast).unwrap_err();
    assert!(matches!(
        err,
        CausalTensorError::EinSumError(EinSumValidationError::RankMismatch {
            expected: 3,
            found: 2
        })
    ));
}

#[test]
fn test_batch_mat_mul_rhs_rank_too_low() {
    let lhs: CausalTensor<f64> = CausalTensor::new(vec![1.0; 8], vec![2, 2, 2]).unwrap();
    // rhs has only 2 dimensions, needs at least 3
    let rhs: CausalTensor<f64> = utils_tests::matrix_tensor(vec![1.0, 2.0, 3.0, 4.0], 2, 2);

    let ast = EinSumOp::<f64>::batch_mat_mul(lhs, rhs);
    let err = CausalTensor::ein_sum(&ast).unwrap_err();
    assert!(matches!(
        err,
        CausalTensorError::EinSumError(EinSumValidationError::RankMismatch {
            expected: 3,
            found: 2
        })
    ));
}

#[test]
fn test_batch_mat_mul_batch_size_mismatch() {
    // lhs batch size is 2, rhs batch size is 3
    let lhs = CausalTensor::new(vec![1.0; 8], vec![2, 2, 2]).unwrap();
    let rhs = CausalTensor::new(vec![1.0; 12], vec![3, 2, 2]).unwrap();

    let ast = EinSumOp::<f64>::batch_mat_mul(lhs, rhs);
    let err = CausalTensor::ein_sum(&ast).unwrap_err();
    assert!(matches!(
        err,
        CausalTensorError::EinSumError(EinSumValidationError::ShapeMismatch { message: _ })
    ));
}

// ============================================================================
// get_binary_operands error tests
// ============================================================================

#[test]
fn test_binary_operand_wrong_children_count() {
    // Provide 3 children instead of 2
    let t1 = utils_tests::scalar_tensor(1.0);
    let t2 = utils_tests::scalar_tensor(2.0);
    let t3 = utils_tests::scalar_tensor(3.0);

    let ast = EinSumAST::with_children(
        EinSumOp::Contraction {
            lhs_axes: vec![],
            rhs_axes: vec![],
        },
        vec![
            EinSumOp::tensor_source(t1),
            EinSumOp::tensor_source(t2),
            EinSumOp::tensor_source(t3),
        ],
    );

    let err = CausalTensor::ein_sum(&ast).unwrap_err();
    assert!(matches!(
        err,
        CausalTensorError::EinSumError(EinSumValidationError::InvalidNumberOfChildren {
            expected: 2,
            found: 3
        })
    ));
}

// ============================================================================
// get_unary_operand error tests
// ============================================================================

#[test]
fn test_unary_operand_wrong_children_count() {
    // Provide 2 children instead of 1 for a unary operation
    let t1 = utils_tests::scalar_tensor(1.0);
    let t2 = utils_tests::scalar_tensor(2.0);

    let ast = EinSumAST::with_children(
        EinSumOp::Trace { axes1: 0, axes2: 1 },
        vec![EinSumOp::tensor_source(t1), EinSumOp::tensor_source(t2)],
    );

    let err = CausalTensor::ein_sum(&ast).unwrap_err();
    assert!(matches!(
        err,
        CausalTensorError::EinSumError(EinSumValidationError::InvalidNumberOfChildren {
            expected: 1,
            found: 2
        })
    ));
}
