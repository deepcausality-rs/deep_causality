/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Shared fixtures and helpers for the cell-splitting and cup-product tests.
//!
//! These live in `src` rather than under `tests/` because each Bazel test suite
//! compiles its sources independently and cannot reach a helper module in the
//! test tree, while the whole `src` tree is available during testing.

pub use crate::types::lattice_complex::cell_splitting::axis_mask;

use crate::traits::cell_splitting::CellLayout;
use crate::traits::cellular_complex::CellularComplex;
use crate::traits::chain_complex::ChainComplex;
use crate::types::lattice_complex::{LatticeCell, LatticeComplex};
use crate::types::simplex::Simplex;
use crate::types::simplicial_complex::{SimplicialComplex, SimplicialComplexBuilder};
use std::collections::HashMap;

/// A fully periodic (toroidal) layout of side `l` in `d` dimensions.
pub fn torus_layout(d: usize, l: usize) -> CellLayout {
    (vec![l; d], vec![true; d])
}

/// A layout with per-axis extent and periodicity, for the non-uniform cases.
pub fn layout_of(shape: &[usize], periodic: &[bool]) -> CellLayout {
    (shape.to_vec(), periodic.to_vec())
}

/// A fully open (non-periodic) layout, so no position wraps.
pub fn open_layout(d: usize, l: usize) -> CellLayout {
    (vec![l; d], vec![false; d])
}

/// A single tetrahedron: a hand-built, non-lattice complex, so tests exercise
/// the generic path rather than only the cubical implementor.
pub fn tetrahedron() -> SimplicialComplex<f64> {
    let mut b = SimplicialComplexBuilder::new(3);
    b.add_simplex(Simplex::new(vec![0, 1, 2, 3]))
        .expect("a tetrahedron is a valid simplex");
    b.build().expect("the tetrahedron builds")
}

/// The coboundary `δ` of a `k`-cochain, taken from the complex's own operator
/// rather than recomputed, so a law checked with it is checked against what the
/// crate actually ships.
pub fn delta<K: ChainComplex>(complex: &K, k: usize, cochain: &[f64]) -> Vec<f64> {
    let m = complex.coboundary_matrix(k);
    let (rows, cols) = m.shape();
    (0..rows)
        .map(|r| {
            (0..cols)
                .map(|j| {
                    let v = m.get_value_at(r, j);
                    if v == 0 { 0.0 } else { v as f64 * cochain[j] }
                })
                .sum()
        })
        .collect()
}

/// A deterministic pseudo-random cochain of length `n`, so a failing law test is
/// reproducible from its seed.
pub fn pseudo_cochain(n: usize, seed: u64) -> Vec<f64> {
    let mut s = seed;
    (0..n)
        .map(|_| {
            s = s
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (((s >> 33) % 1000) as f64) / 500.0 - 1.0
        })
        .collect()
}

/// The largest absolute difference between two equal-length cochains.
///
/// # Panics
/// Panics when the lengths differ, since comparing cochains of different degree
/// is a test bug rather than a numerical result.
pub fn max_abs_diff(a: &[f64], b: &[f64]) -> f64 {
    assert_eq!(a.len(), b.len(), "cochain lengths differ");
    a.iter()
        .zip(b)
        .map(|(x, y)| (x - y).abs())
        .fold(0.0, f64::max)
}

/// Maps each `k`-simplex to its index in the complex's own cell order.
pub fn simplex_index(complex: &SimplicialComplex<f64>, k: usize) -> HashMap<Simplex, usize> {
    complex.cells(k).enumerate().map(|(i, s)| (s, i)).collect()
}

/// The 1-cochain equal to 1 on every edge in direction `dir` and 0 elsewhere.
///
/// On a torus this is a cocycle representing a generator of `H¹` scaled by the
/// side length.
pub fn direction_cochain<const D: usize>(complex: &LatticeComplex<D, f64>, dir: usize) -> Vec<f64> {
    complex
        .cells(1)
        .map(|cell| {
            if cell.orientation() == (1 << dir) {
                1.0
            } else {
                0.0
            }
        })
        .collect()
}

/// Maps each `k`-cell to its index in the complex's own cell order.
pub fn lattice_index<const D: usize>(
    complex: &LatticeComplex<D, f64>,
    k: usize,
) -> HashMap<LatticeCell<D>, usize> {
    complex.cells(k).enumerate().map(|(i, c)| (c, i)).collect()
}
