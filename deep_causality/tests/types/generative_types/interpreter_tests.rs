/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{
    BaseSymbol, CausalSystemState, Data, EuclideanSpace, EuclideanSpacetime, EuclideanTime,
    Interpreter,
};

// Type aliases for testing
type TestData = Data<f64>;
type TestSpace = EuclideanSpace;
type TestTime = EuclideanTime;
type TestSpacetime = EuclideanSpacetime;
type TestSymbol = BaseSymbol;

#[test]
fn test_causal_system_state_new() {
    let state = CausalSystemState::<
        (),
        (),
        TestData,
        TestSpace,
        TestTime,
        TestSpacetime,
        TestSymbol,
        f64,
        f64,
    >::new();

    assert_eq!(state.causaloids.len(), 0);
    assert_eq!(state.contexts.len(), 0);
}

#[test]
fn test_causal_system_state_clone() {
    let state1 = CausalSystemState::<
        (),
        (),
        TestData,
        TestSpace,
        TestTime,
        TestSpacetime,
        TestSymbol,
        f64,
        f64,
    >::new();

    let state2 = state1.clone();

    assert_eq!(state1.causaloids.len(), state2.causaloids.len());
    assert_eq!(state1.contexts.len(), state2.contexts.len());
}

#[test]
fn test_interpreter_new() {
    let interpreter = Interpreter;
    // Interpreter is a zero-sized type, just verify it can be created
    let _ = interpreter;
}

#[test]
fn test_interpreter_is_stateless() {
    let interpreter1 = Interpreter;
    let interpreter2 = Interpreter;

    // Both interpreters should be identical (stateless)
    let _ = (interpreter1, interpreter2);
}

#[test]
fn test_causal_system_state_debug() {
    let state = CausalSystemState::<
        (),
        (),
        TestData,
        TestSpace,
        TestTime,
        TestSpacetime,
        TestSymbol,
        f64,
        f64,
    >::new();

    let debug_str = format!("{:?}", state);
    assert!(debug_str.contains("CausalSystemState"));
}
