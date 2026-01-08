/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::*;

fn get_context() -> BaseContext {
    let id = 1;
    let name = "base context";
    Context::with_capacity(id, name, 10)
}

#[test]
fn test_set_get_data_index_direct() {
    let mut ctx = get_context();

    ctx.set_data_index(0, 0, true);
    let res = ctx.get_data_index(&0, true);
    assert!(res.is_some());
    let result = res.unwrap();
    assert_eq!(result, &0);

    ctx.set_data_index(0, 42, false);

    let res = ctx.get_data_index(&0, false);
    assert!(res.is_some());
    let result = res.unwrap();
    assert_eq!(result, &42);
}

#[test]
fn test_set_get_current_data_index() {
    let mut ctx = get_context();

    ctx.set_current_data_index(42);

    let res = ctx.get_current_data_index();
    assert!(res.is_some());
    let result = res.unwrap();
    assert_eq!(result, &42);
}

#[test]
fn test_set_get_previous_data_index() {
    let mut ctx = get_context();

    ctx.set_previous_data_index(23);
    let res = ctx.get_previous_data_index();
    assert!(res.is_some());
    let result = res.unwrap();
    assert_eq!(result, &23);
}
