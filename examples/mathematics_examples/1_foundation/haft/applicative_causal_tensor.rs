/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_haft::{Applicative, Pure};
use deep_causality_tensor::{CausalTensor, CausalTensorWitness};

fn main() {
    println!("--- Applicative Example: CausalTensor ---");

    // 1. Define a pure value and lift it into CausalTensor context using 'pure'
    let value_a = 5;
    let tensor_a: CausalTensor<i32> = CausalTensorWitness::pure(value_a);
    println!("Tensor A (pure {}): {:?}", value_a, tensor_a);

    // 2. Define a function and lift it into CausalTensor context
    let add_two = |x: i32| x + 2;
    let tensor_f: CausalTensor<fn(i32) -> i32> = CausalTensorWitness::pure(add_two);
    println!("Tensor F (pure add_two function): {:?}", tensor_f);

    // 3. Apply the wrapped function to the wrapped value using 'apply' (or 'ap')
    let result_tensor: CausalTensor<i32> = CausalTensorWitness::apply(tensor_f, tensor_a);
    println!(
        "Result Tensor (add_two to {}): {:?}",
        value_a, result_tensor
    );
    assert_eq!(result_tensor.data(), &[7]);

    // 4. More complex example: Applying a function with multiple arguments
    //    This typically involves lifting a function that takes multiple arguments
    //    and then applying it sequentially.

    let add_all = |a: i32| move |b: i32| move |c: i32| a + b + c;

    // Removed explicit type annotation for tensor_add_all_fn
    let tensor_add_all_fn = CausalTensorWitness::pure(add_all);
    let tensor_val_x: CausalTensor<i32> = CausalTensorWitness::pure(10);
    let tensor_val_y: CausalTensor<i32> = CausalTensorWitness::pure(20);
    let tensor_val_z: CausalTensor<i32> = CausalTensorWitness::pure(30);

    // Apply sequentially
    let res1 = CausalTensorWitness::apply(tensor_add_all_fn, tensor_val_x);
    let res2 = CausalTensorWitness::apply(res1, tensor_val_y);
    let final_sum_tensor: CausalTensor<i32> = CausalTensorWitness::apply(res2, tensor_val_z);

    println!("Final Sum Tensor (10 + 20 + 30): {:?}", final_sum_tensor);
    assert_eq!(final_sum_tensor.data(), &[60]);

    // Example with a CausalTensor that already contains data
    let tensor_values = CausalTensor::new(vec![100, 200], vec![2]).unwrap();
    let add_five_fn = |x: i32| x + 5;
    let tensor_add_five_func: CausalTensor<fn(i32) -> i32> = CausalTensorWitness::pure(add_five_fn);

    // The `apply` method for `CausalTensor` is defined to apply the function in `tensor_add_five_func`
    // to each element in `tensor_values`.
    let result_add_five_to_tensor: CausalTensor<i32> =
        CausalTensorWitness::apply(tensor_add_five_func, tensor_values);
    println!(
        "Result of adding 5 to [100, 200]: {:?}",
        result_add_five_to_tensor
    );
    assert_eq!(result_add_five_to_tensor.data(), &[105, 205]);

    println!("\nApplicative example finished successfully!");
}
