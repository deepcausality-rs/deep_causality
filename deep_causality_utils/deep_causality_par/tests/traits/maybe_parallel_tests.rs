/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_par::MaybeParallel;

/// The bound is satisfiable generically (compile-time property).
fn requires_maybe_parallel<T: MaybeParallel>(value: T) -> T {
    value
}

#[test]
fn test_scalars_satisfy_the_bound() {
    assert_eq!(requires_maybe_parallel(1.5f64), 1.5f64);
    assert_eq!(requires_maybe_parallel(2.5f32), 2.5f32);
    assert_eq!(requires_maybe_parallel(7usize), 7usize);
}

#[test]
fn test_compound_types_satisfy_the_bound() {
    let v = vec![1.0f64, 2.0];
    assert_eq!(requires_maybe_parallel(v.clone()), v);
}

#[test]
fn test_unsized_types_satisfy_the_bound() {
    // The blanket impl is `?Sized`; str and slices qualify.
    fn touch<T: MaybeParallel + ?Sized>(_value: &T) {}
    touch("str");
    touch(&[1.0f64, 2.0][..]);
}

#[cfg(not(feature = "parallel"))]
#[test]
fn test_the_serial_bound_is_vacuous() {
    // The module's central claim: without `parallel` the bound imposes nothing, so a serial
    // consumer never sees `Send + Sync`. `Rc` is neither, so this compiles only while the two
    // cfg arms stay distinct — collapsing them onto the `Send + Sync` definition breaks it.
    // Every other type in this file already satisfies both, so nothing else observes the
    // difference between the arms.
    use std::rc::Rc;
    assert_eq!(*requires_maybe_parallel(Rc::new(1.5f64)), 1.5f64);

    // The same holds through the bound's one consumer.
    let out = deep_causality_par::scoped_map(&[Rc::new(1.0f64), Rc::new(2.0)], |x| **x);
    assert_eq!(out, vec![1.0, 2.0]);
}
