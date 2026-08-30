/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! An abstract simplicial complex, used as the crate's [`ChainComplex`] fixture.
//!
//! # Why this lives in `src` rather than `tests`
//!
//! Bazel cannot reach a helper in the `tests` tree from a test target, but it can reach all of
//! `src`. The consequence is that this file counts toward coverage like any other, which is why it
//! carries its own assertions rather than trusting the callers.
//!
//! # Why the crate ships a fixture at all
//!
//! The claim this crate makes is that homology needs no geometry. A fixture that reached for
//! `deep_causality_topology` would refute it. This one is a list of vertex tuples and the
//! alternating-sign formula — no cells, no metric, no coordinates.
//!
//! # Where the spaces come from
//!
//! `openspec/notes/archive/homology/reference/reference.py` builds the same ten spaces in Python and checks
//! its own Betti numbers against Hatcher, *Algebraic Topology*. This file is an independent
//! construction of the same spaces; the tests check it against the same published values. Two
//! implementations agreeing with a source is not the same as one implementation agreeing with
//! itself.

use crate::traits::chain_complex::ChainComplex;
use alloc::borrow::Cow;
use alloc::collections::BTreeSet;
use alloc::vec;
use alloc::vec::Vec;
use deep_causality_linear::CsrMatrix;

/// A finite abstract simplicial complex, closed under taking faces.
///
/// `simplices[k]` holds the `k`-simplices in ascending order; a simplex's position there is its
/// column index in `∂ₖ`.
#[derive(Debug, Clone)]
pub struct SimplicialFixture {
    name: &'static str,
    simplices: Vec<Vec<Vec<usize>>>,
}

impl SimplicialFixture {
    /// Builds from maximal simplices, generating every face.
    ///
    /// Faces are generated rather than listed, so a facet list that omits a face cannot produce a
    /// structure that is not a complex.
    ///
    /// # Panics
    ///
    /// If a facet repeats a vertex, which would make it not a simplex.
    pub fn new(name: &'static str, facets: &[&[usize]]) -> Self {
        let mut faces: BTreeSet<Vec<usize>> = BTreeSet::new();
        for facet in facets {
            let mut s: Vec<usize> = facet.to_vec();
            s.sort_unstable();
            s.dedup();
            assert_eq!(
                s.len(),
                facet.len(),
                "{name}: facet {facet:?} repeats a vertex"
            );
            // Every non-empty subset, by the bits of a counter over the facet's vertices.
            for mask in 1u32..(1u32 << s.len()) {
                let face: Vec<usize> = (0..s.len())
                    .filter(|i| mask & (1 << i) != 0)
                    .map(|i| s[i])
                    .collect();
                faces.insert(face);
            }
        }
        let dim = faces
            .iter()
            .map(|s| s.len())
            .max()
            .unwrap_or(0)
            .saturating_sub(1);
        let simplices = (0..=dim)
            .map(|k| faces.iter().filter(|s| s.len() == k + 1).cloned().collect())
            .collect();
        Self { name, simplices }
    }

    /// The fixture's name, for assertion messages.
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// The position of a simplex among the `k`-simplices.
    fn position(&self, k: usize, face: &[usize]) -> usize {
        self.simplices[k]
            .binary_search_by(|s| s.as_slice().cmp(face))
            .unwrap_or_else(|_| panic!("{}: face {face:?} is missing from grade {k}", self.name))
    }

    /// The Euler characteristic from cell counts, `Σ(−1)ᵏ nₖ`.
    ///
    /// Computed from the cell counts alone, so it never passes through the rank routine. That is
    /// what makes comparing it with the alternating sum of Betti numbers two computations agreeing
    /// rather than one rearranged.
    pub fn euler_from_cells(&self) -> i64 {
        (0..=self.max_dim())
            .map(|k| {
                let n = self.num_cells(k) as i64;
                if k % 2 == 0 { n } else { -n }
            })
            .sum()
    }
}

impl ChainComplex for SimplicialFixture {
    fn num_cells(&self, k: usize) -> usize {
        self.simplices.get(k).map_or(0, |v| v.len())
    }

    fn max_dim(&self) -> usize {
        self.simplices.len().saturating_sub(1)
    }

    fn boundary_matrix(&self, k: usize) -> Cow<'_, CsrMatrix<i8>> {
        let (rows, cols) = (self.num_cells(k.wrapping_sub(1)), self.num_cells(k));
        // Grade 0 has no (−1)-cells, and the grade past the top has no cells: both keep the shape
        // their dimension implies so the composite is always formable.
        let rows = if k == 0 { 0 } else { rows };
        let mut triplets: Vec<(usize, usize, i8)> = Vec::new();
        if k > 0 && k <= self.max_dim() {
            for (col, s) in self.simplices[k].iter().enumerate() {
                for i in 0..s.len() {
                    // ∂[s] = Σ (−1)ⁱ (s with vertex i dropped).
                    let mut face = s.clone();
                    face.remove(i);
                    let sign: i8 = if i % 2 == 0 { 1 } else { -1 };
                    triplets.push((self.position(k - 1, &face), col, sign));
                }
            }
        }
        Cow::Owned(
            CsrMatrix::from_triplets(rows, cols, &triplets)
                .unwrap_or_else(|e| panic!("{}: ∂_{k} is not well formed: {e}", self.name)),
        )
    }

    fn coboundary_matrix(&self, k: usize) -> Cow<'_, CsrMatrix<i8>> {
        Cow::Owned(self.boundary_matrix(k + 1).transpose())
    }
}

/// Every space the reference oracle covers, as `(fixture, β over ℚ, β over 𝔽₂)`.
///
/// The Betti numbers are the values published in Hatcher, *Algebraic Topology* — Example 2.13 for
/// `S¹`, Corollary 2.14 for `S²`, Example 2.36 for `T²`, Example 2.42 for `ℝP²`, Example 2.47 for
/// the Klein bottle. They are not readings of this code.
pub fn reference_spaces() -> Vec<(SimplicialFixture, Vec<usize>, Vec<usize>)> {
    let mut spaces = vec![
        (SimplicialFixture::new("point", &[&[0]]), vec![1], vec![1]),
        (
            SimplicialFixture::new("interval", &[&[0, 1]]),
            vec![1, 0],
            vec![1, 0],
        ),
        (
            SimplicialFixture::new("circle", &[&[0, 1], &[1, 2], &[0, 2]]),
            vec![1, 1],
            vec![1, 1],
        ),
        (
            SimplicialFixture::new(
                "sphere_2",
                &[&[0, 1, 2], &[0, 1, 3], &[0, 2, 3], &[1, 2, 3]],
            ),
            vec![1, 0, 1],
            vec![1, 0, 1],
        ),
        (torus_2(), vec![1, 2, 1], vec![1, 2, 1]),
        (cylinder(), vec![1, 1, 0], vec![1, 1, 0]),
        (mobius_band(), vec![1, 1, 0], vec![1, 1, 0]),
        // ℝP² and the Klein bottle carry 2-torsion, so ℚ and 𝔽₂ give different answers. Every other
        // fixture in this list, and every complex the workspace shipped before this crate existed,
        // is torsion-free — a suite without these two cannot tell the coefficient fields apart.
        (
            SimplicialFixture::new(
                "real_projective_plane",
                &[
                    &[1, 2, 3],
                    &[1, 3, 4],
                    &[1, 4, 5],
                    &[1, 5, 6],
                    &[1, 2, 6],
                    &[2, 3, 5],
                    &[3, 4, 6],
                    &[2, 4, 5],
                    &[3, 5, 6],
                    &[2, 4, 6],
                ],
            ),
            vec![1, 0, 0],
            vec![1, 1, 1],
        ),
        (klein_bottle(), vec![1, 1, 0], vec![1, 2, 1]),
    ];
    spaces.extend(torus_3_space());
    spaces
}

/// The oracle's tenth space, `T³`.
///
/// At 702 simplices against the next-largest 96 it dominates this crate's test time natively and
/// is unbounded under Miri, where building it alone took 50 seconds against 2 milliseconds native.
/// So `make miri` skips it, the way `deep_causality_num` and `deep_causality_physics` skip their
/// slow suites. Every other space in the set is carried under both.
#[cfg(not(miri))]
fn torus_3_space() -> Option<(SimplicialFixture, Vec<usize>, Vec<usize>)> {
    Some((torus_3(), vec![1, 3, 3, 1], vec![1, 3, 3, 1]))
}

/// The tenth space, withheld under Miri. See the sibling above for why.
#[cfg(miri)]
fn torus_3_space() -> Option<(SimplicialFixture, Vec<usize>, Vec<usize>)> {
    None
}

/// Every ordering of the `dim` axes, `dim!` of them.
///
/// Grown one slot at a time: each partial order is extended by every axis it does not already
/// carry, so an axis appears exactly once per order.
fn axis_orders(dim: usize) -> Vec<Vec<usize>> {
    let mut orders = vec![Vec::new()];
    for _ in 0..dim {
        let mut next: Vec<Vec<usize>> = Vec::new();
        for order in &orders {
            for axis in 0..dim {
                if !order.contains(&axis) {
                    let mut extended = order.clone();
                    extended.push(axis);
                    next.push(extended);
                }
            }
        }
        orders = next;
    }
    orders
}

/// Triangulates a `D`-dimensional lattice of `sizes[d]` cubes per axis and glues it by `ident`.
///
/// Each unit cube splits into `D!` simplices, one per ordering of the axes: start at the low corner
/// and step `+1` along the axes in that order. That is the Kuhn (Freudenthal) triangulation, and
/// the pieces meet face to face, so the union is simplicial. At `D == 2` it is the square split
/// along its diagonal.
///
/// `ident` maps a lattice point to its canonical representative, which is what performs the
/// gluing. A vertex is named by the place-value reading of that representative, one digit per axis.
fn lattice_quotient<const D: usize>(
    name: &'static str,
    sizes: [usize; D],
    ident: impl Fn([usize; D]) -> [usize; D],
) -> SimplicialFixture {
    let vid = |p: [usize; D]| {
        ident(p)
            .iter()
            .zip(sizes.iter())
            .fold(0, |acc, (c, n)| acc * (n + 1) + c)
    };
    let orders = axis_orders(D);
    let cubes: usize = sizes.iter().product();
    let mut facets: Vec<Vec<usize>> = Vec::new();
    for flat in 0..cubes {
        // The cube's low corner, read off the flat index one axis at a time.
        let mut low = [0usize; D];
        let mut rest = flat;
        for (c, n) in low.iter_mut().zip(sizes.iter()).rev() {
            *c = rest % n;
            rest /= n;
        }
        for order in &orders {
            let mut corner = low;
            let mut t = vec![vid(corner)];
            for &axis in order {
                corner[axis] += 1;
                t.push(vid(corner));
            }
            t.sort_unstable();
            t.dedup();
            // A degenerate simplex means the gluing folded two corners together, which the grid
            // sizes are chosen to avoid. Keeping only the non-degenerate ones would hide it.
            assert_eq!(
                t.len(),
                D + 1,
                "{name}: the gluing made a degenerate simplex"
            );
            facets.push(t);
        }
    }
    let refs: Vec<&[usize]> = facets.iter().map(|f| f.as_slice()).collect();
    SimplicialFixture::new(name, &refs)
}

/// The 2-torus: wrap in both directions.
fn torus_2() -> SimplicialFixture {
    lattice_quotient("torus_2", [3, 3], |[x, y]| [x % 3, y % 3])
}

/// The 3-torus: wrap in all three directions. Withheld under Miri; [`torus_3_space`] says why.
#[cfg(not(miri))]
fn torus_3() -> SimplicialFixture {
    lattice_quotient("torus_3", [3, 3, 3], |[x, y, z]| [x % 3, y % 3, z % 3])
}

/// The Klein bottle: wrap in `x` with no flip, wrap in `y` with a flip in `x`.
fn klein_bottle() -> SimplicialFixture {
    lattice_quotient("klein_bottle", [4, 4], |[x, y]| {
        if y >= 4 { [(4 - x) % 4, 0] } else { [x % 4, y] }
    })
}

/// The cylinder: wrap in `x` only, so the `y` edges stay free.
fn cylinder() -> SimplicialFixture {
    lattice_quotient("cylinder", [3, 1], |[x, y]| [x % 3, y])
}

/// The Möbius band: wrap in `x` with a flip in `y`, leaving `y` free.
fn mobius_band() -> SimplicialFixture {
    lattice_quotient("mobius_band", [3, 1], |[x, y]| {
        if x >= 3 { [0, 1 - y] } else { [x, y] }
    })
}
