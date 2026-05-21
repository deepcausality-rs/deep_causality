/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::utils_tests::{MyCustomEffectType, MyEffect, MyMonadEffect3};
use deep_causality_haft::{Effect3, HKT3, MonadEffect3};
use deep_causality_tensor::CausalTensor;

fn main() {
    println!("--- Functional Composition with CausalTensor and Effect System ---");
    println!();

    // 1. Define the specific effect type we'll be working with
    // This effect type will wrap a CausalTensor<i32>
    type MyEffectTensorType<T> = <<MyEffect as Effect3>::HktWitness as HKT3<
        <MyEffect as Effect3>::Fixed1,
        <MyEffect as Effect3>::Fixed2,
    >>::Type<T>;

    // Initial CausalTensor
    let initial_tensor = CausalTensor::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3]).unwrap();
    println!("Initial CausalTensor: {:?}", initial_tensor);

    // 2. Start with a pure value, lifting the CausalTensor into the effect context
    let initial_effect: MyEffectTensorType<CausalTensor<i32>> =
        MyMonadEffect3::pure(initial_tensor);
    println!(
        "Initial effect (pure CausalTensor): {:?}",
        initial_effect.value
    );
    println!();

    // 3. Define a collection of step functions
    // Each function takes an effectful CausalTensor<i32> and returns a new effectful CausalTensor<i32>

    // Step 1: Filter and Double Evens, Log Operation
    let step1 = |input_tensor: CausalTensor<i32>| {
        let mut new_data = Vec::new();
        let mut warnings = Vec::new();
        for &val in input_tensor.as_slice() {
            if val % 2 == 0 {
                new_data.push(val * 2);
                warnings.push(format!("Doubled even number: {}", val));
            } else {
                warnings.push(format!("Filtered odd number: {}", val));
            }
        }
        let len = new_data.len();
        warnings.push("Trace: Executing Step 1 (Filter & Double Evens)".to_string());
        MyCustomEffectType {
            value: CausalTensor::new(new_data, vec![len]).unwrap(), // Result is 1D after filtering/flattening
            error: None,
            warnings,
        }
    };

    // Step 2: Add 5 to each element, Add Trace
    let step2 = |input_tensor: CausalTensor<i32>| {
        let new_data: Vec<i32> = input_tensor.data().iter().map(|x| x + 5).collect();
        let len = new_data.len();
        let warnings = Vec::from([
            "Added 5 to each element.".to_string(),
            "Trace: Executing Step 2 (Add 5)".to_string(),
        ]);

        MyCustomEffectType {
            value: CausalTensor::new(new_data, vec![len]).unwrap(),
            error: None,
            warnings,
        }
    };

    // Step 3: Conditional Error, Multiply by 3
    let step3 = |input_tensor: CausalTensor<i32>| {
        let mut error = None;
        let mut new_data: Vec<i32> = Vec::new();
        let mut warnings = Vec::new();

        for val in input_tensor.data().iter() {
            if *val > 20 {
                error = Some(format!("Error: Value {} exceeded threshold 20.", val));
                warnings.push(format!("Error condition met for value: {}", val));
                // If an error occurs, we might want to stop processing or return a default/empty tensor
                // For this example, we'll still process other values but set the error flag.
                new_data.push(*val); // Keep original value if error, or some default
            } else {
                new_data.push(val * 3);
                warnings.push(format!("Multiplied by 3: {}", val));
            }
        }

        let len = new_data.len();
        warnings.push("Trace: Executing Step 3 (Conditional Error & Multiply by 3)".to_string());
        MyCustomEffectType {
            value: CausalTensor::new(new_data, vec![len]).unwrap(),
            error,
            warnings,
        }
    };

    // 4. Chain Operations using Monad::bind
    println!("Processing steps...");
    let final_effect = MyMonadEffect3::bind(initial_effect, step1);
    let final_effect = MyMonadEffect3::bind(final_effect, step2);
    let final_effect = MyMonadEffect3::bind(final_effect, step3);

    println!();
    println!("--- Final Result ---");
    println!("Final CausalTensor: {:?}", final_effect.value);
    println!("Error: {:?}", final_effect.error);
    println!("Warnings: {:?}", final_effect.warnings);

    assert_eq!(final_effect.value.as_slice(), &[27, 39, 51]);
    assert!(final_effect.error.is_none());
    assert!(!final_effect.warnings.is_empty());

    println!(
        "
Example finished successfully!"
    );
}
