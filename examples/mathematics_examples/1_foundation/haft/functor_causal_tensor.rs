/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_haft::{Functor, HKT, OptionWitness, ResultWitness};
use deep_causality_num::lift;
use deep_causality_tensor::{CausalTensor, CausalTensorWitness};
use std::fmt::Debug;

/// One function, any functor. The witness picks the container at the call site.
fn triple_value<F>(m_a: F::Type<FloatType>) -> F::Type<FloatType>
where
    F: Functor<F> + HKT,
{
    F::fmap(m_a, |x| x * lift::<FloatType>(3.0))
}

/// The working scalar. One `Functor` written against it serves every container below.
pub type FloatType = f64;

fn main() {
    print_header();

    let opt = Some(lift::<FloatType>(5.0));
    let proc_opt = triple_value::<OptionWitness>(opt);
    print_case("Option", &opt, &proc_opt);
    assert_eq!(proc_opt, Some(lift::<FloatType>(15.0)));

    // `ResultWitness<E>` pins the error type, so both sides are the working scalar here.
    let res: Result<FloatType, FloatType> = Ok(lift(5.0));
    let proc_res = triple_value::<ResultWitness<FloatType>>(res);
    print_case("Result", &res, &proc_res);
    assert_eq!(proc_res, Ok(lift::<FloatType>(15.0)));

    let tensor = CausalTensor::new(vec![lift::<FloatType>(1.0), lift(2.0), lift(3.0)], vec![3])
        .expect("three elements in a rank-1 shape of 3");
    let original = tensor.clone();
    let proc_tensor = triple_value::<CausalTensorWitness>(tensor);
    print_case("CausalTensor", &original, &proc_tensor);
    assert_eq!(
        proc_tensor.data(),
        &[lift::<FloatType>(3.0), lift(6.0), lift(9.0)]
    );
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("--- Functor Example: Tripling values in different containers ---");
}

fn print_case<T: Debug>(container: &str, original: &T, tripled: &T) {
    println!("Original {container}: {original:?}");
    println!("Tripled {container}: {tripled:?}");
}
