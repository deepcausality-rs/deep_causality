/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::utils_tests::{MyCustomEffectType, MyEffect, MyMonadEffect3};
use deep_causality_haft::{Effect3, HKT3, MonadEffect3};
use deep_causality_tensor::CausalTensor;
use std::fmt::Debug;

fn main() {
    print_header();

    // 1. Define the specific effect type we'll be working with
    // This effect type will wrap a CausalTensor<i32>
    type MyEffectTensorType<T> = <<MyEffect as Effect3>::HktWitness as HKT3<
        <MyEffect as Effect3>::Fixed1,
        <MyEffect as Effect3>::Fixed2,
    >>::Type<T>;

    // Initial CausalTensor
    let initial_tensor =
        CausalTensor::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3]).expect("six elements in a 2x3 shape");
    let shown_tensor = initial_tensor.clone();

    // 2. Start with a pure value, lifting the CausalTensor into the effect context
    let initial_effect: MyEffectTensorType<CausalTensor<i32>> =
        MyMonadEffect3::pure(initial_tensor);
    print_initial(&shown_tensor, &initial_effect.value);

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
            value: Some(
                CausalTensor::new(new_data, vec![len])
                    .expect("len elements in a rank-1 shape of len"),
            ), // Result is 1D after filtering/flattening
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
            value: Some(
                CausalTensor::new(new_data, vec![len])
                    .expect("len elements in a rank-1 shape of len"),
            ),
            error: None,
            warnings,
        }
    };

    // Step 3: Conditional Error, Multiply by 3
    let step3 = |input_tensor: CausalTensor<i32>| {
        let mut new_data: Vec<i32> = Vec::new();
        let mut warnings = Vec::new();

        for val in input_tensor.data().iter() {
            if *val > 20 {
                warnings.push(format!("Error condition met for value: {}", val));
                warnings.push(
                    "Trace: Executing Step 3 (Conditional Error & Multiply by 3)".to_string(),
                );
                // A failing step returns a well-formed errored carrier: an error and no
                // value. Any bind chained after this one short-circuits.
                return MyCustomEffectType {
                    value: None,
                    error: Some(format!("Error: Value {} exceeded threshold 20.", val)),
                    warnings,
                };
            }
            new_data.push(val * 3);
            warnings.push(format!("Multiplied by 3: {}", val));
        }

        let len = new_data.len();
        warnings.push("Trace: Executing Step 3 (Conditional Error & Multiply by 3)".to_string());
        MyCustomEffectType {
            value: Some(
                CausalTensor::new(new_data, vec![len])
                    .expect("len elements in a rank-1 shape of len"),
            ),
            error: None,
            warnings,
        }
    };

    // 4. Chain Operations using Monad::bind
    print_processing();
    let final_effect = MyMonadEffect3::bind(initial_effect, step1);
    let final_effect = MyMonadEffect3::bind(final_effect, step2);
    let final_effect = MyMonadEffect3::bind(final_effect, step3);

    print_result(
        &final_effect.value,
        &final_effect.error,
        &final_effect.warnings,
    );

    assert_eq!(
        final_effect
            .value
            .as_ref()
            .expect("the chain carries a value")
            .as_slice(),
        &[27, 39, 51]
    );
    assert!(final_effect.error.is_none());
    assert!(!final_effect.warnings.is_empty());

    print_footer();
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("--- Functional Composition with CausalTensor and Effect System ---");
    println!();
}

fn print_initial<T: Debug, E: Debug>(tensor: &T, lifted: &E) {
    println!("Initial CausalTensor: {tensor:?}");
    println!("Initial effect (pure CausalTensor): {lifted:?}");
    println!();
}

fn print_processing() {
    println!("Processing steps...");
}

fn print_result<V: Debug, E: Debug, W: Debug>(value: &V, error: &E, warnings: &W) {
    println!();
    println!("--- Final Result ---");
    println!("Final CausalTensor: {value:?}");
    println!("Error: {error:?}");
    println!("Warnings: {warnings:?}");
}

fn print_footer() {
    println!(
        "
Example finished successfully!"
    );
}
