/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The sweep combinator: input order, first-error semantics, empty input, output
//! genericity (a pointwise body with no march), and run-to-run determinism.

use deep_causality_cfd::sweep;

#[test]
fn outputs_arrive_in_input_order() {
    let items = [3.0_f64, 1.0, 2.0];
    let out: Vec<f64> = sweep(&items, |&x| Ok::<f64, ()>(x * 10.0)).expect("all ok");
    assert_eq!(out, vec![30.0, 10.0, 20.0]);
}

#[test]
fn the_first_error_in_input_order_wins() {
    let items = [1_i64, -2, 3, -4];
    let err = sweep(&items, |&x| {
        if x < 0 {
            Err(format!("negative: {x}"))
        } else {
            Ok(x)
        }
    })
    .expect_err("has failures");
    assert_eq!(err, "negative: -2");
}

#[test]
fn an_empty_sweep_is_ok_and_empty() {
    let items: [f64; 0] = [];
    let out: Vec<f64> = sweep(&items, |&x| Ok::<f64, ()>(x)).expect("empty ok");
    assert!(out.is_empty());
}

#[test]
fn a_pointwise_body_with_no_march_fits() {
    // The placard shape: rows in, computed row arrays out, no report anywhere.
    let rows = [[2.0_f64, 10_000.0], [5.0, 30_000.0]];
    let out: Vec<[f64; 3]> = sweep(&rows, |r| {
        let q = 0.5 * r[0] * r[0] / (1.0 + r[1] * 1.0e-6);
        Ok::<[f64; 3], ()>([r[0], r[1], q])
    })
    .expect("computes");
    assert_eq!(out.len(), 2);
    assert_eq!(out[0][0], 2.0);
}

#[test]
fn repeated_runs_are_identical() {
    let items: Vec<f64> = (0..64).map(|i| i as f64 * 0.1).collect();
    let f = |x: &f64| Ok::<u64, ()>((x * 1.0e6) as u64 ^ 0x5DEECE66D);
    let a = sweep(&items, f).expect("ok");
    let b = sweep(&items, f).expect("ok");
    assert_eq!(a, b, "the sweep is deterministic run to run");
}
