/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # `CausalTensor`: the core API surface
//!
//! Construction from flat data plus a shape, then the operations that shape enables:
//! indexing, reshaping, scalar and tensor arithmetic with broadcasting, axis reductions,
//! sorting, element-wise logarithms, and stacking along a new axis.
//!
//! Every fallible call propagates with `?` rather than unwrapping, so a shape mistake
//! surfaces as a returned error instead of a panic.

use deep_causality_num::{const_scalar_from_int, lift};
use deep_causality_tensor::{CausalTensor, CausalTensorMathExt, Tensor};

/// The working scalar, for the sections that do floating-point arithmetic.
///
/// Several sections below use integer tensors on purpose: reductions, sorting, broadcasting
/// and stacking are shape operations, and integers make the result easy to read off.
pub type FloatType = f64;

const ELEVEN: FloatType = const_scalar_from_int!(FloatType, 11);
const TWELVE: FloatType = const_scalar_from_int!(FloatType, 12);
const THIRTEEN: FloatType = const_scalar_from_int!(FloatType, 13);
const FOURTEEN: FloatType = const_scalar_from_int!(FloatType, 14);

/// Small numbers, declared once at the working type rather than lifted at each use.
const ONE: FloatType = const_scalar_from_int!(FloatType, 1);
const TEN: FloatType = const_scalar_from_int!(FloatType, 10);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. A tensor is flat data plus a shape.
    let tensor = CausalTensor::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3])?;
    print_creation(
        &tensor,
        tensor.shape(),
        tensor.is_empty(),
        tensor.num_dim(),
        tensor.len(),
    );

    // 2. Elements are addressed by multi-dimensional index.
    let element = tensor
        .get(&[1, 2])
        .ok_or("index [1, 2] is inside a 2x3 tensor")?;
    print_element(*element);
    assert_eq!(*element, 6);

    // 3. Reshape and ravel rearrange the shape without copying the data.
    let reshaped = tensor.reshape(&[3, 2])?;
    print_reshape(&reshaped);
    assert_eq!(reshaped.shape(), &[3, 2]);

    let raveled = tensor.ravel(); // consumes the original
    print_ravel(&raveled);
    assert_eq!(raveled.shape(), &[6]);

    // 4. Scalar arithmetic is element-wise.
    let ft = CausalTensor::new(vec![ONE, lift(2.0), lift(3.0), lift(4.0)], vec![2, 2])?;
    let added: CausalTensor<FloatType> = &ft + TEN;
    print_scalar_arithmetic(&ft, &added);
    assert_eq!(added.as_slice(), &[ELEVEN, TWELVE, THIRTEEN, FOURTEEN]);

    // 5. Reductions collapse the named axes; an empty axis list reduces everything.
    let grid = CausalTensor::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3])?;
    let sum_axis0 = grid.sum_axes(&[0])?;
    print_reduction(&grid, &sum_axis0);
    assert_eq!(sum_axis0.as_slice(), &[5, 7, 9]);

    let grid_f = CausalTensor::new(
        (1..=6).map(|i| lift::<FloatType>(i as f64)).collect(),
        vec![2, 3],
    )?;
    let mean_all = grid_f.mean_axes(&[])?;
    print_mean(&mean_all);
    assert_eq!(mean_all.as_slice(), &[lift::<FloatType>(3.5)]);

    // 6. `arg_sort` returns the permutation that would sort a rank-1 tensor.
    let one_d = CausalTensor::new(vec![3, 1, 4, 1, 5, 9], vec![6])?;
    let sorted_indices = one_d.arg_sort();
    print_sorting(&one_d, &sorted_indices);
    assert_eq!(sorted_indices?, vec![1, 3, 0, 2, 4, 5]);

    // 7. Tensor-tensor arithmetic broadcasts compatible shapes; [1,3] stretches over [2,3].
    //    Operands may be by value or by reference in any combination.
    let t1 = CausalTensor::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3])?;
    let t2 = CausalTensor::new(vec![10, 20, 30], vec![1, 3])?;
    let broadcast = &t1 + &t2;
    print_broadcast(&t1, &t2, &broadcast);
    assert_eq!(broadcast.as_slice(), &[11, 22, 33, 14, 25, 36]);

    // 8. Element-wise logarithms, for floating-point data.
    let logs = CausalTensor::new(
        vec![
            ONE,
            lift(std::f64::consts::E),
            lift(10.0),
            lift(100.0),
            lift(4.0),
            lift(16.0),
        ],
        vec![2, 3],
    )?;
    print_logs(&logs, &logs.log_nat()?, &logs.log2()?, &logs.log10()?);

    // 9. Stacking joins a slice of tensors along a new axis, so the rank grows by one.
    let a = CausalTensor::<i32>::new(vec![1, 2], vec![2])?;
    let b = CausalTensor::<i32>::new(vec![3, 4], vec![2])?;
    print_stack_inputs(&a, &b);
    let to_stack = [a, b];

    let axis_0 = CausalTensor::stack(&to_stack, 0)?;
    print_stacked(0, axis_0.shape(), &axis_0);
    assert_eq!(axis_0.shape(), &[2, 2]);
    assert_eq!(axis_0.as_slice(), &[1, 2, 3, 4]);

    let axis_1 = CausalTensor::stack(&to_stack, 1)?;
    print_stacked(1, axis_1.shape(), &axis_1);
    assert_eq!(axis_1.shape(), &[2, 2]);
    assert_eq!(axis_1.as_slice(), &[1, 3, 2, 4]);

    print_footer();
    Ok(())
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_creation(
    tensor: &CausalTensor<i32>,
    shape: &[usize],
    is_empty: bool,
    num_dim: usize,
    len: usize,
) {
    println!("\n--- CausalTensor Example ---");
    println!("\n1. Creating a 2x3 tensor:");
    println!("   Tensor: {tensor}");
    println!("   Shape: {shape:?}");
    println!("   Is empty: {is_empty}");
    println!("   Number of dimensions: {num_dim}");
    println!("   Total elements: {len}");
}

fn print_element(value: i32) {
    println!("\n2. Accessing element at [1, 2]:");
    println!("   Value: {value}");
}

fn print_reshape(reshaped: &CausalTensor<i32>) {
    println!("\n3. Reshaping the tensor to 3x2:");
    println!("   Reshaped Tensor: {reshaped}");
}

fn print_ravel(raveled: &CausalTensor<i32>) {
    println!("\n   Flattening the tensor (ravel):");
    println!("   Raveled Tensor: {raveled}");
}

fn print_scalar_arithmetic(original: &CausalTensor<FloatType>, added: &CausalTensor<FloatType>) {
    println!("\n4. Tensor-Scalar Arithmetic (add 10 to each element):");
    println!("   Original: {original}");
    println!("   Result:   {added}");
}

fn print_reduction(original: &CausalTensor<i32>, sum_axis0: &CausalTensor<i32>) {
    println!("\n5. Reduction Operations on a 2x3 tensor:");
    println!("   Original Tensor: {original}");
    println!("   Sum along axis 0: {sum_axis0}");
}

fn print_mean(mean_all: &CausalTensor<FloatType>) {
    println!("   Mean of all elements: {mean_all}");
}

fn print_sorting<E: std::fmt::Debug>(
    one_d: &CausalTensor<i32>,
    sorted_indices: &Result<Vec<usize>, E>,
) {
    println!("\n6. Sorting a 1D tensor:");
    println!("   Original 1D Tensor: {one_d}");
    println!("   Sorted indices: {sorted_indices:?}");
}

fn print_broadcast(t1: &CausalTensor<i32>, t2: &CausalTensor<i32>, result: &CausalTensor<i32>) {
    println!("\n7. Tensor-Tensor Addition with Broadcasting:");
    println!("   Tensor 1: {t1}");
    println!("   Tensor 2 (to be broadcasted): {t2}");
    println!("   Result (t1 + t2): {result}");
}

fn print_logs(
    original: &CausalTensor<FloatType>,
    nat: &CausalTensor<FloatType>,
    two: &CausalTensor<FloatType>,
    ten: &CausalTensor<FloatType>,
) {
    println!("\n8. Logarithmic Functions on a 2x3 tensor:");
    println!("   Original Tensor: {original}");
    println!("   Natural Log (ln): {nat}");
    println!("   Base 2 Log (log2): {two}");
    println!("   Base 10 Log (log10): {ten}");
}

fn print_stack_inputs(a: &CausalTensor<i32>, b: &CausalTensor<i32>) {
    println!("\n9. Stacking two 2-element vectors:");
    println!("   Tensor A: {a}");
    println!("   Tensor B: {b}");
}

fn print_stacked(axis: usize, shape: &[usize], stacked: &CausalTensor<i32>) {
    println!("   Stacked along axis {axis} (new shape {{:?}}): {{}} {shape:?} {stacked:?}");
}

fn print_footer() {
    println!("\nAll examples executed successfully!");
}
