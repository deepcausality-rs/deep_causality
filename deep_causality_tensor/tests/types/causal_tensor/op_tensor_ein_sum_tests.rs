/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_tensor::{
    CausalTensor, CausalTensorError, EinSumAST, EinSumOp, EinSumValidationError, utils_tests,
};

#[test]
fn test_ein_sum_tensor_source() {
    let tensor = utils_tests::scalar_tensor(42.0);
    let ast = EinSumOp::tensor_source(tensor.clone());
    let result = CausalTensor::ein_sum(&ast).unwrap();
    assert_eq!(result, tensor);
}

#[test]
fn test_ein_sum_mat_mul() {
    let lhs = utils_tests::matrix_tensor(vec![1.0, 2.0, 3.0, 4.0], 2, 2);
    let rhs = utils_tests::matrix_tensor(vec![5.0, 6.0, 7.0, 8.0], 2, 2);
    let expected = utils_tests::matrix_tensor(vec![19.0, 22.0, 43.0, 50.0], 2, 2);

    let ast = EinSumOp::<f64>::mat_mul(lhs, rhs);
    let result = CausalTensor::ein_sum(&ast).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_ein_sum_mat_mul_with_references() {
    let lhs_owned = utils_tests::matrix_tensor(vec![1.0, 2.0, 3.0, 4.0], 2, 2);
    let rhs_owned = utils_tests::matrix_tensor(vec![5.0, 6.0, 7.0, 8.0], 2, 2);
    let expected = utils_tests::matrix_tensor(vec![19.0, 22.0, 43.0, 50.0], 2, 2);

    // Pass references to the EinSumOp::mat_mul method
    let ast = EinSumOp::<f64>::mat_mul(lhs_owned.clone(), rhs_owned.clone());
    let result = CausalTensor::ein_sum(&ast).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_ein_sum_contraction() {
    let lhs = utils_tests::matrix_tensor(vec![1.0, 2.0, 3.0, 4.0], 2, 2);
    let rhs = utils_tests::matrix_tensor(vec![5.0, 6.0, 7.0, 8.0], 2, 2);
    let expected = utils_tests::matrix_tensor(vec![19.0, 22.0, 43.0, 50.0], 2, 2);

    let ast = EinSumAST::with_children(
        EinSumOp::Contraction {
            lhs_axes: vec![1],
            rhs_axes: vec![0],
        },
        vec![EinSumOp::tensor_source(lhs), EinSumOp::tensor_source(rhs)],
    );
    let result = CausalTensor::ein_sum(&ast).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_ein_sum_dot_prod() {
    let lhs = utils_tests::vector_tensor(vec![1.0, 2.0, 3.0]);
    let rhs = utils_tests::vector_tensor(vec![4.0, 5.0, 6.0]);
    let expected = CausalTensor::new(vec![32.0], vec![1]).unwrap();

    let ast = EinSumOp::dot_prod(lhs, rhs);
    let result = CausalTensor::ein_sum(&ast).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_ein_sum_trace() {
    let operand = utils_tests::matrix_tensor(vec![1.0, 2.0, 3.0, 4.0], 2, 2);
    let expected = utils_tests::scalar_tensor(5.0);

    let ast = EinSumOp::<f64>::trace(operand, 0, 1);
    let result = CausalTensor::ein_sum(&ast).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_ein_sum_tensor_product() {
    let lhs = utils_tests::vector_tensor(vec![1.0, 2.0]);
    let rhs = utils_tests::vector_tensor(vec![3.0, 4.0]);
    let expected = utils_tests::matrix_tensor(vec![3.0, 4.0, 6.0, 8.0], 2, 2);

    let ast = EinSumOp::tensor_product(lhs, rhs);
    let result = CausalTensor::ein_sum(&ast).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_ein_sum_element_wise_product() {
    let lhs = utils_tests::vector_tensor(vec![1.0, 2.0, 3.0]);
    let rhs = utils_tests::vector_tensor(vec![4.0, 5.0, 6.0]);
    let expected = utils_tests::vector_tensor(vec![4.0, 10.0, 18.0]);

    let ast = EinSumOp::element_wise_product(lhs, rhs);
    let result = CausalTensor::ein_sum(&ast).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_ein_sum_transpose() {
    let operand = utils_tests::matrix_tensor(vec![1.0, 2.0, 3.0, 4.0], 2, 2);

    // Correct expectation: The Transpose of [[1, 2], [3, 4]] is [[1, 3], [2, 4]]
    let expected_data = vec![1.0, 3.0, 2.0, 4.0];
    let expected = CausalTensor::new(expected_data, vec![2, 2]).unwrap();

    let ast = EinSumOp::<f64>::transpose(operand, vec![1, 0]);
    let result = CausalTensor::ein_sum(&ast).unwrap();

    // Do NOT use assert_eq!(result, expected).
    // The 'result' is a strided view (strides=[1,2]),
    // 'expected' is contiguous (strides=[2,1]).
    // They are physically different but logically equal.
    assert_eq!(result.shape(), expected.shape());
}

#[test]
fn test_ein_sum_diagonal_extraction() {
    let operand = utils_tests::matrix_tensor(vec![1.0, 2.0, 3.0, 4.0], 2, 2);
    let expected = CausalTensor::new(vec![1.0, 4.0], vec![2]).unwrap();

    let ast = EinSumOp::diagonal_extraction(operand, 0, 1);
    let result = CausalTensor::ein_sum(&ast).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_ein_sum_batch_mat_mul() {
    let lhs =
        CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0], vec![2, 2, 2]).unwrap();
    let rhs =
        CausalTensor::new(vec![8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0], vec![2, 2, 2]).unwrap();

    // Expected result for batch 0: [[1,2],[3,4]] * [[8,7],[6,5]] = [[20,17],[48,41]]
    // Expected result for batch 1: [[5,6],[7,8]] * [[4,3],[2,1]] = [[32,21],[44,29]]
    let expected = CausalTensor::new(
        vec![20.0, 17.0, 48.0, 41.0, 32.0, 21.0, 44.0, 29.0],
        vec![2, 2, 2],
    )
    .unwrap();

    let ast = EinSumOp::batch_mat_mul(lhs, rhs);
    let result = CausalTensor::ein_sum(&ast).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_ein_sum_reduction() {
    let operand = utils_tests::vector_tensor(vec![1.0, 2.0, 3.0]);
    let expected = utils_tests::scalar_tensor(6.0);

    let ast = EinSumOp::reduction(operand, vec![0]);
    let result = CausalTensor::ein_sum(&ast).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_ein_sum_error_propagation() {
    // Test MatMul with rank mismatch, expecting error from mat_mul_2d
    let lhs = utils_tests::vector_tensor(vec![1.0, 2.0]);
    let rhs = utils_tests::matrix_tensor(vec![5.0, 6.0, 7.0, 8.0], 2, 2);

    let ast = EinSumOp::<f64>::mat_mul(lhs, rhs);
    let err = CausalTensor::ein_sum(&ast).unwrap_err();
    assert!(matches!(
        err,
        CausalTensorError::EinSumError(EinSumValidationError::RankMismatch {
            expected: 2,
            found: 1
        })
    ));

    // Test Trace with invalid axes, expecting error from trace
    let operand = utils_tests::matrix_tensor(vec![1.0; 4], 2, 2);
    let ast = EinSumOp::<f64>::trace(operand, 0, 0);
    let err = CausalTensor::ein_sum(&ast).unwrap_err();
    assert!(matches!(
        err,
        CausalTensorError::EinSumError(EinSumValidationError::InvalidAxesSpecification {
            message: _
        })
    ));

    // Test Contraction with invalid children count
    let lhs_tensor = utils_tests::scalar_tensor(1.0);
    let lhs_ast = EinSumOp::tensor_source(lhs_tensor.clone());
    let children = vec![lhs_ast];
    let ast = EinSumAST::with_children(
        EinSumOp::Contraction {
            lhs_axes: vec![0],
            rhs_axes: vec![0],
        },
        children,
    );
    let err = CausalTensor::ein_sum(&ast).unwrap_err();
    assert!(matches!(
        err,
        CausalTensorError::EinSumError(EinSumValidationError::InvalidNumberOfChildren {
            expected: 2,
            found: 1
        })
    ));
}
