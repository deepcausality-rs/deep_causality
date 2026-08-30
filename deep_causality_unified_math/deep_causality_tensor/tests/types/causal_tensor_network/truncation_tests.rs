/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algebra::{ConjugateScalar, RealField};
use deep_causality_num::{Float106, FromPrimitive};
use deep_causality_tensor::{CausalTensorError, Truncation};

fn v<T: FromPrimitive>(x: f64) -> T {
    T::from_f64(x).unwrap()
}

fn check_truncation<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>() {
    let svals = [v::<T>(4.0), v::<T>(3.0), v::<T>(2.0), v::<T>(1.0)];

    // Pure bond cap keeps exactly the leading `max_bond` values.
    let t = Truncation::by_bond(2).unwrap();
    assert_eq!(t.retained_rank(&svals), 2);
    assert_eq!(t.max_bond(), 2);

    // A bond cap above the count is clamped to the count.
    let t = Truncation::by_bond(10).unwrap();
    assert_eq!(t.retained_rank(&svals), 4);

    // Relative tolerance keeps while σ_i ≥ rel_tol · σ_0 (0.5 · 4 = 2.0).
    let t = Truncation::by_tol(v::<T>(0.5)).unwrap();
    assert_eq!(t.retained_rank(&svals), 3);

    // Absolute floor gate.
    let t = Truncation::new(10, v::<T>(0.0), v::<T>(2.5)).unwrap();
    assert_eq!(t.retained_rank(&svals), 2);

    // Bond cap AND tolerance: the tighter of the two wins.
    let t = Truncation::new(2, v::<T>(0.0), v::<T>(2.5)).unwrap();
    assert_eq!(t.retained_rank(&svals), 2);

    // Getters round-trip.
    let t = Truncation::new(5, v::<T>(0.1), v::<T>(0.01)).unwrap();
    assert_eq!(t.max_bond(), 5);
    assert!(t.rel_tol() == v::<T>(0.1));
    assert!(t.abs_tol() == v::<T>(0.01));

    // At least one value is always retained.
    let zeros = [v::<T>(0.0), v::<T>(0.0)];
    let t = Truncation::new(4, v::<T>(0.1), v::<T>(0.1)).unwrap();
    assert_eq!(t.retained_rank(&zeros), 1);

    // Empty input retains nothing.
    let empty: [T; 0] = [];
    assert_eq!(t.retained_rank(&empty), 0);
}

#[test]
fn test_truncation_f32() {
    check_truncation::<f32>();
}

#[test]
fn test_truncation_f64() {
    check_truncation::<f64>();
}

#[test]
fn test_truncation_float106() {
    check_truncation::<Float106>();
}

#[test]
fn test_truncation_invalid_params() {
    // Zero bond cap is rejected.
    assert!(matches!(
        Truncation::<f64>::by_bond(0),
        Err(CausalTensorError::InvalidParameter(_))
    ));
    assert!(matches!(
        Truncation::<f64>::new(0, 0.0, 0.0),
        Err(CausalTensorError::InvalidParameter(_))
    ));
    // Negative tolerances are rejected.
    assert!(matches!(
        Truncation::<f64>::new(1, -1.0, 0.0),
        Err(CausalTensorError::InvalidParameter(_))
    ));
    assert!(matches!(
        Truncation::<f64>::new(1, 0.0, -1.0),
        Err(CausalTensorError::InvalidParameter(_))
    ));
    assert!(matches!(
        Truncation::<f64>::by_tol(-0.5),
        Err(CausalTensorError::InvalidParameter(_))
    ));
}
