/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `Moore` — Chebyshev-distance-1 neighborhood on `LatticeComplex<D>`.

use deep_causality_topology::{LatticeComplex, Moore, Neighborhood};

#[test]
fn test_moore_open_2d_corner_three_neighbors() {
    // 4x4 open lattice -> 3x3 top-cube grid. Corner (0,0): Moore neighbors are
    // (1,0), (0,1), (1,1) -> ids 1, 3, 4.
    let c = LatticeComplex::<2, f64>::new([4, 4], [false, false]);
    let mut n: Vec<_> = Moore.neighbors(&c, 0).collect();
    n.sort_unstable();
    assert_eq!(n, vec![1, 3, 4]);
}

#[test]
fn test_moore_open_2d_interior_eight_neighbors() {
    // Center cell on 3x3 top grid has 3^2 - 1 = 8 Moore neighbors.
    let c = LatticeComplex::<2, f64>::new([4, 4], [false, false]);
    let n: Vec<_> = Moore.neighbors(&c, 4).collect();
    assert_eq!(n.len(), 8);
}

#[test]
fn test_moore_periodic_2d_always_eight() {
    // On a torus every cell has the full 3^2 - 1 = 8 Moore neighbors.
    let c = LatticeComplex::<2, f64>::new([3, 3], [true, true]);
    let n: Vec<_> = Moore.neighbors(&c, 0).collect();
    assert_eq!(n.len(), 8);
}

#[test]
fn test_moore_invalid_cell_id_is_empty() {
    let c = LatticeComplex::<2, f64>::new([4, 4], [false, false]);
    let n: Vec<_> = Moore.neighbors(&c, 9999).collect();
    assert!(n.is_empty());
}
