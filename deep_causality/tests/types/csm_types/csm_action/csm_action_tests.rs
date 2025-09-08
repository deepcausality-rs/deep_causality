/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{ActionError, CausalAction};

fn hello_state() -> Result<(), ActionError> {
    println!("Hello State");

    Ok(())
}

fn get_test_action() -> CausalAction {
    let func = hello_state;
    let descr = "Test action that prints Hello State";
    let version = 1;

    CausalAction::new(func, descr, version)
}

#[test]
fn test_new() {
    let ca = get_test_action();

    assert_eq!(ca.descr(), "Test action that prints Hello State");
    assert_eq!(ca.version(), 1);
}

#[test]
fn test_fire() {
    let ca = get_test_action();

    let res = ca.fire();

    assert!(res.is_ok());
    assert_eq!(ca.descr(), "Test action that prints Hello State");
    assert_eq!(ca.version(), 1);
}

#[test]
fn test_display() {
    let func = hello_state;
    let descr = "Test action that prints Hello State";
    let version = 1;

    let ca = CausalAction::new(func, descr, version);
    let expected =
        "CausalAction { descr: \"Test action that prints Hello State\", version: 1 }".to_string();
    assert_eq!(ca.to_string(), expected);
}
