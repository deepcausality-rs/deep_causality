/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{Functor, Monad, OptionWitness, ResultWitness};

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
}
