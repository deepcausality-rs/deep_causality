/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `VonNeumann` — grid face-adjacency on `LatticeComplex<D>`.

use deep_causality_topology::{LatticeComplex, Neighborhood, VonNeumann};

#[test]
fn test_von_neumann_open_2d_corner_has_two_neighbors() {
    // 4x4 open lattice -> top-cube grid is 3x3, dim_max = [3, 3].
    // cell_id formula: id = pos[0] + pos[1] * 3.
    let c = LatticeComplex::<2, f64>::new([4, 4], [false, false]);
    // Corner cell at pos (0, 0) -> id 0. Neighbors: (1,0)=1, (0,1)=3.
    let mut n: Vec<_> = VonNeumann.neighbors(&c, 0).collect();
    n.sort_unstable();
    assert_eq!(n, vec![1, 3]);
}

#[test]
fn test_von_neumann_open_2d_interior_has_four_neighbors() {
    let c = LatticeComplex::<2, f64>::new([4, 4], [false, false]);
    // Center cell pos (1,1) -> id 4. Neighbors: 1,3,5,7.
    let mut n: Vec<_> = VonNeumann.neighbors(&c, 4).collect();
    n.sort_unstable();
    assert_eq!(n, vec![1, 3, 5, 7]);
}

#[test]
fn test_von_neumann_periodic_2d_full_2d_neighbors() {
    // Periodic 3x3 torus -> dim_max = [3, 3], 9 top cells. Every cell has 2*D = 4 neighbors.
    let c = LatticeComplex::<2, f64>::new([3, 3], [true, true]);
    let n: Vec<_> = VonNeumann.neighbors(&c, 0).collect();
    assert_eq!(
        n.len(),
        4,
        "torus von-Neumann yields exactly 2D = 4 neighbors"
    );
}

#[test]
fn test_von_neumann_invalid_cell_id_yields_empty() {
    let c = LatticeComplex::<2, f64>::new([4, 4], [false, false]);
    let n: Vec<_> = VonNeumann.neighbors(&c, 9999).collect();
    assert!(n.is_empty());
}

#[test]
fn test_von_neumann_zero_shape_axis_yields_empty() {
    // An open axis with shape 0 forces `top_axis_range` into its `shape == 0` branch
    // (returning 0), which in turn makes `cell_id_to_top_pos` hit its `dim_max == 0`
    // early-return (`None`). Any cell_id therefore resolves to no position and the
    // neighbor iterator is empty. Covers src/types/neighborhood/mod.rs lines 51 and 68.
    let c = LatticeComplex::<2, f64>::new([0, 4], [false, false]);
    let n: Vec<_> = VonNeumann.neighbors(&c, 0).collect();
    assert!(n.is_empty());

    // Also exercise the case where the zero axis is not the first axis: the loop in
    // `cell_id_to_top_pos` advances past axis 0 (valid) and rejects at axis 1.
    let c2 = LatticeComplex::<2, f64>::new([4, 0], [false, false]);
    let n2: Vec<_> = VonNeumann.neighbors(&c2, 0).collect();
    assert!(n2.is_empty());
}
