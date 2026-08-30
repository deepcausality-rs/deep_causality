/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `KRing<const K>` — Chebyshev-distance-≤-K neighborhood on `LatticeComplex<D>`.

use deep_causality_topology::{KRing, LatticeComplex, Moore, Neighborhood};

#[test]
fn test_k_ring_zero_yields_empty() {
    let c = LatticeComplex::<2, f64>::new([4, 4], [false, false]);
    let n: Vec<_> = KRing::<0>.neighbors(&c, 4).collect();
    assert!(n.is_empty(), "KRing<0> excludes the target itself");
}

#[test]
fn test_k_ring_one_matches_moore() {
    // KRing<1> is equivalent to Moore on the same lattice.
    let c = LatticeComplex::<2, f64>::new([4, 4], [false, false]);
    let mut a: Vec<_> = KRing::<1>.neighbors(&c, 4).collect();
    let mut b: Vec<_> = Moore.neighbors(&c, 4).collect();
    a.sort_unstable();
    b.sort_unstable();
    assert_eq!(a, b);
}

#[test]
fn test_k_ring_two_torus_full_count() {
    // On a 5x5 torus the top grid is 5x5 = 25. KRing<2> yields (2*2 + 1)^2 - 1 = 24.
    let c = LatticeComplex::<2, f64>::new([5, 5], [true, true]);
    let n: Vec<_> = KRing::<2>.neighbors(&c, 0).collect();
    assert_eq!(n.len(), 24);
}

#[test]
fn test_k_ring_two_open_corner_clipped() {
    // 6x6 open lattice -> 5x5 top grid. From corner (0,0) with K=2, candidates
    // come from a (2K+1) x (2K+1) = 5x5 stencil clipped to the upper-right
    // quadrant: positions (0..=2) x (0..=2) minus the origin = 9 - 1 = 8.
    let c = LatticeComplex::<2, f64>::new([6, 6], [false, false]);
    let n: Vec<_> = KRing::<2>.neighbors(&c, 0).collect();
    assert_eq!(n.len(), 8);
}

#[test]
fn test_k_ring_invalid_cell_id_is_empty() {
    let c = LatticeComplex::<2, f64>::new([4, 4], [false, false]);
    let n: Vec<_> = KRing::<2>.neighbors(&c, 9999).collect();
    assert!(n.is_empty());
}
