/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::{
    CausalArrow, CausalFlow, CausalityError, CausalityErrorEnum, causal_arrow,
};
use deep_causality_haft::Arrow;
use std::cell::Cell;

fn err(msg: &str) -> CausalityError {
    CausalityError::new(CausalityErrorEnum::Custom(msg.into()))
}

#[test]
fn arrow_is_reusable_across_inputs() {
    let inc = causal_arrow(|x: i64| CausalFlow::value(x + 1));
    // `run` takes `&self`, so the same arrow applies to many inputs.
    assert_eq!(inc.run(3).finish(), Ok(4));
    assert_eq!(inc.run(10).finish(), Ok(11));
}

#[test]
fn arrow_application_equals_the_monad_pipeline() {
    let arrow = causal_arrow(|x: i64| CausalFlow::value(x + 1));
    let via_arrow = arrow.run(7).finish();
    let via_monad = CausalFlow::value(7_i64).map(|x| x + 1).finish();
    assert_eq!(via_arrow, via_monad);
}

#[test]
fn next_composes_by_binding() {
    let arrow = causal_arrow(|x: i64| CausalFlow::value(x + 1))
        .next(|x| CausalFlow::value(x * 2))
        .build();
    // (3 + 1) * 2
    assert_eq!(arrow.run(3).finish(), Ok(8));
}

#[test]
fn next_short_circuits_second_stage_on_first_error() {
    let ran = Cell::new(false);
    let arrow = causal_arrow(|_x: i64| CausalFlow::<i64>::fail(err("first stage failed")))
        .next(|x| {
            ran.set(true);
            CausalFlow::value(x + 1)
        })
        .build();
    let out = arrow.run(0);
    assert!(out.is_err());
    assert!(
        !ran.get(),
        "second stage must not run after a first-stage error"
    );
}

#[test]
fn three_pipelines_compose_into_one() {
    // ((3 + 1) * 2) - 3 == 5
    let composite = causal_arrow(|x: i64| CausalFlow::value(x + 1))
        .next(|x| CausalFlow::value(x * 2))
        .next(|x| CausalFlow::value(x - 3))
        .build();
    assert_eq!(composite.run(3).finish(), Ok(5));
    // Reusable: a second input runs the same composite.
    assert_eq!(composite.run(10).finish(), Ok(19));
}

#[test]
fn builder_run_terminal_applies_the_chain() {
    let out = causal_arrow(|x: i64| CausalFlow::value(x + 1))
        .next(|x| CausalFlow::value(x * 2))
        .run(3);
    assert_eq!(out.finish(), Ok(8));
}

#[test]
fn and_then_applies_a_reified_arrow() {
    // The bridge from a live flow to a held-as-data composite: no dedicated `pipe`, just `and_then`.
    let arrow = causal_arrow(|x: i64| CausalFlow::value(x * 10)).build();
    let out = CausalFlow::value(2_i64).and_then(|v| arrow.run(v)).finish();
    assert_eq!(out, Ok(20));
}

#[test]
fn causal_arrow_marker_trait_is_a_usable_bound() {
    fn run_arrow<A: CausalArrow<i64, i64>>(a: &A, x: i64) -> Result<i64, CausalityError> {
        a.run(x).finish()
    }
    let a = causal_arrow(|x: i64| CausalFlow::value(x + 1)).build();
    assert_eq!(run_arrow(&a, 5), Ok(6));
}
