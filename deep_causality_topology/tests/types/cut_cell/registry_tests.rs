/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the cut-cell registry accessors: the cell-merging floor's
//! no-op arm, the staircase no-slip edge set on an open lattice, and the dual
//! clip on a degenerate lattice.

use deep_causality_topology::{
    CellularComplex, CubicalReggeGeometry, CutCell, CutCellRegistry, LatticeCell, LatticeComplex,
};

const TOL: f64 = 1e-12;

/// Index of the top cell whose base is `base` on a 2-D lattice.
fn top_index(lattice: &LatticeComplex<2, f64>, base: [usize; 2]) -> usize {
    let top = LatticeCell::<2>::new(base, 0b11);
    lattice.cells(2).position(|c| c == top).unwrap()
}

#[test]
fn clipped_cell_volume_keeps_a_free_volume_above_the_floor() {
    // Cell merging inflates only volumes below `min_fraction · full`. A cut
    // cell holding 0.8 of a unit cell is already above a 0.2 floor, so the
    // clipped volume is its own 0.8.
    let lattice = LatticeComplex::<2, f64>::square_torus(4);
    let geom = CubicalReggeGeometry::<2, f64>::uniform(1.0);
    let base = [1usize, 1usize];
    let top = LatticeCell::<2>::new(base, 0b11);
    let idx = top_index(&lattice, base);

    let mut reg = CutCellRegistry::<2, f64>::new();
    reg.insert(
        idx,
        CutCell::<2, f64>::cut(1.0, 0.8, [[1.0, 1.0], [1.0, 1.0]], Vec::new()),
    );
    let stab = reg.with_cell_merging(0.2);

    let v = stab.clipped_cell_volume(&geom, &lattice, &top);
    assert!((v - 0.8).abs() < TOL, "expected the unfloored 0.8, got {v}");
}

#[test]
fn clipped_cell_volume_leaves_a_dry_cell_at_zero() {
    // A fully solid cell has zero free volume. The floor applies to free
    // volumes only, so an interior-solid cell stays dry and is dropped from the
    // dynamics rather than inflated.
    let lattice = LatticeComplex::<2, f64>::square_torus(4);
    let geom = CubicalReggeGeometry::<2, f64>::uniform(1.0);
    let base = [2usize, 1usize];
    let top = LatticeCell::<2>::new(base, 0b11);
    let idx = top_index(&lattice, base);

    let mut reg = CutCellRegistry::<2, f64>::new();
    reg.insert(idx, CutCell::<2, f64>::solid(1.0));
    let stab = reg.with_cell_merging(0.25);

    let v = stab.clipped_cell_volume(&geom, &lattice, &top);
    assert_eq!(v, 0.0, "a dry cell keeps its zero volume");
}

#[test]
fn solid_incident_edges_on_an_open_lattice_pins_the_solid_cube_ring() {
    // Open 3x3 lattice: top cells have bases (0,0), (1,0), (0,1), (1,1). Mark
    // the cube at base (0,0) solid; it spans [0,1] x [0,1], so exactly its four
    // bounding edges are pinned:
    //   axis-0 edges at (0,0) and (0,1)  -> flat indices 0 and 2,
    //   axis-1 edges at (0,0) and (1,0)  -> flat indices 6 and 7.
    // The `iter_cells(1)` order is axis-major with axis 0 varying fastest:
    // six axis-0 edges (2 x 3 positions) precede six axis-1 edges (3 x 2).
    let lattice = LatticeComplex::<2, f64>::square_open(3);
    let idx = top_index(&lattice, [0, 0]);

    let mut reg = CutCellRegistry::<2, f64>::new();
    reg.insert(idx, CutCell::<2, f64>::solid(1.0));

    let pinned = reg.solid_incident_edges(&lattice);
    assert_eq!(pinned, vec![0, 2, 6, 7]);
}

#[test]
fn solid_incident_edges_is_empty_without_solid_cells() {
    // A registry holding only cut cells pins nothing: the staircase set covers
    // solid cells, whose interiors carry no flow.
    let lattice = LatticeComplex::<2, f64>::square_open(3);
    let idx = top_index(&lattice, [0, 0]);

    let mut reg = CutCellRegistry::<2, f64>::new();
    reg.insert(
        idx,
        CutCell::<2, f64>::cut(1.0, 0.5, [[0.5, 0.5], [0.5, 0.5]], Vec::new()),
    );

    assert!(reg.solid_incident_edges(&lattice).is_empty());
}

#[test]
fn dual_fluid_fraction_is_zero_on_a_lattice_with_a_zero_extent() {
    // A lattice with a zero extent along one axis holds no cells, so an edge's
    // dual has no in-bounds incident cube. Every corner is outside the domain
    // and contributes 0, and the fraction is the empty average: 0.
    let lattice = LatticeComplex::<2, f64>::open([3, 0]);
    let reg = CutCellRegistry::<2, f64>::new();
    let edge = LatticeCell::<2>::new([0, 0], 0b01);

    assert_eq!(reg.dual_fluid_fraction(&lattice, &edge), 0.0);
}

#[test]
fn dual_fluid_fraction_of_an_interior_edge_on_an_empty_registry_is_one() {
    // On a torus with no registered cells every incident cube is fluid, so the
    // dual clip is the identity.
    let lattice = LatticeComplex::<2, f64>::square_torus(4);
    let reg = CutCellRegistry::<2, f64>::new();
    let edge = LatticeCell::<2>::new([1, 1], 0b01);

    assert_eq!(reg.dual_fluid_fraction(&lattice, &edge), 1.0);
}
