/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the `Neighborhood<K>` strategy trait and its concrete impls.
//!
//! Covers:
//!   * Zero-sized assertion on all strategy types.
//!   * Moore yielding 8 / 26 neighbors on 2D / 3D open grids (interior cell).
//!   * `KRing::<2>` yielding 24 neighbors on a 2D open grid (interior cell).
//!   * VonNeumann wrap-around on a torus.
//!   * VonNeumann boundary trimming on an open grid.
//!   * `FaceAdjacent` coincides with `VonNeumann` on top cubes.

use deep_causality_topology::{
    ChainComplex, FaceAdjacent, KRing, LatticeComplex, Moore, Neighborhood, VonNeumann,
};
use std::collections::HashSet;
use std::mem::size_of;

#[test]
fn neighborhood_strategies_are_zero_sized() {
    assert_eq!(size_of::<VonNeumann>(), 0);
    assert_eq!(size_of::<Moore>(), 0);
    assert_eq!(size_of::<KRing<1>>(), 0);
    assert_eq!(size_of::<KRing<2>>(), 0);
    assert_eq!(size_of::<KRing<5>>(), 0);
    assert_eq!(size_of::<FaceAdjacent>(), 0);
}

fn collect<I: Iterator<Item = usize>>(it: I) -> Vec<usize> {
    let mut v: Vec<usize> = it.collect();
    v.sort_unstable();
    v.dedup();
    v
}

#[test]
fn moore_3d_interior_yields_26_neighbors() {
    // 4×4×4 open grid: top cubes occupy 3×3×3 = 27 positions; the center is at [1,1,1].
    let cubes = LatticeComplex::<3>::open([4, 4, 4]);
    let center_pos = [1usize, 1, 1];
    // cell_id of center: position[0] + position[1]*3 + position[2]*9 = 1 + 3 + 9 = 13.
    let center_id = 1 + 3 + 9;
    let neighbors = collect(Moore.neighbors(&cubes, center_id));
    assert_eq!(neighbors.len(), 26, "Moore-3D open interior: {neighbors:?}");
    assert!(!neighbors.contains(&center_id), "self should not appear");
    let _ = center_pos; // documentation only
}

#[test]
fn k_ring_2_2d_interior_yields_24_neighbors() {
    // 6×6 open grid: top cubes occupy 5×5 = 25 positions; an interior cell at [2,2].
    let cubes = LatticeComplex::<2>::open([6, 6]);
    let center_id = 2 + 2 * 5usize; // = 12
    let neighbors = collect(KRing::<2>.neighbors(&cubes, center_id));
    assert_eq!(
        neighbors.len(),
        24,
        "KRing<2>-2D open interior: {neighbors:?}"
    );
    assert!(!neighbors.contains(&center_id));
}

#[test]
fn von_neumann_2d_torus_wraps() {
    // 4×4 torus: corner cell at [0,0] (cell_id = 0) wraps to all 4 face-adjacent neighbors.
    let cubes = LatticeComplex::<2>::torus([4, 4]);
    let neighbors = collect(VonNeumann.neighbors(&cubes, 0));
    assert_eq!(neighbors.len(), 4, "torus corner cell: {neighbors:?}");
    // Expected wrap-around: ±1 in each axis. shape[0] = shape[1] = 4.
    // East: pos[0]=1 → id 1. West: pos[0]=3 → id 3. North: pos[1]=1 → id 4. South: pos[1]=3 → id 12.
    let expected: HashSet<usize> = HashSet::from([1, 3, 4, 12]);
    let got: HashSet<usize> = neighbors.into_iter().collect();
    assert_eq!(got, expected, "torus wrap neighbors");
}

#[test]
fn von_neumann_2d_open_corner_yields_two_neighbors() {
    // 4×4 open grid: top cubes occupy 3×3; corner at [0,0] (cell_id = 0) has 2 in-bounds neighbors.
    let cubes = LatticeComplex::<2>::open([4, 4]);
    let neighbors = collect(VonNeumann.neighbors(&cubes, 0));
    assert_eq!(
        neighbors.len(),
        2,
        "open corner has exactly 2 face-adjacent neighbors: {neighbors:?}"
    );
    let expected: HashSet<usize> = HashSet::from([1, 3]); // +x = 1, +y = 3 (stride = 3).
    let got: HashSet<usize> = neighbors.into_iter().collect();
    assert_eq!(got, expected);
}

#[test]
fn face_adjacent_coincides_with_von_neumann_on_top_cubes() {
    // On a regular cubical grid, FaceAdjacent (via ∂) and VonNeumann (via grid coords)
    // must yield the SAME set of neighbors for every interior top cell.
    let cubes = LatticeComplex::<2>::open([5, 5]);
    let n_top = ChainComplex::num_cells(&cubes, 2);
    for cell_id in 0..n_top {
        let face_adj = collect(FaceAdjacent.neighbors(&cubes, cell_id));
        let von_neu = collect(VonNeumann.neighbors(&cubes, cell_id));
        assert_eq!(
            face_adj, von_neu,
            "FaceAdjacent vs VonNeumann at top cell {cell_id}"
        );
    }
}

#[test]
fn moore_open_corner_yields_three_neighbors_in_2d() {
    // 4×4 open grid: top cubes 3×3, corner at [0,0]. Moore = {[1,0], [0,1], [1,1]}.
    let cubes = LatticeComplex::<2>::open([4, 4]);
    let neighbors = collect(Moore.neighbors(&cubes, 0));
    assert_eq!(neighbors.len(), 3, "Moore open corner 2D: {neighbors:?}");
}
