/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::utils_tests::*;
use deep_causality_haft::{Effect5, HKT5, MonadEffect5};

fn main() {
    println!("--- Type-Encoded Effect System Example (Arity 5) ---");
    println!();

    // Define the specific effect type we'll be working with
    type MyEffectType<T> = <<MyEffect5 as Effect5>::HktWitness as HKT5<
        <MyEffect5 as Effect5>::Fixed1,
        <MyEffect5 as Effect5>::Fixed2,
        <MyEffect5 as Effect5>::Fixed3,
        <MyEffect5 as Effect5>::Fixed4,
    >>::Type<T>;

    // 1. Start with a pure value, lifting it into the effect context
    let initial_effect: MyEffectType<i32> = MyMonadEffect5::pure(10);
    println!("Initial effect (pure 10): {:?}", initial_effect);
    println!();

    // 2. Define a collection of step functions
    // Each function takes an i32 and returns an effectful i32
    let step_functions: Vec<Box<dyn Fn(i32) -> MyEffectType<i32>>> = vec![
        Box::new(|x: i32| MyCustomEffectType5 {
            value: x * 2,
            f1: None,
            f2: vec!["Operation A: Multiplied by 2".to_string()],
            f3: vec![1],
            f4: vec!["Trace: Executing step 1".to_string()],
        }),
        Box::new(|x: i32| MyCustomEffectType5 {
            value: x + 5,
            f1: None,
            f2: vec!["Operation B: Added 5".to_string()],
            f3: vec![1],
            f4: vec!["Trace: Executing step 2".to_string()],
        }),
        Box::new(|x: i32| MyCustomEffectType5 {
            value: x * 3,
            f1: None,
            f2: vec!["Operation C: Multiplied by 3".to_string()],
            f3: vec![1],
            f4: vec!["Trace: Executing step 3".to_string()],
        }),
    ];

    // 3. Execute all step functions in sequence
    println!("Process Steps: ");
    let mut current_effect = initial_effect;
    for (i, f) in step_functions.into_iter().enumerate() {
        let prev_logs_len = current_effect.f2.len();
        current_effect = MyMonadEffect5::bind(current_effect, f);
        for log_msg in current_effect.f2.iter().skip(prev_logs_len) {
            println!("  Log (Step {}): {}", i + 1, log_msg);
        }
    }
    println!();

    println!("Sequenced outcome: {:?}", current_effect.value);
    println!();

    // Expected final value: (10 * 2 + 5) * 3 = 75
    assert_eq!(current_effect.value, 75);
    assert!(current_effect.f1.is_none());
    assert_eq!(
        current_effect.f2,
        vec![
            "Operation A: Multiplied by 2".to_string(),
            "Operation B: Added 5".to_string(),
            "Operation C: Multiplied by 3".to_string()
        ]
    );
    assert_eq!(current_effect.f3, vec![1, 1, 1]);
    assert_eq!(
        current_effect.f4,
        vec![
            "Trace: Executing step 1".to_string(),
            "Trace: Executing step 2".to_string(),
            "Trace: Executing step 3".to_string()
        ]
    );

    println!("\n--- Example with Error Propagation ---");
    println!();

    let error_step_functions: Vec<Box<dyn Fn(i32) -> MyEffectType<i32>>> = vec![
        Box::new(|x: i32| MyCustomEffectType5 {
            value: x * 2,
            f1: None,
            f2: vec!["Error Step 1: Multiplied by 2".to_string()],
            f3: vec![1],
            f4: vec!["Trace: Executing error step 1".to_string()],
        }),
        Box::new(|_x: i32| {
            MyCustomEffectType5 {
                value: 0, // Value doesn't matter much if error occurs
                f1: Some("Error: Something went wrong!".to_string()),
                f2: vec!["Error Step 2: Failed here".to_string()],
                f3: vec![1],
                f4: vec!["Trace: Executing error step 2".to_string()],
            }
        }),
        Box::new(|x: i32| MyCustomEffectType5 {
            value: x * 100,
            f1: None,
            f2: vec!["Error Step 3: Should not run".to_string()],
            f3: vec![1],
            f4: vec!["Trace: Executing error step 3".to_string()],
        }),
    ];

    let mut current_error_effect: MyEffectType<i32> = MyMonadEffect5::pure(5);
    println!("Process Error Steps: ");
    for (i, f) in error_step_functions.into_iter().enumerate() {
        let prev_logs_len = current_error_effect.f2.len();
        current_error_effect = MyMonadEffect5::bind(current_error_effect, f);
        for log_msg in current_error_effect.f2.iter().skip(prev_logs_len) {
            println!("  Log (Error Step {}): {}", i + 1, log_msg);
        }
    }

    println!("Sequenced effect with error: {:?}", current_error_effect);
    println!();

    assert_eq!(
        current_error_effect.f1,
        Some("Error: Something went wrong!".to_string())
    );
    assert_eq!(
        current_error_effect.f2,
        vec![
            "Error Step 1: Multiplied by 2".to_string(),
            "Error Step 2: Failed here".to_string(),
        ]
    );
    assert_eq!(current_error_effect.f3, vec![1, 1]);
    assert_eq!(
        current_error_effect.f4,
        vec![
            "Trace: Executing error step 1".to_string(),
            "Trace: Executing error step 2".to_string(),
        ]
    );

    println!("\nExample finished successfully!");
}
