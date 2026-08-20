/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the cell-splitting trait.
//!
//! Reference for every formula asserted here: Chen, Y.-A. & Tata, S., *Higher
//! cup products on hypercubic lattices*, arXiv:2106.05274, J. Math. Phys. **64**,
//! 091902 (2023). Eq. (5) gives the simplicial (Alexander–Whitney) split; Fig. 1
//! gives the two-dimensional cubical split.
//!
//! Signs asserted here were computed by hand from the shuffle rule
//! `sgn(S_α ascending, then S_β ascending)` and independently confirmed to be the
//! unique rule satisfying the Leibniz identity against this crate's own
//! coboundary operators.

use deep_causality_topology::utils_tests::{axis_mask, open_layout, torus_layout};
use deep_causality_topology::{Cell, CellSplit, LatticeCell, Simplex, SplittableCell};

// ---------------------------------------------------------------------------
// Simplex: Alexander-Whitney, Chen & Tata Eq. (5)
// ---------------------------------------------------------------------------

#[test]
fn simplex_split_yields_exactly_one_term() {
    // AW is a single term, unlike the cubical sum.
    let s = Simplex::new(vec![0, 1, 2]);
    assert_eq!(s.split(1, None).len(), 1);
}

#[test]
fn simplex_split_left_is_the_leading_vertices() {
    // (a_p cup b_q)(0..p+q) = a_p(0 -> p) * b_q(p -> p+q), Eq. (5).
    let s = Simplex::new(vec![0, 1, 2]);
    let terms = s.split(1, None);
    assert_eq!(terms[0].left(), &Simplex::new(vec![0, 1]));
}

#[test]
fn simplex_split_right_is_the_trailing_vertices() {
    let s = Simplex::new(vec![0, 1, 2]);
    let terms = s.split(1, None);
    assert_eq!(terms[0].right(), &Simplex::new(vec![1, 2]));
}

#[test]
fn simplex_split_shares_the_middle_vertex() {
    // Left ends and right begins at vertex p; they overlap in exactly that one.
    let s = Simplex::new(vec![0, 1, 2, 3]);
    let terms = s.split(2, None);
    let l = terms[0].left().vertices().clone();
    let r = terms[0].right().vertices().clone();
    assert_eq!(l.last(), r.first());
    let shared: Vec<_> = l.iter().filter(|v| r.contains(v)).collect();
    assert_eq!(shared.len(), 1);
}

#[test]
fn simplex_split_sign_is_positive() {
    // Eq. (5) carries no sign.
    let s = Simplex::new(vec![0, 1, 2, 3]);
    for d in 0..=3 {
        for t in s.split(d, None) {
            assert_eq!(t.sign(), 1, "left_dim {d}");
        }
    }
}

#[test]
fn simplex_split_at_left_dim_zero() {
    // Left is the single leading vertex; right is the whole simplex.
    let s = Simplex::new(vec![2, 5, 9]);
    let terms = s.split(0, None);
    assert_eq!(terms.len(), 1);
    assert_eq!(terms[0].left(), &Simplex::new(vec![2]));
    assert_eq!(terms[0].right(), &Simplex::new(vec![2, 5, 9]));
}

#[test]
fn simplex_split_at_full_dimension() {
    // Left is the whole simplex; right is the single trailing vertex.
    let s = Simplex::new(vec![2, 5, 9]);
    let terms = s.split(2, None);
    assert_eq!(terms.len(), 1);
    assert_eq!(terms[0].left(), &Simplex::new(vec![2, 5, 9]));
    assert_eq!(terms[0].right(), &Simplex::new(vec![9]));
}

#[test]
fn simplex_split_beyond_its_dimension_is_empty() {
    // A degree the cell cannot carry contributes zero rather than failing.
    let s = Simplex::new(vec![0, 1, 2]);
    assert!(s.split(3, None).is_empty());
    assert!(s.split(99, None).is_empty());
}

#[test]
fn simplex_split_ignores_the_layout() {
    // A simplicial split is purely combinatorial.
    let s = Simplex::new(vec![0, 1, 2]);
    let layout = torus_layout(2, 4);
    assert_eq!(s.split(1, None), s.split(1, Some(&layout)));
}

#[test]
fn simplex_split_uses_sorted_vertex_order() {
    // Construction sorts, so the split does not depend on the argument order.
    let a = Simplex::new(vec![2, 0, 1]);
    let b = Simplex::new(vec![0, 1, 2]);
    assert_eq!(a.split(1, None), b.split(1, None));
}

// ---------------------------------------------------------------------------
// LatticeCell: cubical split, Chen & Tata Fig. 1
// ---------------------------------------------------------------------------

#[test]
fn lattice_split_term_count_is_binomial() {
    // C(k, left_dim) terms, one per choice of left directions.
    let layout = torus_layout(3, 4);
    let cell = LatticeCell::<3>::new([0, 0, 0], axis_mask(&[0, 1, 2]));
    assert_eq!(cell.split(0, Some(&layout)).len(), 1); // C(3,0)
    assert_eq!(cell.split(1, Some(&layout)).len(), 3); // C(3,1)
    assert_eq!(cell.split(2, Some(&layout)).len(), 3); // C(3,2)
    assert_eq!(cell.split(3, Some(&layout)).len(), 1); // C(3,3)
}

#[test]
fn lattice_2d_split_matches_the_published_example() {
    // Chen & Tata Fig. 1. For the unit square at the origin, the two terms are
    //   + (bottom x-edge, right y-edge)   and   - (left y-edge, top x-edge)
    // which reduce mod 2 to a(box_01)b(box_13) + a(box_02)b(box_23).
    let layout = torus_layout(2, 4);
    let face = LatticeCell::<2>::new([0, 0], axis_mask(&[0, 1]));
    let terms = face.split(1, Some(&layout));
    assert_eq!(terms.len(), 2);

    let bottom = LatticeCell::<2>::new([0, 0], axis_mask(&[0]));
    let right = LatticeCell::<2>::new([1, 0], axis_mask(&[1]));
    let left = LatticeCell::<2>::new([0, 0], axis_mask(&[1]));
    let top = LatticeCell::<2>::new([0, 1], axis_mask(&[0]));

    let find = |l: &LatticeCell<2>, r: &LatticeCell<2>| {
        terms.iter().find(|t| t.left() == l && t.right() == r)
    };
    assert_eq!(
        find(&bottom, &right).map(CellSplit::sign),
        Some(1),
        "bottom x-edge paired with right y-edge, sign +1"
    );
    assert_eq!(
        find(&left, &top).map(CellSplit::sign),
        Some(-1),
        "left y-edge paired with top x-edge, sign -1"
    );
}

#[test]
fn lattice_split_left_cell_sits_at_the_base_position() {
    // Convention: the left cell takes the leading directions from the base.
    let layout = torus_layout(3, 5);
    let cell = LatticeCell::<3>::new([1, 2, 3], axis_mask(&[0, 1, 2]));
    for t in cell.split(2, Some(&layout)) {
        assert_eq!(t.left().position(), &[1, 2, 3]);
    }
}

#[test]
fn lattice_split_right_cell_is_offset_by_the_left_directions() {
    // The right cell begins where the left one ends.
    let layout = torus_layout(3, 5);
    let cell = LatticeCell::<3>::new([1, 1, 1], axis_mask(&[0, 1, 2]));
    for t in cell.split(1, Some(&layout)) {
        let left_dirs: Vec<usize> = (0..3).filter(|i| t.left().orientation() & (1 << i) != 0).collect();
        let mut expected = [1usize, 1, 1];
        for j in left_dirs {
            expected[j] = (expected[j] + 1) % 5;
        }
        assert_eq!(t.right().position(), &expected);
    }
}

#[test]
fn lattice_split_directions_partition_the_active_axes() {
    let layout = torus_layout(3, 4);
    let cell = LatticeCell::<3>::new([0, 0, 0], axis_mask(&[0, 1, 2]));
    for t in cell.split(1, Some(&layout)) {
        let l = t.left().orientation();
        let r = t.right().orientation();
        assert_eq!(l & r, 0, "left and right directions are disjoint");
        assert_eq!(l | r, axis_mask(&[0, 1, 2]), "together they are the cell's axes");
    }
}

#[test]
fn lattice_3d_split_signs_are_the_shuffle_signs() {
    // Hand-computed from sgn(S_alpha ascending, then S_beta ascending):
    //   S_a={0} -> 0 inversions -> +1
    //   S_a={1} -> 1 inversion  -> -1
    //   S_a={2} -> 2 inversions -> +1
    let layout = torus_layout(3, 4);
    let cell = LatticeCell::<3>::new([0, 0, 0], axis_mask(&[0, 1, 2]));
    let expected = [(axis_mask(&[0]), 1i8), (axis_mask(&[1]), -1), (axis_mask(&[2]), 1)];
    for (dirs, sign) in expected {
        let t = cell
            .split(1, Some(&layout))
            .into_iter()
            .find(|t| t.left().orientation() == dirs)
            .expect("term for these left directions");
        assert_eq!(t.sign(), sign, "left directions {dirs:#b}");
    }
}

#[test]
fn lattice_3d_split_signs_at_left_dim_two() {
    // S_a={0,1} -> 0 inversions -> +1
    // S_a={0,2} -> 1 inversion  -> -1
    // S_a={1,2} -> 2 inversions -> +1
    let layout = torus_layout(3, 4);
    let cell = LatticeCell::<3>::new([0, 0, 0], axis_mask(&[0, 1, 2]));
    let expected = [(axis_mask(&[0, 1]), 1i8), (axis_mask(&[0, 2]), -1), (axis_mask(&[1, 2]), 1)];
    for (dirs, sign) in expected {
        let t = cell
            .split(2, Some(&layout))
            .into_iter()
            .find(|t| t.left().orientation() == dirs)
            .expect("term for these left directions");
        assert_eq!(t.sign(), sign, "left directions {dirs:#b}");
    }
}

#[test]
fn lattice_split_wraps_on_periodic_axes() {
    // A cell at the far edge of a torus pairs with one that has wrapped.
    let l = 4;
    let layout = torus_layout(2, l);
    let face = LatticeCell::<2>::new([l - 1, 0], axis_mask(&[0, 1]));
    let t = face
        .split(1, Some(&layout))
        .into_iter()
        .find(|t| t.left().orientation() == axis_mask(&[0]))
        .expect("term with left direction x");
    assert_eq!(t.right().position(), &[0, 0], "x wrapped from L-1 to 0");
}

#[test]
fn lattice_split_does_not_wrap_on_open_axes() {
    let l = 4;
    let layout = open_layout(2, l);
    let face = LatticeCell::<2>::new([l - 1, 0], axis_mask(&[0, 1]));
    let t = face
        .split(1, Some(&layout))
        .into_iter()
        .find(|t| t.left().orientation() == axis_mask(&[0]))
        .expect("term with left direction x");
    assert_eq!(t.right().position(), &[l, 0], "no wrap without periodicity");
}

#[test]
fn lattice_split_without_a_layout_does_not_wrap() {
    let face = LatticeCell::<2>::new([3, 0], axis_mask(&[0, 1]));
    let t = face
        .split(1, None)
        .into_iter()
        .find(|t| t.left().orientation() == axis_mask(&[0]))
        .expect("term with left direction x");
    assert_eq!(t.right().position(), &[4, 0]);
}

#[test]
fn lattice_split_at_left_dim_zero() {
    // Left is the base vertex; right is the whole cell.
    let layout = torus_layout(2, 4);
    let face = LatticeCell::<2>::new([1, 1], axis_mask(&[0, 1]));
    let terms = face.split(0, Some(&layout));
    assert_eq!(terms.len(), 1);
    assert_eq!(terms[0].left(), &LatticeCell::<2>::new([1, 1], 0));
    assert_eq!(terms[0].right(), &face);
    assert_eq!(terms[0].sign(), 1);
}

#[test]
fn lattice_split_at_full_dimension() {
    // Left is the whole cell; right is the opposite corner vertex.
    let layout = torus_layout(2, 4);
    let face = LatticeCell::<2>::new([1, 1], axis_mask(&[0, 1]));
    let terms = face.split(2, Some(&layout));
    assert_eq!(terms.len(), 1);
    assert_eq!(terms[0].left(), &face);
    assert_eq!(terms[0].right(), &LatticeCell::<2>::new([2, 2], 0));
    assert_eq!(terms[0].sign(), 1);
}

#[test]
fn lattice_split_beyond_its_dimension_is_empty() {
    let layout = torus_layout(2, 4);
    let edge = LatticeCell::<2>::new([0, 0], axis_mask(&[0]));
    assert!(edge.split(2, Some(&layout)).is_empty());
    let vertex = LatticeCell::<2>::new([0, 0], 0);
    assert!(vertex.split(1, Some(&layout)).is_empty());
}

#[test]
fn lattice_vertex_splits_only_at_dim_zero() {
    let layout = torus_layout(2, 4);
    let vertex = LatticeCell::<2>::new([2, 3], 0);
    let terms = vertex.split(0, Some(&layout));
    assert_eq!(terms.len(), 1);
    assert_eq!(terms[0].left(), &vertex);
    assert_eq!(terms[0].right(), &vertex);
    assert_eq!(terms[0].sign(), 1);
}

// ---------------------------------------------------------------------------
// CellSplit accessors and the Cell trait's stability
// ---------------------------------------------------------------------------

#[test]
fn cell_split_exposes_its_parts() {
    let l = Simplex::new(vec![0, 1]);
    let r = Simplex::new(vec![1, 2]);
    let t = CellSplit::new(l.clone(), r.clone(), -1);
    assert_eq!(t.left(), &l);
    assert_eq!(t.right(), &r);
    assert_eq!(t.sign(), -1);
}

#[test]
fn cell_split_into_parts_round_trips() {
    let l = Simplex::new(vec![0, 1]);
    let r = Simplex::new(vec![1, 2]);
    let (a, b, s) = CellSplit::new(l.clone(), r.clone(), 1).into_parts();
    assert_eq!((a, b, s), (l, r, 1));
}

#[test]
fn cell_trait_is_unchanged_for_every_implementor() {
    // The splitting trait is additive: Cell's own surface still resolves.
    let s = Simplex::new(vec![0, 1, 2]);
    assert_eq!(Cell::dim(&s), 2);
    assert!(!Cell::boundary(&s).is_empty());

    let c = LatticeCell::<2>::new([0, 0], axis_mask(&[0, 1]));
    assert_eq!(Cell::dim(&c), 2);
    assert!(!Cell::boundary(&c).is_empty());
}

#[test]
fn splitting_is_opt_in_and_does_not_bound_cell_users() {
    // A function bounded on Cell alone still compiles and runs, so nothing that
    // does not need cup products has to know the splitting trait exists.
    fn dim_of<C: Cell>(c: &C) -> usize {
        c.dim()
    }
    assert_eq!(dim_of(&Simplex::new(vec![0, 1])), 1);
    assert_eq!(dim_of(&LatticeCell::<2>::new([0, 0], 1)), 1);
}
