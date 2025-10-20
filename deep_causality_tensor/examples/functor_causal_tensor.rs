/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_haft::{Functor, HKT, OptionWitness, ResultWitness};
use deep_causality_tensor::{CausalTensor, CausalTensorWitness};

fn triple_value<F>(m_a: F::Type<i32>) -> F::Type<i32>
where
    F: Functor<F> + HKT,
{
    F::fmap(m_a, |x| x * 3)
}

fn main() {
    println!("--- Functor Example: Tripling values in different containers ---");

    // Using triple_value with Option
    let opt = Some(5);
    println!("Original Option: {:?}", opt);
    let proc_opt = triple_value::<OptionWitness>(opt);
    println!("Tripled Option: {:?}", proc_opt);
    assert_eq!(proc_opt, Some(15));

    // Using triple_value with Result
    let res = Ok(5);
    println!("Original Result: {:?}", res);
    let proc_res = triple_value::<ResultWitness<i32>>(res);
    println!("Tripled Result: {:?}", proc_res);
    assert_eq!(proc_res, Ok(15));

    // Using triple_value with CausalTensor
    let tensor = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();
    println!("Original CausalTensor: {:?}", tensor);
    let proc_tensor = triple_value::<CausalTensorWitness>(tensor);
    println!("Tripled CausalTensor: {:?}", proc_tensor);
    assert_eq!(proc_tensor.data(), &[3, 6, 9]);
}
