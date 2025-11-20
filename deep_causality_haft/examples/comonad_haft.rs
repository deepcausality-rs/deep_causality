/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_haft::{BoxWitness, CoMonad, HKT};

fn main() {
    println!("--- CoMonad Example: BoxWitness ---");

    // 1. Example for `extract`
    let box_int = Box::new(42);
    println!("\nOriginal Box: {:?}", box_int);

    let extracted_value = BoxWitness::extract(&box_int);
    println!("Extracted value: {}", extracted_value);
    assert_eq!(extracted_value, 42);

    let box_string = Box::new("hello".to_string());
    println!("\nOriginal Box: {:?}", box_string);
    let extracted_string = BoxWitness::extract(&box_string);
    println!("Extracted string: {}", extracted_string);
    assert_eq!(extracted_string, "hello".to_string());

    // 2. Example for `extend`
    let box_val = Box::new(5);
    println!("\nOriginal Box for extend: {:?}", box_val);

    // Define a function that operates on the Box context
    // This function calculates the square of the value inside the box
    let f_square = |b: &<BoxWitness as HKT>::Type<i32>| (**b) * (**b);
    let extended_square = BoxWitness::extend(&box_val, f_square);
    println!("Extended (square): {:?}", extended_square);
    assert_eq!(extended_square, Box::new(25));

    // Another example for extend: convert to string representation within the Box context
    let f_to_string = |b: &<BoxWitness as HKT>::Type<i32>| format!("Value is: {}", **b);
    let extended_string_box = BoxWitness::extend(&box_val, f_to_string);
    println!("Extended (to string): {:?}", extended_string_box);
    assert_eq!(extended_string_box, Box::new("Value is: 5".to_string()));

    // Chaining extend operations
    let box_start = Box::new(2);
    println!("\nChaining extend: Initial Box: {:?}", box_start);

    let extended_add_one = BoxWitness::extend(&box_start, |b| **b + 1);
    println!("Extended (add one): {:?}", extended_add_one);
    assert_eq!(extended_add_one, Box::new(3));

    let extended_times_ten = BoxWitness::extend(&extended_add_one, |b| **b * 10);
    println!("Extended (times ten): {:?}", extended_times_ten);
    assert_eq!(extended_times_ten, Box::new(30));

    println!("\nCoMonad Example finished successfully!");
}
