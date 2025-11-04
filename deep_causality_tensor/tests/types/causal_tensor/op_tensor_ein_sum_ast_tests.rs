/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_tensor::{CausalTensor, EinSumOp, utils_tests};

#[test]
fn test_tensor_source_ast() {
    let tensor = utils_tests::scalar_tensor(1.0);
    let ast = EinSumOp::tensor_source(tensor.clone());

    assert!(matches!(ast.value(), EinSumOp::TensorSource { tensor: t } if t == &tensor));
    assert!(ast.children().is_empty());
}

#[test]
fn test_contraction_ast() {
    let lhs = utils_tests::matrix_tensor(vec![1.0, 2.0, 3.0, 4.0], 2, 2);
    let rhs = utils_tests::matrix_tensor(vec![5.0, 6.0, 7.0, 8.0], 2, 2);
    let lhs_axes = vec![1];
    let rhs_axes = vec![0];

    let ast = EinSumOp::contraction(lhs.clone(), rhs.clone(), lhs_axes.clone(), rhs_axes.clone());

    assert!(
        matches!(ast.value(), EinSumOp::Contraction { lhs_axes: la, rhs_axes: ra } if la == &lhs_axes && ra == &rhs_axes)
    );
    assert_eq!(ast.children().len(), 2);
    assert!(matches!(ast.children()[0].value(), EinSumOp::TensorSource { tensor: t } if t == &lhs));
    assert!(matches!(ast.children()[1].value(), EinSumOp::TensorSource { tensor: t } if t == &rhs));
}

#[test]
fn test_reduction_ast() {
    let operand = utils_tests::vector_tensor(vec![1.0, 2.0, 3.0]);
    let axes = vec![0];

    let ast = EinSumOp::reduction(operand.clone(), axes.clone());

    assert!(matches!(ast.value(), EinSumOp::Reduction { axes: a } if a == &axes));
    assert_eq!(ast.children().len(), 1);
    assert!(
        matches!(ast.children()[0].value(), EinSumOp::TensorSource { tensor: t } if t == &operand)
    );
}

#[test]
fn test_mat_mul_ast() {
    let lhs = utils_tests::matrix_tensor(vec![1.0, 2.0, 3.0, 4.0], 2, 2);
    let rhs = utils_tests::matrix_tensor(vec![5.0, 6.0, 7.0, 8.0], 2, 2);

    let ast = EinSumOp::mat_mul(lhs.clone(), rhs.clone());

    assert!(matches!(ast.value(), EinSumOp::MatMul));
    assert_eq!(ast.children().len(), 2);
    assert!(matches!(ast.children()[0].value(), EinSumOp::TensorSource { tensor: t } if t == &lhs));
    assert!(matches!(ast.children()[1].value(), EinSumOp::TensorSource { tensor: t } if t == &rhs));
}

#[test]
fn test_dot_prod_ast() {
    let lhs = utils_tests::vector_tensor(vec![1.0, 2.0]);
    let rhs = utils_tests::vector_tensor(vec![3.0, 4.0]);

    let ast = EinSumOp::dot_prod(lhs.clone(), rhs.clone());

    assert!(matches!(ast.value(), EinSumOp::DotProd));
    assert_eq!(ast.children().len(), 2);
    assert!(matches!(ast.children()[0].value(), EinSumOp::TensorSource { tensor: t } if t == &lhs));
    assert!(matches!(ast.children()[1].value(), EinSumOp::TensorSource { tensor: t } if t == &rhs));
}

#[test]
fn test_trace_ast() {
    let operand = utils_tests::matrix_tensor(vec![1.0, 2.0, 3.0, 4.0], 2, 2);
    let axes1 = 0;
    let axes2 = 1;

    let ast = EinSumOp::trace(operand.clone(), axes1, axes2);

    assert!(
        matches!(ast.value(), EinSumOp::Trace { axes1: a1, axes2: a2 } if *a1 == axes1 && *a2 == axes2)
    );
    assert_eq!(ast.children().len(), 1);
    assert!(
        matches!(ast.children()[0].value(), EinSumOp::TensorSource { tensor: t } if t == &operand)
    );
}

#[test]
fn test_tensor_product_ast() {
    let lhs = utils_tests::vector_tensor(vec![1.0, 2.0]);
    let rhs = utils_tests::vector_tensor(vec![3.0, 4.0]);

    let ast = EinSumOp::tensor_product(lhs.clone(), rhs.clone());

    assert!(matches!(ast.value(), EinSumOp::TensorProduct));
    assert_eq!(ast.children().len(), 2);
    assert!(matches!(ast.children()[0].value(), EinSumOp::TensorSource { tensor: t } if t == &lhs));
    assert!(matches!(ast.children()[1].value(), EinSumOp::TensorSource { tensor: t } if t == &rhs));
}

#[test]
fn test_element_wise_product_ast() {
    let lhs = utils_tests::vector_tensor(vec![1.0, 2.0]);
    let rhs = utils_tests::vector_tensor(vec![3.0, 4.0]);

    let ast = EinSumOp::element_wise_product(lhs.clone(), rhs.clone());

    assert!(matches!(ast.value(), EinSumOp::ElementWiseProduct));
    assert_eq!(ast.children().len(), 2);
    assert!(matches!(ast.children()[0].value(), EinSumOp::TensorSource { tensor: t } if t == &lhs));
    assert!(matches!(ast.children()[1].value(), EinSumOp::TensorSource { tensor: t } if t == &rhs));
}

#[test]
fn test_transpose_ast() {
    let operand = utils_tests::matrix_tensor(vec![1.0, 2.0, 3.0, 4.0], 2, 2);
    let new_order = vec![1, 0];

    let ast = EinSumOp::transpose(operand.clone(), new_order.clone());

    assert!(matches!(ast.value(), EinSumOp::Transpose { new_order: no } if no == &new_order));
    assert_eq!(ast.children().len(), 1);
    assert!(
        matches!(ast.children()[0].value(), EinSumOp::TensorSource { tensor: t } if t == &operand)
    );
}

#[test]
fn test_diagonal_extraction_ast() {
    let operand = utils_tests::matrix_tensor(vec![1.0, 2.0, 3.0, 4.0], 2, 2);
    let axes1 = 0;
    let axes2 = 1;

    let ast = EinSumOp::diagonal_extraction(operand.clone(), axes1, axes2);

    assert!(
        matches!(ast.value(), EinSumOp::DiagonalExtraction { axes1: a1, axes2: a2 } if *a1 == axes1 && *a2 == axes2)
    );
    assert_eq!(ast.children().len(), 1);
    assert!(
        matches!(ast.children()[0].value(), EinSumOp::TensorSource { tensor: t } if t == &operand)
    );
}

#[test]
fn test_batch_mat_mul_ast() {
    let lhs = CausalTensor::new(vec![1.0; 8], vec![2, 2, 2]).unwrap();
    let rhs = CausalTensor::new(vec![1.0; 8], vec![2, 2, 2]).unwrap();

    let ast = EinSumOp::batch_mat_mul(lhs.clone(), rhs.clone());

    assert!(matches!(ast.value(), EinSumOp::BatchMatMul));
    assert_eq!(ast.children().len(), 2);
    assert!(matches!(ast.children()[0].value(), EinSumOp::TensorSource { tensor: t } if t == &lhs));
    assert!(matches!(ast.children()[1].value(), EinSumOp::TensorSource { tensor: t } if t == &rhs));
}
