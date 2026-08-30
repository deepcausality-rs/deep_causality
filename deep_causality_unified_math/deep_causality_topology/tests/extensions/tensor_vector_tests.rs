/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `TensorVector` constructors and conversions.
//!
//! These lived in `hkt_curvature_tests.rs`, where four of seven tests exercised this type rather
//! than the curvature operator and made the witness look better covered than it was.

use deep_causality_topology::TensorVector;

#[test]
fn new_copies_the_slice_and_reports_its_dimension() {
    let v = TensorVector::<f64>::new(&[1.0, 2.0, 3.0]);
    assert_eq!(v.dim(), 3);
    assert_eq!(v.as_slice(), &[1.0, 2.0, 3.0]);

    let empty = TensorVector::<f64>::new(&[]);
    assert_eq!(empty.dim(), 0);
}

#[test]
fn zeros_fills_with_zero_at_every_width() {
    for dim in [0usize, 1, 2, 7] {
        let z = TensorVector::<f64>::zeros(dim);
        assert_eq!(z.dim(), dim);
        assert!(z.as_slice().iter().all(|&x| x == 0.0));
    }
}

#[test]
fn basis_sets_exactly_one_component() {
    for dim in [1usize, 3, 5] {
        for i in 0..dim {
            let b = TensorVector::<f64>::basis(dim, i);
            assert_eq!(b.dim(), dim);
            for (j, &x) in b.as_slice().iter().enumerate() {
                let want = if j == i { 1.0 } else { 0.0 };
                assert_eq!(x, want, "basis({dim}, {i}) component {j}");
            }
        }
    }
}

#[test]
fn basis_out_of_range_yields_the_zero_vector() {
    // `basis` guards the index rather than panicking; that behaviour is asserted rather than
    // assumed, because the guard is easy to drop in a refactor.
    let b = TensorVector::<f64>::basis(3, 7);
    assert_eq!(b.dim(), 3);
    assert!(b.as_slice().iter().all(|&x| x == 0.0));
}

#[test]
fn vec_conversion_round_trips_in_both_directions() {
    for raw in [vec![], vec![4.0f64], vec![1.0, 2.0, 3.0, -4.5]] {
        let tv: TensorVector<f64> = raw.clone().into();
        assert_eq!(tv.dim(), raw.len());
        let back: Vec<f64> = tv.into();
        assert_eq!(back, raw);
    }
}
