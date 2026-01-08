/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_tensor::{CausalTensor, EinSumOp};

fn main() {
    // Example 1: Matrix Multiplication
    println!("--- Example 1: Matrix Multiplication ---");
    let lhs_tensor = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
    let rhs_tensor = CausalTensor::new(vec![5.0, 6.0, 7.0, 8.0], vec![2, 2]).unwrap();

    println!("LHS Tensor: {:?}", lhs_tensor);
    println!("RHS Tensor: {:?}", rhs_tensor);

    let result_mat_mul = CausalTensor::ein_sum(&EinSumOp::mat_mul(lhs_tensor, rhs_tensor)).unwrap();
    println!("Result of Matrix Multiplication:\n{:?}", result_mat_mul);

    let expected_mat_mul = CausalTensor::new(vec![19.0, 22.0, 43.0, 50.0], vec![2, 2]).unwrap();
    assert_eq!(result_mat_mul, expected_mat_mul);

    // Example 2: Dot Product
    println!("\n--- Example 2: Dot Product ---");
    let t1 = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let t2 = CausalTensor::new(vec![4.0, 5.0, 6.0], vec![3]).unwrap();

    println!("LHS Tensor: {:?}", t1);
    println!("RHS Tensor: {:?}", t2);

    let result_dot_prod = CausalTensor::ein_sum(&EinSumOp::dot_prod(t1, t2)).unwrap();
    println!("Result of Dot Product:\n{:?}", result_dot_prod);

    let expected_dot_prod = CausalTensor::new(vec![32.0], vec![]).unwrap();
    assert_eq!(result_dot_prod, expected_dot_prod);

    // Example 3: Trace
    println!("\n--- Example 3: Trace ---");
    let trace_tensor = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
    println!("Tensor for Trace: {:?}", trace_tensor);

    let result_trace = CausalTensor::ein_sum(&EinSumOp::trace(trace_tensor, 0, 1)).unwrap();
    println!("Result of Trace (axes 0, 1):\n{:?}", result_trace);

    let expected_trace = CausalTensor::new(vec![5.0], vec![]).unwrap();
    assert_eq!(result_trace, expected_trace);

    // Example 4: Element-wise Product
    println!("\n--- Example 4: Element-wise Product ---");
    let ew_lhs_tensor = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let ew_rhs_tensor = CausalTensor::new(vec![4.0, 5.0, 6.0], vec![3]).unwrap();

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

    println!("LHS Tensor for Batch MatMul: {:?}", bmm_lhs_tensor);
    println!("RHS Tensor for Batch MatMul: {:?}", bmm_rhs_tensor);

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
