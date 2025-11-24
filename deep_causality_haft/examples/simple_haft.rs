/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{Applicative, Bifunctor, Functor, Monad};
use deep_causality_haft::{OptionWitness, ResultUnboundWitness, ResultWitness};

fn main() {
    println!("--- Option Example ---");

    let opt_a = Some(5);
    let f_map = |x| x * 2;
    let opt_b = OptionWitness::fmap(opt_a, f_map);
    println!("fmap(Some(5), |x| x * 2) = {:?}", opt_b);
    assert_eq!(opt_b, Some(10));

    let f_bind = |x| Some(x + 1);
    let opt_c = OptionWitness::bind(opt_b, f_bind);
    println!("bind(Some(10), |x| Some(x + 1)) = {:?}", opt_c);
    assert_eq!(opt_c, Some(11));

    let pure_val_opt = OptionWitness::pure(100);
    println!("pure(100) = {:?}", pure_val_opt);
    assert_eq!(pure_val_opt, Some(100));

    println!("\n--- Result Example ---");

    type MyResult<T> = Result<T, String>;

    let res_a: MyResult<i32> = Ok(10);
    let f_map_res = |x: i32| x.to_string();
    let res_b = ResultWitness::fmap(res_a, f_map_res);
    println!("fmap(Ok(10), |x| x.to_string()) = {:?}", res_b);
    assert_eq!(res_b, Ok("10".to_string()));

    let f_bind_res = |s: String| Ok(s.len());
    let res_c = ResultWitness::bind(res_b, f_bind_res);
    println!("bind(Ok(\"10\"), |s| Ok(s.len())) = {:?}", res_c);
    assert_eq!(res_c, Ok(2));

    let res_err: MyResult<i32> = Err("An error occurred".to_string());
    let res_err_mapped = ResultWitness::fmap(res_err.clone(), f_map_res);
    println!("fmap(Err(...), |x| x.to_string()) = {:?}", res_err_mapped);
    assert_eq!(res_err_mapped, Err("An error occurred".to_string()));

    let res_err_bound = ResultWitness::bind(res_err, |x| Ok(x + 1));
    println!("bind(Err(...), |x| Ok(x + 1)) = {:?}", res_err_bound);
    assert_eq!(res_err_bound, Err("An error occurred".to_string()));

    let pure_val_res: MyResult<i32> = ResultWitness::pure(200);
    println!("pure(200) = {:?}", pure_val_res);
    assert_eq!(pure_val_res, Ok(200));

    println!("\n--- Unbound HKT Example (Bifunctor) ---");
    // Bifunctor allows mapping BOTH the success (Ok) and failure (Err) types simultaneously.
    // This is impossible with standard Functor which only maps the 'Ok' value.

    let res_ok: Result<i32, &str> = Ok(10);
    let res_err: Result<i32, &str> = Err("error");

    // Map Ok: i32 -> f64 (x * 2.5)
    // Map Err: &str -> usize (len)
    let mapped_ok = ResultUnboundWitness::bimap(res_ok, |x| x as f64 * 2.5, |e| e.len());
    let mapped_err = ResultUnboundWitness::bimap(res_err, |x| x as f64 * 2.5, |e| e.len());

    println!("bimap(Ok(10)) -> {:?}", mapped_ok);
    assert_eq!(mapped_ok, Ok(25.0));

    println!("bimap(Err(\"error\")) -> {:?}", mapped_err);
    assert_eq!(mapped_err, Err(5));
}
