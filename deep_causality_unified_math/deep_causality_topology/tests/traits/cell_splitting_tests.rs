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
//! `sgn(S_α ascending, then S_β ascending)`, confirmed against an independent
//! reference implementation, and independently shown to be the unique rule
//! satisfying the Leibniz identity against this crate's own coboundary operators.

use deep_causality_topology::utils_tests::{axis_mask, layout_of, open_layout, torus_layout};
use deep_causality_topology::{Cell, CellSplit, LatticeCell, Simplex, SplittableCell};

// ---------------------------------------------------------------------------
// Simplex: Alexander-Whitney, Chen & Tata Eq. (5)
// ---------------------------------------------------------------------------

#[test]
fn simplex_split_yields_exactly_one_term() {
    let s = Simplex::new(vec![0, 1, 2]);
    assert_eq!(s.split(1, None).len(), 1);
}

#[test]
fn simplex_split_left_is_the_leading_vertices() {
    let s = Simplex::new(vec![0, 1, 2]);
    assert_eq!(s.split(1, None)[0].left(), &Simplex::new(vec![0, 1]));
}

#[test]
fn simplex_split_right_is_the_trailing_vertices() {
    let s = Simplex::new(vec![0, 1, 2]);
    assert_eq!(s.split(1, None)[0].right(), &Simplex::new(vec![1, 2]));
}

#[test]
fn simplex_split_shares_the_middle_vertex() {
    let s = Simplex::new(vec![0, 1, 2, 3]);
    let terms = s.split(2, None);
    let l = terms[0].left().vertices().clone();
    let r = terms[0].right().vertices().clone();
    assert_eq!(l.last(), r.first());
    assert_eq!(l.iter().filter(|v| r.contains(v)).count(), 1);
}

#[test]
fn simplex_split_sign_is_positive() {
    let s = Simplex::new(vec![0, 1, 2, 3]);
    for d in 0..=3 {
        for t in s.split(d, None) {
            assert_eq!(t.sign(), 1, "left_dim {d}");
        }
    }
}

#[test]
fn simplex_split_at_left_dim_zero() {
    let s = Simplex::new(vec![2, 5, 9]);
    let terms = s.split(0, None);
    assert_eq!(terms.len(), 1);
    assert_eq!(terms[0].left(), &Simplex::new(vec![2]));
    assert_eq!(terms[0].right(), &Simplex::new(vec![2, 5, 9]));
}

#[test]
fn simplex_split_at_full_dimension() {
    let s = Simplex::new(vec![2, 5, 9]);
    let terms = s.split(2, None);
    assert_eq!(terms.len(), 1);
    assert_eq!(terms[0].left(), &Simplex::new(vec![2, 5, 9]));
    assert_eq!(terms[0].right(), &Simplex::new(vec![9]));
}

#[test]
fn simplex_split_beyond_its_dimension_is_empty() {
    let s = Simplex::new(vec![0, 1, 2]);
    assert!(s.split(3, None).is_empty());
    assert!(s.split(99, None).is_empty());
}

#[test]
fn split_at_the_maximum_left_dim_is_empty_not_a_panic() {
    // Guards against `left_dim + 1` overflowing: in release that wraps to zero
    // and the guard would let an out-of-bounds slice through.
    assert!(
        Simplex::new(vec![0, 1, 2])
            .split(usize::MAX, None)
            .is_empty()
    );
    assert!(
        LatticeCell::<2>::new([0, 0], axis_mask(&[0, 1]))
            .split(usize::MAX, None)
            .is_empty()
    );
}

#[test]
fn simplex_split_ignores_the_layout() {
    let s = Simplex::new(vec![0, 1, 2]);
    let layout = torus_layout(2, 4);
    assert_eq!(s.split(1, None), s.split(1, Some(&layout)));
}

#[test]
fn simplex_split_uses_sorted_vertex_order() {
    let a = Simplex::new(vec![2, 0, 1]);
    let b = Simplex::new(vec![0, 1, 2]);
    assert_eq!(a.split(1, None), b.split(1, None));
}

#[test]
fn zero_simplex_splits_into_itself() {
    let v = Simplex::new(vec![7]);
    let terms = v.split(0, None);
    assert_eq!(terms.len(), 1);
    assert_eq!(terms[0].left(), &v);
    assert_eq!(terms[0].right(), &v);
    assert_eq!(terms[0].sign(), 1);
    assert!(v.split(1, None).is_empty());
}

#[test]
fn simplex_construction_sorts_its_vertices() {
    // The guarantee the cup product rests on is the ordering. Distinct inputs
    // therefore come back strictly increasing.
    for raw in [vec![2, 0, 1], vec![9], vec![3, 1, 2, 0]] {
        let s = Simplex::new(raw.clone());
        let v = s.vertices();
        assert!(
            v.windows(2).all(|w| w[0] < w[1]),
            "distinct input {raw:?} should be strictly increasing, got {v:?}"
        );
    }
}

#[test]
fn simplex_construction_sorts_but_does_not_deduplicate() {
    // The documented invariant is non-decreasing order, not uniqueness. A
    // repeated index survives, which is the caller's to avoid.
    let s = Simplex::new(vec![5, 5, 1]);
    assert_eq!(s.vertices(), &vec![1, 5, 5], "sorted, duplicate retained");
    assert!(s.vertices().windows(2).all(|w| w[0] <= w[1]));
}

#[test]
fn a_degenerate_simplex_splits_without_panicking() {
    // A split of a repeated-vertex simplex yields a face no complex contains,
    // so the cup product's lookup misses and the term contributes zero. What
    // matters here is that the split itself stays total.
    let s = Simplex::new(vec![5, 5, 1]);
    let terms = s.split(1, None);
    assert_eq!(terms.len(), 1);
    assert_eq!(terms[0].left(), &Simplex::new(vec![1, 5]));
    assert_eq!(terms[0].right(), &Simplex::new(vec![5, 5]));
}

// ---------------------------------------------------------------------------
// LatticeCell: cubical split, Chen & Tata Fig. 1
// ---------------------------------------------------------------------------

#[test]
fn lattice_split_term_count_is_binomial() {
    let layout = torus_layout(3, 4);
    let cell = LatticeCell::<3>::new([0, 0, 0], axis_mask(&[0, 1, 2]));
    for (left_dim, expected) in [(0, 1), (1, 3), (2, 3), (3, 1)] {
        assert_eq!(
            cell.split(left_dim, Some(&layout)).len(),
            expected,
            "C(3,{left_dim})"
        );
    }
}

#[test]
fn lattice_2d_split_matches_the_published_example() {
    // Chen & Tata Fig. 1: + (bottom x-edge, right y-edge) - (left y-edge, top x-edge).
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
    assert_eq!(find(&bottom, &right).map(CellSplit::sign), Some(1));
    assert_eq!(find(&left, &top).map(CellSplit::sign), Some(-1));
}

#[test]
fn lattice_split_left_cell_sits_at_the_base_position() {
    let layout = torus_layout(3, 5);
    let cell = LatticeCell::<3>::new([1, 2, 3], axis_mask(&[0, 1, 2]));
    for t in cell.split(2, Some(&layout)) {
        assert_eq!(t.left().position(), &[1, 2, 3]);
    }
}

#[test]
fn lattice_split_right_cell_is_offset_by_the_left_directions() {
    let layout = torus_layout(3, 5);
    let cell = LatticeCell::<3>::new([1, 1, 1], axis_mask(&[0, 1, 2]));
    for t in cell.split(1, Some(&layout)) {
        let dirs: Vec<usize> = (0..3)
            .filter(|i| t.left().orientation() & (1 << i) != 0)
            .collect();
        let mut expected = [1usize, 1, 1];
        for j in dirs {
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
        let (l, r) = (t.left().orientation(), t.right().orientation());
        assert_eq!(l & r, 0, "disjoint");
        assert_eq!(l | r, axis_mask(&[0, 1, 2]), "together the cell's axes");
    }
}

#[test]
fn lattice_3d_split_signs_are_the_shuffle_signs() {
    // Inversions: {0}->0 (+1), {1}->1 (-1), {2}->2 (+1).
    let layout = torus_layout(3, 4);
    let cell = LatticeCell::<3>::new([0, 0, 0], axis_mask(&[0, 1, 2]));
    for (dirs, sign) in [(vec![0], 1i8), (vec![1], -1), (vec![2], 1)] {
        let t = cell
            .split(1, Some(&layout))
            .into_iter()
            .find(|t| t.left().orientation() == axis_mask(&dirs))
            .unwrap_or_else(|| panic!("term for {dirs:?}"));
        assert_eq!(t.sign(), sign, "left directions {dirs:?}");
    }
}

#[test]
fn lattice_3d_split_signs_at_left_dim_two() {
    // Inversions: {0,1}->0 (+1), {0,2}->1 (-1), {1,2}->2 (+1).
    let layout = torus_layout(3, 4);
    let cell = LatticeCell::<3>::new([0, 0, 0], axis_mask(&[0, 1, 2]));
    for (dirs, sign) in [(vec![0, 1], 1i8), (vec![0, 2], -1), (vec![1, 2], 1)] {
        let t = cell
            .split(2, Some(&layout))
            .into_iter()
            .find(|t| t.left().orientation() == axis_mask(&dirs))
            .unwrap_or_else(|| panic!("term for {dirs:?}"));
        assert_eq!(t.sign(), sign, "left directions {dirs:?}");
    }
}

#[test]
fn lattice_split_wraps_on_periodic_axes() {
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
    assert_eq!(t.right().position(), &[l, 0]);
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
fn lattice_split_wraps_per_axis_on_a_non_square_torus() {
    // A wrapping bug reusing one axis extent for every axis passes every
    // square-lattice test. Extents 3 and 5 separate the two.
    let layout = layout_of(&[3, 5], &[true, true]);
    let face = LatticeCell::<2>::new([2, 4], axis_mask(&[0, 1]));
    let pick = |dirs: &[usize]| {
        face.split(1, Some(&layout))
            .into_iter()
            .find(|t| t.left().orientation() == axis_mask(dirs))
            .expect("term")
    };
    assert_eq!(pick(&[0]).right().position(), &[0, 4], "x wraps modulo 3");
    assert_eq!(pick(&[1]).right().position(), &[2, 0], "y wraps modulo 5");
}

#[test]
fn lattice_split_respects_per_axis_periodicity() {
    // A cylinder: x periodic, y open. The face is one the complex carries:
    // with y open, faces exist only at y in {0, 1}.
    let layout = layout_of(&[3, 3], &[true, false]);
    let face = LatticeCell::<2>::new([2, 1], axis_mask(&[0, 1]));
    let pick = |dirs: &[usize]| {
        face.split(1, Some(&layout))
            .into_iter()
            .find(|t| t.left().orientation() == axis_mask(dirs))
            .expect("term")
    };
    assert_eq!(pick(&[0]).right().position(), &[0, 1], "x wraps modulo 3");
    assert_eq!(pick(&[1]).right().position(), &[2, 2], "y is open");
}

#[test]
fn lattice_4d_split_signs_are_the_shuffle_signs() {
    let layout = layout_of(&[2, 2, 2, 2], &[true; 4]);
    let cell = LatticeCell::<4>::new([0, 0, 0, 0], axis_mask(&[0, 1, 2, 3]));
    let expected = [
        (vec![0, 1], 1i8),
        (vec![0, 2], -1),
        (vec![0, 3], 1),
        (vec![1, 2], 1),
        (vec![1, 3], -1),
        (vec![2, 3], 1),
    ];
    for (axes, sign) in expected {
        let t = cell
            .split(2, Some(&layout))
            .into_iter()
            .find(|t| t.left().orientation() == axis_mask(&axes))
            .unwrap_or_else(|| panic!("term for {axes:?}"));
        assert_eq!(t.sign(), sign, "left directions {axes:?}");
    }
}

#[test]
fn lattice_4d_split_term_count_is_binomial() {
    let layout = layout_of(&[2, 2, 2, 2], &[true; 4]);
    let cell = LatticeCell::<4>::new([0, 0, 0, 0], axis_mask(&[0, 1, 2, 3]));
    for (left_dim, expected) in [(0, 1), (1, 4), (2, 6), (3, 4), (4, 1)] {
        assert_eq!(
            cell.split(left_dim, Some(&layout)).len(),
            expected,
            "C(4,{left_dim})"
        );
    }
}

#[test]
fn lattice_1d_split_is_the_degenerate_case() {
    let layout = layout_of(&[4], &[true]);
    let edge = LatticeCell::<1>::new([3], axis_mask(&[0]));
    let terms = edge.split(0, Some(&layout));
    assert_eq!(terms.len(), 1);
    assert_eq!(terms[0].left(), &LatticeCell::<1>::new([3], 0));
    assert_eq!(
        terms[0].right(),
        &LatticeCell::<1>::new([3], axis_mask(&[0]))
    );
}

#[test]
fn lattice_split_at_left_dim_zero() {
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
    assert!(
        LatticeCell::<2>::new([0, 0], axis_mask(&[0]))
            .split(2, Some(&layout))
            .is_empty()
    );
    assert!(
        LatticeCell::<2>::new([0, 0], 0)
            .split(1, Some(&layout))
            .is_empty()
    );
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

#[test]
fn lattice_cell_corner_order_is_the_documented_one() {
    let cell = LatticeCell::<3>::new([1, 2, 3], axis_mask(&[0, 2]));
    let corners = cell.vertices();
    assert_eq!(corners.len(), 4, "2^k corners for a k-cell");
    assert_eq!(corners[0], [1, 2, 3], "corner 0 is the base position");
    assert_eq!(corners[1], [2, 2, 3], "bit 0 -> first active axis (0)");
    assert_eq!(corners[2], [1, 2, 4], "bit 1 -> second active axis (2)");
    assert_eq!(corners[3], [2, 2, 4], "both bits");
}

// ---------------------------------------------------------------------------
// CellSplit accessors and the Cell trait's stability
// ---------------------------------------------------------------------------

#[test]
fn cell_split_exposes_its_parts() {
    let (l, r) = (Simplex::new(vec![0, 1]), Simplex::new(vec![1, 2]));
    let t = CellSplit::negative(l.clone(), r.clone());
    assert_eq!(t.left(), &l);
    assert_eq!(t.right(), &r);
    assert_eq!(t.sign(), -1);
}

#[test]
fn cell_split_into_parts_round_trips() {
    let (l, r) = (Simplex::new(vec![0, 1]), Simplex::new(vec![1, 2]));
    let (a, b, s) = CellSplit::positive(l.clone(), r.clone()).into_parts();
    assert_eq!((a, b, s), (l, r, 1));
}

#[test]
fn cell_split_sign_is_always_a_unit() {
    // There is no constructor taking an arbitrary i8, so a stray 0 cannot
    // annihilate a term and no other magnitude can be read as a unit.
    let (l, r) = (Simplex::new(vec![0, 1]), Simplex::new(vec![1, 2]));
    assert_eq!(CellSplit::positive(l.clone(), r.clone()).sign(), 1);
    assert_eq!(CellSplit::negative(l.clone(), r.clone()).sign(), -1);
    for inv in 0..6usize {
        let sign = CellSplit::from_parity(l.clone(), r.clone(), inv).sign();
        assert_eq!(sign, if inv % 2 == 0 { 1 } else { -1 });
    }
}

#[test]
fn every_split_a_shipped_cell_family_produces_carries_a_unit_sign() {
    let layout = torus_layout(3, 4);
    let cell = LatticeCell::<3>::new([0, 0, 0], axis_mask(&[0, 1, 2]));
    for d in 0..=3 {
        for t in cell.split(d, Some(&layout)) {
            assert!(t.sign() == 1 || t.sign() == -1, "left_dim {d}");
        }
    }
    for t in Simplex::new(vec![0, 1, 2, 3]).split(2, None) {
        assert_eq!(t.sign(), 1);
    }
}

#[test]
fn cell_trait_is_unchanged_for_every_implementor() {
    let s = Simplex::new(vec![0, 1, 2]);
    assert_eq!(Cell::dim(&s), 2);
    assert!(!Cell::boundary(&s).is_empty());
    let c = LatticeCell::<2>::new([0, 0], axis_mask(&[0, 1]));
    assert_eq!(Cell::dim(&c), 2);
    assert!(!Cell::boundary(&c).is_empty());
}

#[test]
fn cell_remains_usable_as_a_standalone_bound() {
    // Code generic over `Cell` alone still compiles and sees the same surface.
    fn dim_and_boundary_len<C: Cell>(c: &C) -> (usize, usize) {
        (c.dim(), c.boundary().len())
    }
    assert_eq!(dim_and_boundary_len(&Simplex::new(vec![0, 1])), (1, 2));
    let (d, b) = dim_and_boundary_len(&LatticeCell::<2>::new([0, 0], axis_mask(&[0, 1])));
    assert_eq!(d, 2);
    assert_eq!(b, 4, "a square has four boundary edges");
}

#[test]
fn splitting_is_opt_in_for_a_type_that_only_implements_cell() {
    // A cell type that implements `Cell` and not `SplittableCell` still compiles
    // and is usable everywhere `Cell` is required. This is what keeping the
    // splitting trait separate buys, and it is why adding a required method to
    // `Cell` was rejected.
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    struct PlainCell(usize);

    impl Cell for PlainCell {
        fn dim(&self) -> usize {
            self.0
        }
        fn boundary(&self) -> Vec<(Self, i8)> {
            if self.0 == 0 {
                Vec::new()
            } else {
                vec![(PlainCell(self.0 - 1), 1)]
            }
        }
    }

    fn uses_cell_only<C: Cell>(c: &C) -> usize {
        c.dim() + c.boundary().len()
    }
    assert_eq!(uses_cell_only(&PlainCell(2)), 3);
    assert_eq!(uses_cell_only(&PlainCell(0)), 0);
}
