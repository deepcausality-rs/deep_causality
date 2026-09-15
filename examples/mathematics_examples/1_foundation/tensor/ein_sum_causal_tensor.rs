/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # `EinSumOp`: Einstein summation over `CausalTensor`
//!
//! Einstein notation names the contraction rather than spelling out the loops. `EinSumOp`
//! builds the expression as a value and `CausalTensor::ein_sum` evaluates it, so the five
//! operations below are one interface rather than five functions:
//!
//! ```text
//! mat_mul                 [2,2] x [2,2] -> [2,2]     contract the inner axis
//! dot_prod                [3]   x [3]   -> []        contract the only axis, to a scalar
//! trace(a, b)             [2,2]         -> []        contract two axes of one tensor
//! element_wise_product    [3]   x [3]   -> [3]       contract nothing
//! batch_mat_mul           [2,2,2] x ... -> [2,2,2]   the leading axis rides along
//! ```
//!
//! Every result is checked against the value worked out by hand.

use deep_causality_num::lift;
use deep_causality_tensor::{CausalTensor, EinSumOp, Tensor};

/// The working scalar. Every tensor component carries it.
pub type FloatType = f64;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Matrix multiplication contracts the inner axis: [2,2] x [2,2] -> [2,2].
    let lhs = tensor(&[1.0, 2.0, 3.0, 4.0], &[2, 2])?;
    let rhs = tensor(&[5.0, 6.0, 7.0, 8.0], &[2, 2])?;
    print_inputs("--- Example 1: Matrix Multiplication ---", &lhs, &rhs);
    let mat_mul = CausalTensor::ein_sum(&EinSumOp::mat_mul(lhs, rhs))?;
    print_result("Result of Matrix Multiplication", &mat_mul);
    assert_eq!(mat_mul, tensor(&[19.0, 22.0, 43.0, 50.0], &[2, 2])?);

    // 2. A dot product contracts the only axis, so the result is a rank-0 scalar.
    let t1 = tensor(&[1.0, 2.0, 3.0], &[3])?;
    let t2 = tensor(&[4.0, 5.0, 6.0], &[3])?;
    print_inputs("\n--- Example 2: Dot Product ---", &t1, &t2);
    let dot = CausalTensor::ein_sum(&EinSumOp::dot_prod(t1, t2))?;
    print_result("Result of Dot Product", &dot);
    assert_eq!(dot, tensor(&[32.0], &[])?); // 1*4 + 2*5 + 3*6

    // 3. A trace contracts two axes of a single tensor: 1 + 4 = 5.
    let square = tensor(&[1.0, 2.0, 3.0, 4.0], &[2, 2])?;
    print_single("\n--- Example 3: Trace ---", "Tensor for Trace", &square);
    let trace = CausalTensor::ein_sum(&EinSumOp::trace(square, 0, 1))?;
    print_result("Result of Trace (axes 0, 1)", &trace);
    assert_eq!(trace, tensor(&[5.0], &[])?);

    // 4. An element-wise product contracts nothing, so the shape survives.
    let ew_lhs = tensor(&[1.0, 2.0, 3.0], &[3])?;
    let ew_rhs = tensor(&[4.0, 5.0, 6.0], &[3])?;
    print_pair_multiline(
        "\n--- Example 4: Element-wise Product ---",
        &ew_lhs,
        &ew_rhs,
    );
    let ew = CausalTensor::ein_sum(&EinSumOp::element_wise_product(ew_lhs, ew_rhs))?;
    print_result("Result of Element-wise Product", &ew);
    assert_eq!(ew, tensor(&[4.0, 10.0, 18.0], &[3])?);

    // 5. Batch matrix multiplication: the leading axis is carried, not contracted, so two
    //    independent 2x2 products happen in one call.
    let bmm_lhs = tensor(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0], &[2, 2, 2])?;
    let bmm_rhs = tensor(&[9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0], &[2, 2, 2])?;
    print_batch_inputs(&bmm_lhs, &bmm_rhs);
    let bmm = CausalTensor::ein_sum(&EinSumOp::batch_mat_mul(bmm_lhs, bmm_rhs))?;
    print_result("Result of Batch Matrix Multiplication", &bmm);
    assert_eq!(
        bmm,
        tensor(
            &[31.0, 34.0, 71.0, 78.0, 155.0, 166.0, 211.0, 226.0],
            &[2, 2, 2]
        )?
    );

    Ok(())
}

/// A tensor of the working scalar from row-major literals.
fn tensor(
    values: &[f64],
    shape: &[usize],
) -> Result<CausalTensor<FloatType>, Box<dyn std::error::Error>> {
    let data: Vec<FloatType> = values.iter().map(|&v| lift(v)).collect();
    Ok(CausalTensor::new(data, shape.to_vec())?)
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_inputs(title: &str, lhs: &CausalTensor<FloatType>, rhs: &CausalTensor<FloatType>) {
    println!("{title}");
    println!("LHS Tensor: {lhs:?}");
    println!("RHS Tensor: {rhs:?}");
}

fn print_single(title: &str, label: &str, t: &CausalTensor<FloatType>) {
    println!("{title}");
    println!("{label}: {t:?}");
}

fn print_pair_multiline(title: &str, lhs: &CausalTensor<FloatType>, rhs: &CausalTensor<FloatType>) {
    println!("{title}");
    println!("LHS Tensor for Element-wise Product:\n{lhs:?}");
    println!("RHS Tensor for Element-wise Product:\n{rhs:?}");
}

fn print_batch_inputs(lhs: &CausalTensor<FloatType>, rhs: &CausalTensor<FloatType>) {
    println!("\n--- Example 5: Batch Matrix Multiplication ---");
    println!("LHS Tensor for Batch MatMul: {lhs:?}");
    println!("RHS Tensor for Batch MatMul: {rhs:?}");
}

fn print_result(label: &str, t: &CausalTensor<FloatType>) {
    println!("{label}:\n{t:?}");
}
