/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_haft::{
    BoxWitness, Functor, HKT, OptionWitness, ResultWitness, VecDequeWitness, VecWitness,
};
use std::collections::VecDeque;

// A generic function that doubles the value inside *any* Functor
fn double_value<F>(m_a: F::Type<i32>) -> F::Type<i32>
where
    F: Functor<F> + HKT,
{
    F::fmap(m_a, |x| x * 2)
}

fn main() {
    println!("--- Functor Example: Doubling values in different containers ---");

    // Using double_value with Option
    let opt = Some(5);
    println!("Original Option: {:?}", opt);
    let doubled_opt = double_value::<OptionWitness>(opt);
    println!("Doubled Option: {:?}", doubled_opt);
    assert_eq!(doubled_opt, Some(10));

    // Using double_value with Result
    let res = Ok(5);
    println!("Original Result: {:?}", res);
    let doubled_res = double_value::<ResultWitness<i32>>(res);
    println!("Doubled Result: {:?}", doubled_res);
    assert_eq!(doubled_res, Ok(10));

    // Using double_value with Box
    let b = Box::new(7);
    println!("Original Box: {:?}", b);
    let doubled_box = double_value::<BoxWitness>(b);
    println!("Doubled Box: {:?}", doubled_box);
    assert_eq!(doubled_box, Box::new(14));

    // Using double_value with Vec
    let vec = vec![1, 2, 3];
    println!("Original Vec: {:?}", vec);
    let doubled_vec = double_value::<VecWitness>(vec);
    println!("Doubled Vec: {:?}", doubled_vec);
    assert_eq!(doubled_vec, vec![2, 4, 6]);

    // Using double_value with VecDeque
    let vec_dec = VecDeque::<i32>::from(vec![2, 4, 6]);
    println!("Original VecDec: {:?}", vec_dec);
    let doubled_vec_dec = double_value::<VecDequeWitness>(vec_dec);
    println!("Doubled VecDec: {:?}", doubled_vec_dec);
    assert_eq!(doubled_vec_dec, vec![4, 8, 12]);

    println!("\nExample finished successfully!");
}
