/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::prelude::{
    BaseContext, Context, Contextoid, ContextoidType, ExtendableContextuableGraph, Root,
};

fn get_context() -> BaseContext {
    let id = 1;
    let name = "base context";
    Context::with_capacity(id, name, 10)
}

#[test]
fn test_extra_ctx_add_new_with_id_err() {
    let mut context = get_context();
    let id = 1;
    let capacity = 10;
    let default = true;
    let res = context.extra_ctx_add_new_with_id(id, capacity, default);
    assert!(res.is_ok());

    let res = context.extra_ctx_add_new_with_id(id, capacity, default);
    assert!(res.is_err());
}

#[test]
fn test_extra_ctx_unset_current_id_err() {
    let mut context = get_context();
    let res = context.extra_ctx_unset_current_id();
    assert!(res.is_err());
}

#[test]
fn test_extra_ctx_add_node_err() {
    let mut context = get_context();
    let id = 1;
    let contextoid = Contextoid::new(id, ContextoidType::Root(Root::new(id)));
    let res = context.extra_ctx_add_node(contextoid);
    assert!(res.is_err());
}
