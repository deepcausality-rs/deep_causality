/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `CubicalReggeGeometry<D>` — the cubical Regge geometry primitive.

use deep_causality_metric::Metric;
use deep_causality_topology::CubicalReggeGeometry;

// -- Constructors / classification -------------------------------------------------

#[test]
fn unit_constructor_marks_unit_edge() {
    let g = CubicalReggeGeometry::<3, f64>::unit();
    assert!(g.is_unit_edge());
    assert!(g.is_isotropic());
    assert!(g.is_axis_aligned());
    assert!(!g.is_lorentzian());
}

#[test]
fn uniform_constructor_is_isotropic_but_not_unit_edge() {
    let g = CubicalReggeGeometry::<3, f64>::uniform(0.5);
    assert!(!g.is_unit_edge());
    assert!(g.is_isotropic());
    assert!(g.is_axis_aligned());
}

#[test]
fn per_axis_constructor_is_axis_aligned_but_not_isotropic() {
    let g = CubicalReggeGeometry::<3, f64>::per_axis([1.0, 2.0, 3.0]);
    assert!(!g.is_unit_edge());
    assert!(!g.is_isotropic());
    assert!(g.is_axis_aligned());
}

#[test]
fn per_edge_constructor_is_general() {
    let g = CubicalReggeGeometry::<2, f64>::from_edge_lengths(vec![1.0, 1.5, 2.0, 2.5]);
    assert!(!g.is_unit_edge());
    assert!(!g.is_isotropic());
    assert!(!g.is_axis_aligned());
}

// -- Uniform-length getter -------------------------------------------------------

#[test]
fn uniform_length_returns_one_for_unit_edge() {
    let g = CubicalReggeGeometry::<3, f64>::unit();
    assert_eq!(g.uniform_length(), Some(1.0));
}

#[test]
fn uniform_length_returns_value_for_uniform() {
    let g = CubicalReggeGeometry::<3, f64>::uniform(0.25);
    assert_eq!(g.uniform_length(), Some(0.25));
}

#[test]
fn uniform_length_is_none_for_per_axis() {
    let g = CubicalReggeGeometry::<3, f64>::per_axis([1.0, 2.0, 3.0]);
    assert!(g.uniform_length().is_none());
}

#[test]
fn uniform_length_is_none_for_per_edge() {
    let g = CubicalReggeGeometry::<2, f64>::from_edge_lengths(vec![1.0, 2.0, 3.0]);
    assert!(g.uniform_length().is_none());
}

// -- axis_lengths getter ---------------------------------------------------------

#[test]
fn axis_lengths_returns_ones_for_unit_edge() {
    let g = CubicalReggeGeometry::<3, f64>::unit();
    assert_eq!(g.axis_lengths(), Some([1.0, 1.0, 1.0]));
}

#[test]
fn axis_lengths_broadcasts_uniform() {
    let g = CubicalReggeGeometry::<3, f64>::uniform(2.5);
    assert_eq!(g.axis_lengths(), Some([2.5, 2.5, 2.5]));
}

#[test]
fn axis_lengths_passes_through_per_axis() {
    let g = CubicalReggeGeometry::<3, f64>::per_axis([0.5, 1.0, 2.0]);
    assert_eq!(g.axis_lengths(), Some([0.5, 1.0, 2.0]));
}

#[test]
fn axis_lengths_is_none_for_per_edge() {
    let g = CubicalReggeGeometry::<2, f64>::from_edge_lengths(vec![1.0, 2.0]);
    assert!(g.axis_lengths().is_none());
}

// -- axis_length single-axis getter ----------------------------------------------

#[test]
fn axis_length_per_axis_indexes_correctly() {
    let g = CubicalReggeGeometry::<3, f64>::per_axis([0.5, 1.0, 2.0]);
    assert_eq!(g.axis_length(0), Some(0.5));
    assert_eq!(g.axis_length(1), Some(1.0));
    assert_eq!(g.axis_length(2), Some(2.0));
}

#[test]
fn axis_length_out_of_range_is_none() {
    let g = CubicalReggeGeometry::<3, f64>::unit();
    assert!(g.axis_length(3).is_none());
    assert!(g.axis_length(99).is_none());
}

#[test]
fn axis_length_is_none_for_per_edge() {
    let g = CubicalReggeGeometry::<2, f64>::from_edge_lengths(vec![1.0, 2.0]);
    assert!(g.axis_length(0).is_none());
}

#[test]
fn axis_length_unit_edge_returns_one_in_range() {
    let g = CubicalReggeGeometry::<3, f64>::unit();
    assert_eq!(g.axis_length(0), Some(1.0));
    assert_eq!(g.axis_length(1), Some(1.0));
    assert_eq!(g.axis_length(2), Some(1.0));
}

#[test]
fn axis_length_uniform_returns_length_in_range() {
    let g = CubicalReggeGeometry::<3, f64>::uniform(0.75);
    assert_eq!(g.axis_length(0), Some(0.75));
    assert_eq!(g.axis_length(2), Some(0.75));
}

// -- edge_length_at single-edge getter -------------------------------------------

#[test]
fn edge_length_at_returns_one_for_unit_edge() {
    let g = CubicalReggeGeometry::<3, f64>::unit();
    assert_eq!(g.edge_length_at(0), Some(1.0));
    assert_eq!(g.edge_length_at(42), Some(1.0));
}

#[test]
fn edge_length_at_returns_value_for_uniform() {
    let g = CubicalReggeGeometry::<3, f64>::uniform(0.75);
    assert_eq!(g.edge_length_at(0), Some(0.75));
    assert_eq!(g.edge_length_at(100), Some(0.75));
}

#[test]
fn edge_length_at_is_none_for_per_axis() {
    // Per-axis representation cannot answer edge_id alone — axis is required.
    let g = CubicalReggeGeometry::<3, f64>::per_axis([1.0, 2.0, 3.0]);
    assert!(g.edge_length_at(0).is_none());
}

#[test]
fn edge_length_at_indexes_per_edge() {
    let g = CubicalReggeGeometry::<2, f64>::from_edge_lengths(vec![1.0, 1.5, 2.0]);
    assert_eq!(g.edge_length_at(0), Some(1.0));
    assert_eq!(g.edge_length_at(1), Some(1.5));
    assert_eq!(g.edge_length_at(2), Some(2.0));
    assert!(g.edge_length_at(3).is_none());
}

// -- edge_lengths flat slice ---------------------------------------------------

#[test]
fn edge_lengths_slice_only_for_per_edge() {
    assert!(
        CubicalReggeGeometry::<3, f64>::unit()
            .edge_lengths()
            .is_none()
    );
    assert!(
        CubicalReggeGeometry::<3, f64>::uniform(1.0)
            .edge_lengths()
            .is_none()
    );
    assert!(
        CubicalReggeGeometry::<3, f64>::per_axis([1.0, 1.0, 1.0])
            .edge_lengths()
            .is_none()
    );

    let g = CubicalReggeGeometry::<2, f64>::from_edge_lengths(vec![1.0, 2.0, 3.0]);
    assert_eq!(g.edge_lengths(), Some([1.0, 2.0, 3.0].as_slice()));
}

// -- Lorentzian / timelike axes -----------------------------------------------

#[test]
fn timelike_axes_default_is_none() {
    let g = CubicalReggeGeometry::<3, f64>::unit();
    assert!(g.timelike_axes().is_none());
    assert!(!g.is_lorentzian());
}

#[test]
fn with_timelike_axes_attaches_pattern() {
    let g = CubicalReggeGeometry::<4, f64>::unit().with_timelike_axes([false, false, false, true]);
    assert_eq!(g.timelike_axes(), Some(&[false, false, false, true]));
    assert!(g.is_lorentzian());
}

#[test]
fn all_spacelike_axes_is_not_lorentzian() {
    let g = CubicalReggeGeometry::<3, f64>::unit().with_timelike_axes([false, false, false]);
    assert!(!g.is_lorentzian());
}

// -- Signature -----------------------------------------------------------------

#[test]
fn signature_euclidean_for_unflagged() {
    let g = CubicalReggeGeometry::<3, f64>::unit();
    let m = g.signature();
    // Euclidean (3, 0, 0) — all three axes spacelike.
    match m {
        Metric::Euclidean(d) => assert_eq!(d, 3),
        other => panic!("expected Euclidean(3), got {other:?}"),
    }
}

#[test]
fn signature_lorentzian_for_one_timelike_axis() {
    let g = CubicalReggeGeometry::<4, f64>::unit().with_timelike_axes([false, false, false, true]);
    let m = g.signature();
    // (3, 1, 0) — three spacelike + one timelike → Lorentzian 4D.
    match m {
        Metric::Lorentzian(d) => assert_eq!(d, 4),
        other => panic!("expected Lorentzian(4), got {other:?}"),
    }
}

// -- Equality / Debug / Clone --------------------------------------------------

#[test]
fn equality_distinguishes_representations() {
    let unit_a = CubicalReggeGeometry::<2, f64>::unit();
    let unit_b = CubicalReggeGeometry::<2, f64>::unit();
    assert_eq!(unit_a, unit_b);

    let uniform_one = CubicalReggeGeometry::<2, f64>::uniform(1.0);
    // Unit and Uniform{1.0} differ at the variant level even though they encode the same data.
    // This is intentional: the variant carries intent for downstream optimization.
    assert_ne!(unit_a, uniform_one);
}

#[test]
fn clone_preserves_state() {
    let g = CubicalReggeGeometry::<3, f64>::per_axis([0.5, 1.0, 2.0])
        .with_timelike_axes([true, false, false]);
    let c = g.clone();
    assert_eq!(g, c);
    assert_eq!(c.axis_lengths(), Some([0.5, 1.0, 2.0]));
    assert!(c.is_lorentzian());
}
