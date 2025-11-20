/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_haft::{HKT, OptionWitness, ResultWitness, Traversable, VecWitness};

fn main() {
    println!("--- Traversable Example: Vec<Option<A>> to Option<Vec<A>> ---");

    // Case 1: All inner Options are Some
    let vec_opt_all_some = vec![Some(1), Some(2), Some(3)];
    println!("\nOriginal Vec<Option<i32>>: {:?}", vec_opt_all_some);

    type OptionAsMonad = OptionWitness; // M is OptionWitness
    let sequenced_option_all_some: <OptionAsMonad as HKT>::Type<<VecWitness as HKT>::Type<i32>> =
        VecWitness::sequence::<i32, OptionAsMonad>(vec_opt_all_some);
    println!(
        "Sequenced Option<Vec<i32>>: {:?}",
        sequenced_option_all_some
    );
    assert_eq!(sequenced_option_all_some, Some(vec![1, 2, 3]));

    // Case 2: One inner Option is None
    let vec_opt_with_none = vec![Some(1), None, Some(3)];
    println!("\nOriginal Vec<Option<i32>>: {:?}", vec_opt_with_none);
    let sequenced_option_with_none: <OptionAsMonad as HKT>::Type<<VecWitness as HKT>::Type<i32>> =
        VecWitness::sequence::<i32, OptionAsMonad>(vec_opt_with_none);
    println!(
        "Sequenced Option<Vec<i32>>: {:?}",
        sequenced_option_with_none
    );
    assert_eq!(sequenced_option_with_none, None);

    // Case 3: Empty vector
    let empty_vec_opt: Vec<Option<i32>> = vec![];
    println!("\nOriginal empty Vec<Option<i32>>: {:?}", empty_vec_opt);
    let sequenced_empty_option: <OptionAsMonad as HKT>::Type<<VecWitness as HKT>::Type<i32>> =
        VecWitness::sequence::<i32, OptionAsMonad>(empty_vec_opt);
    println!(
        "Sequenced empty Option<Vec<i32>>: {:?}",
        sequenced_empty_option
    );
    assert_eq!(sequenced_empty_option, Some(vec![]));

    println!("\n--- Traversable Example: Vec<Result<A, E>> to Result<Vec<A>, E> ---");

    // Case 1: All inner Results are Ok
    let vec_res_all_ok: Vec<Result<i32, String>> = vec![Ok(1), Ok(2), Ok(3)];
    println!("\nOriginal Vec<Result<i32, String>>: {:?}", vec_res_all_ok);

    type ResultAsMonad = ResultWitness<String>; // M is ResultWitness<String>
    let sequenced_result_all_ok: <ResultAsMonad as HKT>::Type<<VecWitness as HKT>::Type<i32>> =
        VecWitness::sequence::<i32, ResultAsMonad>(vec_res_all_ok);
    println!(
        "Sequenced Result<Vec<i32>, String>: {:?}",
        sequenced_result_all_ok
    );
    assert_eq!(sequenced_result_all_ok, Ok(vec![1, 2, 3]));

    // Case 2: One inner Result is Err
    let vec_res_with_err: Vec<Result<i32, String>> =
        vec![Ok(1), Err("Error occurred!".to_string()), Ok(3)];
    println!(
        "\nOriginal Vec<Result<i32, String>>: {:?}",
        vec_res_with_err
    );
    let sequenced_result_with_err: <ResultAsMonad as HKT>::Type<<VecWitness as HKT>::Type<i32>> =
        VecWitness::sequence::<i32, ResultAsMonad>(vec_res_with_err);
    println!(
        "Sequenced Result<Vec<i32>, String>: {:?}",
        sequenced_result_with_err
    );
    assert_eq!(
        sequenced_result_with_err,
        Err("Error occurred!".to_string())
    );

    // Case 3: Empty vector
    let empty_vec_res: Vec<Result<i32, String>> = vec![];
    println!(
        "\nOriginal empty Vec<Result<i32, String>>: {:?}",
        empty_vec_res
    );
    let sequenced_empty_result: <ResultAsMonad as HKT>::Type<<VecWitness as HKT>::Type<i32>> =
        VecWitness::sequence::<i32, ResultAsMonad>(empty_vec_res);
    println!(
        "Sequenced empty Result<Vec<i32>, String>: {:?}",
        sequenced_empty_result
    );
    assert_eq!(sequenced_empty_result, Ok(vec![]));

    println!("\nTraversable Example finished successfully!");
}
