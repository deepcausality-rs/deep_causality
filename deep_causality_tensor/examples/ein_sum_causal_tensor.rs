/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_tensor::{CausalTensor, EinSumOp};

fn main() {
    // Example 1: Matrix Multiplication
    println!("--- Example 1: Matrix Multiplication ---");
    let lhs_data = vec![1.0, 2.0, 3.0, 4.0];
    let lhs_shape = vec![2, 2];
    let lhs_tensor = CausalTensor::new(lhs_data, lhs_shape).unwrap();

    let rhs_data = vec![5.0, 6.0, 7.0, 8.0];
    let rhs_shape = vec![2, 2];
    let rhs_tensor = CausalTensor::new(rhs_data, rhs_shape).unwrap();

    println!("LHS Tensor:\n{:?}", lhs_tensor);
    println!("RHS Tensor:\n{:?}", rhs_tensor);

    let result_mat_mul = CausalTensor::ein_sum(&EinSumOp::mat_mul(lhs_tensor, rhs_tensor)).unwrap();
    println!("Result of Matrix Multiplication:\n{:?}", result_mat_mul);
    let expected_mat_mul = CausalTensor::new(vec![19.0, 22.0, 43.0, 50.0], vec![2, 2]).unwrap();
    assert_eq!(result_mat_mul, expected_mat_mul);

    // Example 2: Dot Product
    println!("\n--- Example 2: Dot Product ---");
    let vec1_data = vec![1.0, 2.0, 3.0];
    let vec1_shape = vec![3];
    let vec1_tensor = CausalTensor::new(vec1_data, vec1_shape).unwrap();

    let vec2_data = vec![4.0, 5.0, 6.0];
    let vec2_shape = vec![3];
    let vec2_tensor = CausalTensor::new(vec2_data, vec2_shape).unwrap();

    println!("Vector 1:\n{:?}", vec1_tensor);
    println!("Vector 2:\n{:?}", vec2_tensor);

    let result_dot_prod =
        CausalTensor::ein_sum(&EinSumOp::dot_prod(vec1_tensor, vec2_tensor)).unwrap();
    println!("Result of Dot Product:\n{:?}", result_dot_prod);
    let expected_dot_prod = CausalTensor::new(vec![32.0], vec![]).unwrap();
    assert_eq!(result_dot_prod, expected_dot_prod);

    // Example 3: Trace
    println!("\n--- Example 3: Trace ---");
    let trace_data = vec![1.0, 2.0, 3.0, 4.0];
    let trace_shape = vec![2, 2];
    let trace_tensor = CausalTensor::new(trace_data, trace_shape).unwrap();

    println!("Tensor for Trace:\n{:?}", trace_tensor);
    let result_trace = CausalTensor::ein_sum(&EinSumOp::trace(trace_tensor, 0, 1)).unwrap();
    println!("Result of Trace (axes 0, 1):\n{:?}", result_trace);
    let expected_trace = CausalTensor::new(vec![5.0], vec![]).unwrap();
    assert_eq!(result_trace, expected_trace);

    // Example 4: Element-wise Product
    println!("\n--- Example 4: Element-wise Product ---");
    let ew_lhs_data = vec![1.0, 2.0, 3.0];
    let ew_lhs_shape = vec![3];
    let ew_lhs_tensor = CausalTensor::new(ew_lhs_data, ew_lhs_shape).unwrap();

    let ew_rhs_data = vec![4.0, 5.0, 6.0];
    let ew_rhs_shape = vec![3];
    let ew_rhs_tensor = CausalTensor::new(ew_rhs_data, ew_rhs_shape).unwrap();

    println!("LHS Tensor for Element-wise Product:\n{:?}", ew_lhs_tensor);
    println!("RHS Tensor for Element-wise Product:\n{:?}", ew_rhs_tensor);

    let result_ew_prod = CausalTensor::ein_sum(&EinSumOp::element_wise_product(
        ew_lhs_tensor,
        ew_rhs_tensor,
    ))
    .unwrap();
    println!("Result of Element-wise Product:\n{:?}", result_ew_prod);
    let expected_ew_prod = CausalTensor::new(vec![4.0, 10.0, 18.0], vec![3]).unwrap();
    assert_eq!(result_ew_prod, expected_ew_prod);

    // Example 5: Batch Matrix Multiplication
    println!("\n--- Example 5: Batch Matrix Multiplication ---");
    // Batch of two 2x2 matrices
    let bmm_lhs_data = vec![
        1.0, 2.0, 3.0, 4.0, // First 2x2 matrix
        5.0, 6.0, 7.0, 8.0, // Second 2x2 matrix
    ];
    let bmm_lhs_shape = vec![2, 2, 2]; // 2 batches, 2 rows, 2 cols
    let bmm_lhs_tensor = CausalTensor::new(bmm_lhs_data, bmm_lhs_shape).unwrap();

    let bmm_rhs_data = vec![
        9.0, 10.0, 11.0, 12.0, // First 2x2 matrix
        13.0, 14.0, 15.0, 16.0, // Second 2x2 matrix
    ];
    let bmm_rhs_shape = vec![2, 2, 2]; // 2 batches, 2 rows, 2 cols
    let bmm_rhs_tensor = CausalTensor::new(bmm_rhs_data, bmm_rhs_shape).unwrap();

    println!("LHS Tensor for Batch MatMul:\n{:?}", bmm_lhs_tensor);
    println!("RHS Tensor for Batch MatMul:\n{:?}", bmm_rhs_tensor);

    let result_bmm =
        CausalTensor::ein_sum(&EinSumOp::batch_mat_mul(bmm_lhs_tensor, bmm_rhs_tensor)).unwrap();

    println!("Result of Batch Matrix Multiplication:\n{:?}", result_bmm);
    let expected_bmm = CausalTensor::new(
        vec![
            31.0, 34.0, 71.0, 78.0, // First 2x2 matrix result
            155.0, 166.0, 211.0, 226.0, // Second 2x2 matrix result
        ],
        vec![2, 2, 2],
    )
    .unwrap();
    assert_eq!(result_bmm, expected_bmm);
}
