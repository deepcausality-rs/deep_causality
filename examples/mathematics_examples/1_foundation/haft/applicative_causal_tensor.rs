/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_haft::{Applicative, Pure};
use deep_causality_tensor::{CausalTensor, CausalTensorWitness};
use std::fmt::Debug;

// -----------------------------------------------------------------------------------------
// `Applicative` lifts a *function* into the container and applies it to a lifted value.
//
// `pure` puts either one there; `apply` brings them together. Chaining `apply` over a curried
// function is how an applicative takes more than one argument. The payload here is `i32`,
// because the subject is function application rather than arithmetic precision.
// -----------------------------------------------------------------------------------------

fn main() {
    // 1. A pure value, and a pure function, both lifted into CausalTensor.
    let value_a = 5;
    let tensor_a: CausalTensor<i32> = CausalTensorWitness::pure(value_a);
    let add_two = |x: i32| x + 2;
    let tensor_f: CausalTensor<fn(i32) -> i32> = CausalTensorWitness::pure(add_two);
    print_pure(value_a, &tensor_a, &tensor_f);

    // 2. `apply` brings the two together.
    let result_tensor: CausalTensor<i32> = CausalTensorWitness::apply(tensor_f, tensor_a);
    print_apply(value_a, &result_tensor);
    assert_eq!(result_tensor.data(), &[7]);

    // 3. A curried function takes its arguments one `apply` at a time.
    let add_all = |a: i32| move |b: i32| move |c: i32| a + b + c;
    let tensor_add_all_fn = CausalTensorWitness::pure(add_all);
    let tensor_val_x: CausalTensor<i32> = CausalTensorWitness::pure(10);
    let tensor_val_y: CausalTensor<i32> = CausalTensorWitness::pure(20);
    let tensor_val_z: CausalTensor<i32> = CausalTensorWitness::pure(30);

    let res1 = CausalTensorWitness::apply(tensor_add_all_fn, tensor_val_x);
    let res2 = CausalTensorWitness::apply(res1, tensor_val_y);
    let final_sum_tensor: CausalTensor<i32> = CausalTensorWitness::apply(res2, tensor_val_z);
    print_curried(&final_sum_tensor);
    assert_eq!(final_sum_tensor.data(), &[60]);

    // 4. A one-function tensor broadcasts across a tensor that already holds data.
    let tensor_values =
        CausalTensor::new(vec![100, 200], vec![2]).expect("two elements in a rank-1 shape of 2");
    let add_five_fn = |x: i32| x + 5;
    let tensor_add_five_func: CausalTensor<fn(i32) -> i32> = CausalTensorWitness::pure(add_five_fn);
    let result_add_five_to_tensor: CausalTensor<i32> =
        CausalTensorWitness::apply(tensor_add_five_func, tensor_values);
    print_broadcast(&result_add_five_to_tensor);
    assert_eq!(result_add_five_to_tensor.data(), &[105, 205]);

    print_footer();
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_pure<V: Debug, F: Debug>(value: i32, tensor_a: &V, tensor_f: &F) {
    println!("--- Applicative Example: CausalTensor ---");
    println!("Tensor A (pure {value}): {tensor_a:?}");
    println!("Tensor F (pure add_two function): {tensor_f:?}");
}

fn print_apply(value: i32, result: &CausalTensor<i32>) {
    println!("Result Tensor (add_two to {value}): {result:?}");
}

fn print_curried(total: &CausalTensor<i32>) {
    println!("Final Sum Tensor (10 + 20 + 30): {total:?}");
}

fn print_broadcast(result: &CausalTensor<i32>) {
    println!("Result of adding 5 to [100, 200]: {result:?}");
}

fn print_footer() {
    println!("\nApplicative example finished successfully!");
}
