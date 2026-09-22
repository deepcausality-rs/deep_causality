/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Expected values are the literals handed to the constructor, read back in order; `remaining` is
//! asserted at every step so an off-by-one in either direction is caught.
//!
//! Corner cases (rows A to K): A empty reserve in `test_an_empty_reserve`; B one identifier in
//! `test_a_single_identifier`; C n/a; D the boundary where `taken == len` in
//! `test_a_reserve_yields_each_identifier_once` (a fourth `next` after three); E/F/G/H/I/J/K n/a.

use deep_causality_context_store::IdReserve;

#[test]
fn test_a_reserve_yields_each_identifier_once() {
    let mut reserve = IdReserve::new(vec![10, 11, 12]);
    assert_eq!(reserve.remaining(), 3);
    assert_eq!(reserve.next(), Some(10));
    assert_eq!(reserve.remaining(), 2);
    assert_eq!(reserve.next(), Some(11));
    assert_eq!(reserve.remaining(), 1);
    assert_eq!(reserve.next(), Some(12));
    assert_eq!(reserve.remaining(), 0);
    assert_eq!(reserve.next(), None);
    assert_eq!(reserve.remaining(), 0);
}

#[test]
fn test_a_single_identifier() {
    let mut reserve = IdReserve::new(vec![7]);
    assert_eq!(reserve.remaining(), 1);
    assert_eq!(reserve.next(), Some(7));
    assert_eq!(reserve.remaining(), 0);
    assert_eq!(reserve.next(), None);
}

#[test]
fn test_an_empty_reserve() {
    let mut reserve = IdReserve::new(vec![]);
    assert_eq!(reserve.remaining(), 0);
    assert_eq!(reserve.next(), None);
    assert_eq!(reserve.remaining(), 0);
}

#[test]
fn test_clone_equality_and_debug() {
    let mut reserve = IdReserve::new(vec![1, 2]);
    let fresh = reserve.clone();
    assert_eq!(reserve, fresh);
    reserve.next();
    assert_ne!(reserve, fresh);
    assert!(format!("{reserve:?}").contains("IdReserve"));
}
