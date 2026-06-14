/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Cut-aware Hodge-star tests (Group B foundation / B6 equivalence):
//!
//! - an **empty** registry's `cut_hodge_star_matrix` is byte-equal to the shipped
//!   `hodge_star_matrix` at every grade, on open and periodic lattices, across the unit,
//!   uniform, per-axis and graded tiers — the cut clip reduces to the Stage-3 wall clip;
//! - a registry with a solid cell genuinely shrinks the affected dual diagonal entries
//!   (the cut star is not a no-op when cells are removed).

use deep_causality_topology::{
    ChainComplex, CubicalReggeGeometry, CutCell, CutCellRegistry, HasHodgeStar, LatticeCell,
    LatticeComplex,
};

const TOL: f64 = 1e-12;

fn assert_star_equiv<const D: usize>(
    lattice: &LatticeComplex<D, f64>,
    geom: &CubicalReggeGeometry<D, f64>,
) {
    let empty = CutCellRegistry::<D, f64>::new();
    for k in 0..=D {
        let std = geom.hodge_star_matrix(lattice, k).unwrap();
        let cut = geom.cut_hodge_star_matrix(lattice, &empty, k).unwrap();
        let sv = std.values();
        let cv = cut.values();
        assert_eq!(sv.len(), cv.len(), "k={k}: nnz differ");
        for (i, (a, b)) in sv.iter().zip(cv.iter()).enumerate() {
            assert!(
                (a - b).abs() < TOL,
                "k={k} entry {i}: std {a} != cut {b} (empty registry must reduce to the wall clip)"
            );
        }
    }
}

#[test]
fn empty_registry_cut_star_equals_standard_star_2d() {
    let open = LatticeComplex::<2, f64>::square_open(4);
    let torus = LatticeComplex::<2, f64>::square_torus(4);
    for lattice in [&open, &torus] {
        assert_star_equiv(lattice, &CubicalReggeGeometry::<2, f64>::unit());
        assert_star_equiv(lattice, &CubicalReggeGeometry::<2, f64>::uniform(0.7));
        assert_star_equiv(
            lattice,
            &CubicalReggeGeometry::<2, f64>::per_axis([0.5, 1.3]),
        );
    }
}

#[test]
fn empty_registry_cut_star_equals_standard_star_3d() {
    let open = LatticeComplex::<3, f64>::open([3, 3, 3]);
    let torus = LatticeComplex::<3, f64>::cubic_torus(3);
    for lattice in [&open, &torus] {
        assert_star_equiv(lattice, &CubicalReggeGeometry::<3, f64>::unit());
        assert_star_equiv(lattice, &CubicalReggeGeometry::<3, f64>::uniform(1.5));
    }
}

#[test]
fn empty_registry_cut_star_equals_standard_star_graded() {
    // The graded PerEdge tier must also reduce exactly (cut rides the graded substrate).
    let lattice = LatticeComplex::<2, f64>::square_open(5);
    let geom =
        CubicalReggeGeometry::<2, f64>::from_graded_geometric(&lattice, [1.0, 1.0], [1.2, 1.0]);
    assert_star_equiv(&lattice, &geom);
}

#[test]
fn solid_cell_shrinks_the_affected_dual_entries() {
    // Mark one interior top cell solid; the vertices/edges on its boundary lose dual corners,
    // so their cut-star diagonal entries drop below the (interior, unclipped) standard value.
    let lattice = LatticeComplex::<2, f64>::square_torus(4);
    let geom = CubicalReggeGeometry::<2, f64>::unit();

    let mut reg = CutCellRegistry::<2, f64>::new();
    // Top cell at base [1,1] (interior of the periodic lattice).
    let solid_cell = LatticeCell::<2>::new([1, 1], 0b11);
    let idx = lattice.cells(2).position(|c| c == solid_cell).unwrap();
    reg.insert(idx, CutCell::<2, f64>::solid(1.0));

    let std = geom.hodge_star_matrix(&lattice, 0).unwrap();
    let cut = geom.cut_hodge_star_matrix(&lattice, &reg, 0).unwrap();

    // The vertex at [1,1] is a corner of the solid cell: one of its 4 dual quadrants is dry,
    // so its star entry is 3/4 of the unclipped value.
    let v = LatticeCell::<2>::vertex([1, 1]);
    let vi = lattice.cells(0).position(|c| c == v).unwrap();
    assert!(
        (std.values()[vi] - 1.0).abs() < TOL,
        "periodic vertex is unclipped in std star"
    );
    assert!(
        (cut.values()[vi] - 0.75).abs() < TOL,
        "vertex adjacent to a solid cell should lose 1/4 of its dual, got {}",
        cut.values()[vi]
    );

    // A vertex far from the solid cell is unchanged.
    let far = LatticeCell::<2>::vertex([3, 3]);
    let fi = lattice.cells(0).position(|c| c == far).unwrap();
    assert!(
        (cut.values()[fi] - 1.0).abs() < TOL,
        "distant vertex must be unaffected"
    );
}
