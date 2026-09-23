/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Named extra contexts and the allocator rule of `context-named-extra-contexts`. Expected
//! identifiers follow from the rule stated in the spec, one more than the highest held and 1 when
//! none is held; expected names are the literals handed in.
//!
//! Corner cases (rows A to K): A a context with no extras allocates 1,
//! `test_sequential_numbering_is_unchanged`; B one explicit extra then an allocation,
//! `test_an_explicit_identifier_is_never_reached`; C two explicit identifiers adjacent to each
//! other, `test_allocation_past_large_identifiers`; D the allocation lands exactly one past the
//! highest, in both tests; F identifier 0 refused, `test_identifier_zero_is_refused`; H
//! `u64::MAX - 1` as an explicit identifier followed by an allocation of `u64::MAX`,
//! `test_allocation_past_large_identifiers`; every other row n/a.

use deep_causality_context::{BaseContext, Context, ExtendableContextuableGraph};

fn context() -> BaseContext {
    Context::with_capacity(1, "base", 10)
}

#[test]
fn test_a_name_is_kept_with_the_extra() {
    let mut ctx = context();
    let weather = ctx.extra_ctx_add_new("weather", 4, true);
    ctx.extra_ctx_add_new_with_id(7, "terrain", 4, false)
        .unwrap();
    assert_eq!(ctx.extra_ctx_get_name(weather), Some("weather"));
    assert_eq!(ctx.extra_ctx_get_name(7), Some("terrain"));
    assert_eq!(ctx.extra_ctx_get_name(weather + 1_000), None);
    assert_eq!(ctx.extra_ctx_get_name(0), None);
}

#[test]
fn test_names_survive_a_clone() {
    let mut ctx = context();
    let id = ctx.extra_ctx_add_new("weather", 4, true);
    let copy = ctx.clone();
    assert_eq!(copy.extra_ctx_get_name(id), Some("weather"));
    assert_eq!(copy.extra_ctx_get_current_id(), id);
}

#[test]
fn test_identifier_zero_is_refused() {
    let mut ctx = context();
    assert!(ctx.extra_ctx_add_new_with_id(0, "none", 4, false).is_err());
    assert!(!ctx.extra_ctx_check_exists(0));
    assert_eq!(ctx.extra_ctx_get_name(0), None);
}

#[test]
fn test_sequential_numbering_is_unchanged() {
    let mut ctx = context();
    assert_eq!(ctx.extra_ctx_add_new("a", 4, false), 1);
    assert_eq!(ctx.extra_ctx_add_new("b", 4, false), 2);
    assert_eq!(ctx.extra_ctx_add_new("c", 4, false), 3);
    assert_eq!(ctx.extra_ctx_get_name(2), Some("b"));
}

#[test]
fn test_an_explicit_identifier_is_never_reached() {
    let mut ctx = context();
    ctx.extra_ctx_add_new_with_id(7, "seven", 4, false).unwrap();
    assert_eq!(ctx.extra_ctx_add_new("eight", 4, false), 8);
    assert_eq!(ctx.extra_ctx_add_new("nine", 4, false), 9);
    assert!(ctx.extra_ctx_check_exists(7));
    assert!(ctx.extra_ctx_check_exists(8));
    assert!(ctx.extra_ctx_check_exists(9));
    assert!(ctx.extra_ctx_add_new_with_id(8, "again", 4, false).is_err());
}

#[test]
fn test_allocation_past_large_identifiers() {
    let mut ctx = context();
    ctx.extra_ctx_add_new_with_id(40, "forty", 4, false)
        .unwrap();
    ctx.extra_ctx_add_new_with_id(41, "forty-one", 4, false)
        .unwrap();
    assert_eq!(ctx.extra_ctx_add_new("forty-two", 4, false), 42);
    let mut top = context();
    top.extra_ctx_add_new_with_id(u64::MAX - 1, "penultimate", 4, false)
        .unwrap();
    assert_eq!(top.extra_ctx_add_new("last", 4, false), u64::MAX);
}
