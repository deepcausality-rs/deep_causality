/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_tensor::{CausalTensor, CausalTensorMathExt, CausalTensorStackExt, Tensor};

pub fn main() {
    println!("\n--- CausalTensor Example ---");

    // 1. Creating a new CausalTensor
    // Tensors can be created from a flat Vec of data and a shape.
    println!("\n1. Creating a 2x3 tensor:");
    let data = vec![1, 2, 3, 4, 5, 6];
    let shape = vec![2, 3];
    let tensor = CausalTensor::new(data, shape).unwrap();
    println!("   Tensor: {}", tensor);
    println!("   Shape: {:?}", tensor.shape());
    println!("   Is empty: {}", tensor.is_empty());
    println!("   Number of dimensions: {}", tensor.num_dim());
    println!("   Total elements: {}", tensor.len());

    // 2. Accessing elements
    // Elements can be accessed using their multi-dimensional index.
    println!("\n2. Accessing element at [1, 2]:");
    let element = tensor.get(&[1, 2]).unwrap();
    println!("   Value: {}", element);
    assert_eq!(*element, 6);

    // 3. Shape Manipulation
    // Tensors can be reshaped or flattened without copying the underlying data.
    println!("\n3. Reshaping the tensor to 3x2:");
    let reshaped_tensor = tensor.reshape(&[3, 2]).unwrap();
    println!("   Reshaped Tensor: {}", reshaped_tensor);
    assert_eq!(reshaped_tensor.shape(), &[3, 2]);

    println!("\n   Flattening the tensor (ravel):");
    let raveled_tensor = tensor.ravel(); // Consumes the original tensor
    println!("   Raveled Tensor: {}", raveled_tensor);
    assert_eq!(raveled_tensor.shape(), &[6]);

    // 4. Tensor-Scalar Arithmetic
    // Tensors support element-wise arithmetic with scalars.
    println!("\n4. Tensor-Scalar Arithmetic (add 10 to each element):");
    let tensor = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
    let added_tensor = &tensor + 10.0;
    println!("   Original: {}", tensor);
    println!("   Result:   {}", added_tensor);
    assert_eq!(added_tensor.as_slice(), &[11.0, 12.0, 13.0, 14.0]);

    // 5. Reduction Operations
    // You can reduce tensors along axes using operations like sum and mean.
    println!("\n5. Reduction Operations on a 2x3 tensor:");
    let tensor = CausalTensor::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3]).unwrap();
    println!("   Original Tensor: {}", tensor);

    // Sum along axis 0 (columns)
    let sum_axis0 = tensor.sum_axes(&[0]).unwrap();
    println!("   Sum along axis 0: {}", sum_axis0);
    assert_eq!(sum_axis0.as_slice(), &[5, 7, 9]);

    // Mean of all elements (full reduction)
    let tensor_f64 = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], vec![2, 3]).unwrap();
    let mean_all = tensor_f64.mean_axes(&[]).unwrap();
    println!("   Mean of all elements: {}", mean_all);
    assert_eq!(mean_all.as_slice(), &[3.5]);

    // 6. Sorting a 1D Tensor
    // You can get the indices that would sort a 1D tensor.
    println!("\n6. Sorting a 1D tensor:");
    let tensor_1d = CausalTensor::new(vec![3, 1, 4, 1, 5, 9], vec![6]).unwrap();
    println!("   Original 1D Tensor: {}", tensor_1d);
    let sorted_indices = tensor_1d.arg_sort().unwrap();
    println!("   Sorted indices: {:?}", sorted_indices);
    assert_eq!(sorted_indices, vec![1, 3, 0, 2, 4, 5]);

    // 7. Tensor-Tensor Arithmetic
    // Tensors can be added, subtracted, etc., with other tensors.
    // This supports broadcasting for compatible shapes.
    println!("\n7. Tensor-Tensor Addition with Broadcasting:");
    let t1 = CausalTensor::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3]).unwrap();
    // This tensor will be broadcasted to match the shape of t1.
    let t2 = CausalTensor::new(vec![10, 20, 30], vec![1, 3]).unwrap();
    println!("   Tensor 1: {}", t1);
    println!("   Tensor 2 (to be broadcasted): {}", t2);

    // Tensor ops can be by value, by ref, or any combination i.e. &t1 + t2
    let result = &t1 + &t2;
    println!("   Result (t1 + t2): {}", result);
    assert_eq!(result.as_slice(), &[11, 22, 33, 14, 25, 36]);

    // 8. Logarithmic Functions. Import CausalTensorLogMathExt
    // Tensors with floating-point data support element-wise logarithmic operations.
    println!("\n8. Logarithmic Functions on a 2x3 tensor:");
    let tensor_f64_log = CausalTensor::new(
        vec![1.0, std::f64::consts::E, 10.0, 100.0, 4.0, 16.0],
        vec![2, 3],
    )
    .unwrap();
    println!("   Original Tensor: {}", tensor_f64_log);

    let log_nat_tensor = tensor_f64_log.log_nat().unwrap();
    println!("   Natural Log (ln): {}", log_nat_tensor);

    let log2_tensor = tensor_f64_log.log2().unwrap();
    println!("   Base 2 Log (log2): {}", log2_tensor);

    let log10_tensor = tensor_f64_log.log10().unwrap();
    println!("   Base 10 Log (log10): {}", log10_tensor);

    // 9. Stacking Tensors. Import CausalTensorCollectionExt
    // A slice of tensors can be stacked along a new axis.
    println!("\n9. Stacking two 2-element vectors:");
    let tensor_a = CausalTensor::<i32>::new(vec![1, 2], vec![2]).unwrap();
    println!("   Tensor A: {}", tensor_a);
    let tensor_b = CausalTensor::<i32>::new(vec![3, 4], vec![2]).unwrap();
    println!("   Tensor B: {}", tensor_b);

    let tensors_to_stack = [tensor_a, tensor_b];

    // Stack along axis 0
    let stacked_axis_0 = tensors_to_stack.stack(0).unwrap();
    println!(
        "   Stacked along axis 0 (new shape {{:?}}): {{}} {:?} {:?}",
        stacked_axis_0.shape(),
        stacked_axis_0
    );
    assert_eq!(stacked_axis_0.shape(), &[2, 2]);
    assert_eq!(stacked_axis_0.as_slice(), &[1, 2, 3, 4]);

    // Stack along axis 1
    let stacked_axis_1 = tensors_to_stack.stack(1).unwrap();
    println!(
        "   Stacked along axis 1 (new shape {{:?}}): {{}} {:?} {:?}",
        stacked_axis_1.shape(),
        stacked_axis_1
    );
    assert_eq!(stacked_axis_1.shape(), &[2, 2]);
    assert_eq!(stacked_axis_1.as_slice(), &[1, 3, 2, 4]);

    println!("\nAll examples executed successfully!");
}
