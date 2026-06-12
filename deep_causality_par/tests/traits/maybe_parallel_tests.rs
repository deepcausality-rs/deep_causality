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
