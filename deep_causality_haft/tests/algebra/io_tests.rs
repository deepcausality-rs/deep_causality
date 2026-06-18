/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use core::cell::Cell;
use deep_causality_haft::{IoAction, io_fail, io_pure};

// An in-memory IO action that records each execution in a shared counter. It lets the tests prove
// laziness (the counter stays 0 until `run`) and one-shot execution (it ticks exactly once per run).
struct Tick<'a>(&'a Cell<u32>, i32);

impl IoAction for Tick<'_> {
    type Output = i32;
    type Error = String;

    fn run(self) -> Result<i32, String> {
        self.0.set(self.0.get() + 1);
        Ok(self.1)
    }
}

// --- run / laziness ---

#[test]
fn test_pure_runs_with_no_effect() {
    let action = io_pure::<i32, String>(42);
    assert_eq!(action.run(), Ok(42));
}

#[test]
fn test_fail_runs_to_error() {
    let action = io_fail::<i32, String>("boom".to_string());
    assert_eq!(action.run(), Err("boom".to_string()));
}

#[test]
fn test_construction_and_composition_perform_no_effect() {
    let ticks = Cell::new(0);
    // Build and compose, but do not run.
    let _action = Tick(&ticks, 5).map(|x| x + 1).and_then(|x| io_pure(x * 2));
    assert_eq!(ticks.get(), 0, "no effect must occur before run");
}

#[test]
fn test_run_executes_the_effect_exactly_once() {
    let ticks = Cell::new(0);
    let out = Tick(&ticks, 5).run();
    assert_eq!(out, Ok(5));
    assert_eq!(ticks.get(), 1);
}

// --- map / map_err ---

#[test]
fn test_map_transforms_success() {
    let ticks = Cell::new(0);
    let out = Tick(&ticks, 5).map(|x| x + 100).run();
    assert_eq!(out, Ok(105));
    assert_eq!(ticks.get(), 1);
}

#[test]
fn test_map_leaves_error_untouched() {
    let out = io_fail::<i32, String>("e".to_string()).map(|x| x + 1).run();
    assert_eq!(out, Err("e".to_string()));
}

#[test]
fn test_map_err_transforms_error() {
    let out = io_fail::<i32, String>("e".to_string())
        .map_err(|e| format!("wrapped: {e}"))
        .run();
    assert_eq!(out, Err("wrapped: e".to_string()));
}

#[test]
fn test_map_err_leaves_success_untouched() {
    let out = io_pure::<i32, String>(7).map_err(|e| format!("x{e}")).run();
    assert_eq!(out, Ok(7));
}

// --- and_then / short-circuit ---

#[test]
fn test_and_then_chains_dependent_action() {
    let out = io_pure::<i32, String>(5)
        .and_then(|x| io_pure::<i32, String>(x * 2))
        .run();
    assert_eq!(out, Ok(10));
}

#[test]
fn test_and_then_short_circuits_on_first_error() {
    let ticks = Cell::new(0);
    let out = io_fail::<i32, String>("boom".to_string())
        .and_then(|x| Tick(&ticks, x))
        .run();
    assert_eq!(out, Err("boom".to_string()));
    assert_eq!(
        ticks.get(),
        0,
        "the second action must not run after a failure"
    );
}

// --- monad laws (compared on run output) ---

#[test]
fn test_left_identity() {
    let f = |x: i32| io_pure::<i32, String>(x + 3);
    let lhs = io_pure::<i32, String>(10).and_then(f).run();
    let rhs = f(10).run();
    assert_eq!(lhs, rhs);
}

#[test]
fn test_right_identity() {
    let lhs = io_pure::<i32, String>(10)
        .and_then(io_pure::<i32, String>)
        .run();
    let rhs = io_pure::<i32, String>(10).run();
    assert_eq!(lhs, rhs);
}

#[test]
fn test_associativity() {
    let f = |x: i32| io_pure::<i32, String>(x + 1);
    let g = |x: i32| io_pure::<i32, String>(x * 2);

    let left = io_pure::<i32, String>(5).and_then(f).and_then(g).run();
    let right = io_pure::<i32, String>(5)
        .and_then(|x| f(x).and_then(g))
        .run();
    assert_eq!(left, right);
}
