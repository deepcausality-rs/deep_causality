/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Generators for the HKT law tests.
//!
//! The categorical laws are universally quantified, so a fixture cannot establish them: one
//! hand-built input at cursor 0 passes for implementations that are wrong everywhere else. These
//! generators sweep the shapes a law has to hold across, and every case is derived from an explicit
//! seed so a counterexample is reproducible from its report.

use crate::{Chain, Graph, Manifold, Simplex, SimplicialComplex, Skeleton};
use deep_causality_linear::CsrMatrix;
use deep_causality_tensor::CausalTensor;
use std::sync::Arc;

/// A seeded linear congruential generator.
///
/// Deterministic on purpose. A property test that cannot reproduce its own counterexample reports a
/// failure nobody can act on, so the seed is part of every case label.
#[derive(Debug, Clone)]
pub struct LawRng(u64);

impl LawRng {
    pub fn new(seed: u64) -> Self {
        Self(seed ^ 0x9E37_79B9_7F4A_7C15)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 11
    }

    pub fn below(&mut self, n: usize) -> usize {
        (self.next_u64() as usize) % n.max(1)
    }

    /// A scalar in `[-mag, mag]`, with the awkward values mixed in rather than left to chance:
    /// exact zero, a negative zero, a subnormal, and a large magnitude all appear.
    pub fn scalar(&mut self, mag: f64) -> f64 {
        match self.next_u64() % 16 {
            0 => 0.0,
            1 => -0.0,
            // `f64::MIN_POSITIVE` is the smallest positive *normal*, so it never reaches the
            // subnormal path. `from_bits(1)` is the smallest positive subnormal, 5e-324.
            2 => f64::from_bits(1),
            3 => mag * 1e12,
            _ => {
                let unit = (self.next_u64() % 1_000_001) as f64 / 1_000_000.0;
                unit * 2.0 * mag - mag
            }
        }
    }

    pub fn scalars(&mut self, n: usize, mag: f64) -> Vec<f64> {
        (0..n).map(|_| self.scalar(mag)).collect()
    }

    /// A scalar in `[-mag, mag]` with **no** extreme magnitudes mixed in.
    ///
    /// [`Self::scalar`] deliberately injects `1e12` and subnormals, which is right for structural
    /// laws (`fmap(id) == m` holds bit-for-bit whatever the payload is) and wrong for numerical
    /// ones. Summing terms that span twenty orders of magnitude loses roughly `1e12 * 2^-52` to
    /// cancellation, so a contraction test built on `scalar` measures float conditioning rather
    /// than the implementation. Numeric laws use this instead.
    pub fn well_scaled(&mut self, mag: f64) -> f64 {
        let unit = (self.next_u64() % 1_000_001) as f64 / 1_000_000.0;
        unit * 2.0 * mag - mag
    }

    pub fn well_scaled_vec(&mut self, n: usize, mag: f64) -> Vec<f64> {
        (0..n).map(|_| self.well_scaled(mag)).collect()
    }
}

/// A path complex on `n` vertices: `n` vertices and `n - 1` edges, so `2n - 1` simplices.
///
/// `n` varies across cases because a law that holds on a 2-vertex line and fails on a 4-vertex one
/// is exactly the kind of defect a single fixture hides.
pub fn path_complex<T>(n: usize) -> SimplicialComplex<T> {
    assert!(n >= 2, "a path complex needs at least two vertices");

    let vertices: Vec<Simplex> = (0..n).map(|i| Simplex::new(vec![i])).collect();
    let edges: Vec<Simplex> = (0..n - 1).map(|i| Simplex::new(vec![i, i + 1])).collect();

    let triplets: Vec<(usize, usize, i8)> = (0..n - 1)
        .flat_map(|e| [(e, e, -1i8), (e + 1, e, 1i8)])
        .collect();
    let d1 = CsrMatrix::from_triplets(n, n - 1, &triplets).expect("path boundary operator");

    SimplicialComplex::new(
        vec![Skeleton::new(0, vertices), Skeleton::new(1, edges)],
        vec![d1],
        vec![],
        Vec::new(),
    )
}

/// Total simplices in [`path_complex`], which is the data length `Manifold::new` requires.
///
/// Carries the same precondition as [`path_complex`]. Below two vertices the count would describe
/// a complex that cannot be built, and `2 * n - 1` underflows at `n == 0`.
pub fn path_complex_len(n: usize) -> usize {
    assert!(n >= 2, "a path complex needs at least two vertices");
    2 * n - 1
}

/// One generated case, carrying the label that reproduces it.
#[derive(Debug, Clone)]
pub struct Case<T> {
    pub label: String,
    pub value: T,
}

/// Manifolds over path complexes of several widths, **at every legal cursor**.
///
/// Sweeping the cursor is the point: `bind` and `extend` both thread a focus, and a suite that only
/// ever builds cursor 0 cannot see a focus that is dropped.
pub fn manifold_cases(seed: u64) -> Vec<Case<Manifold<SimplicialComplex<f64>, f64>>> {
    let mut rng = LawRng::new(seed);
    let mut out = Vec::new();

    for n in [2usize, 3, 5] {
        let len = path_complex_len(n);
        for cursor in 0..len {
            let vals = rng.scalars(len, 8.0);
            let tensor = CausalTensor::from_vec(vals, &[len]);
            let m = Manifold::new(path_complex(n), tensor, cursor)
                .expect("generated manifold should be well formed");
            out.push(Case {
                label: format!("seed={seed} vertices={n} len={len} cursor={cursor}"),
                value: m,
            });
        }
    }
    out
}

/// Graphs at every legal cursor, with widths that are not all equal to each other.
pub fn graph_cases(seed: u64) -> Vec<Case<Graph<f64>>> {
    let mut rng = LawRng::new(seed);
    let mut out = Vec::new();

    for n in [1usize, 2, 3, 6] {
        for cursor in 0..n {
            let vals = rng.scalars(n, 8.0);
            let tensor = CausalTensor::from_vec(vals, &[n]);
            let g = Graph::new(n, tensor, cursor).expect("generated graph should be well formed");
            out.push(Case {
                label: format!("seed={seed} vertices={n} cursor={cursor}"),
                value: g,
            });
        }
    }
    out
}

/// Chains over path complexes, across grades and sparsity patterns.
///
/// Every generated chain stores at least one weight, and the assertion below keeps it that way.
/// The empty chain is reachable input rather than a corner case, because CSR drops explicit zeros,
/// but `Adjunction::right_adjunct` and `Adjunction::counit` answer it with an `Err` instead of a
/// value. That case belongs in the tests written for it in `hkt_adjunction_law_tests`, not in a
/// sweep whose assertions read a stored weight.
pub fn chain_cases(seed: u64) -> Vec<Case<Chain<f64, f64>>> {
    let mut rng = LawRng::new(seed);
    let mut out = Vec::new();

    for n in [2usize, 3, 4] {
        let complex = Arc::new(path_complex::<f64>(n));
        for grade in 0..2usize {
            let cols = if grade == 0 { n } else { n - 1 };
            // Weights are drawn away from zero: see the assertion below.
            let weight = |rng: &mut LawRng| {
                let v = rng.well_scaled(6.0);
                if v.abs() < 0.5 { v + 1.0 } else { v }
            };
            // Dense row.
            let dense: Vec<(usize, usize, f64)> =
                (0..cols).map(|c| (0, c, weight(&mut rng))).collect();
            // Gapped row: every other column, so column positions differ from a 0..k run.
            let gapped: Vec<(usize, usize, f64)> = (0..cols)
                .filter(|c| c % 2 == 0)
                .map(|c| (0, c, weight(&mut rng)))
                .collect();

            for (kind, trips) in [("dense", dense), ("gapped", gapped)] {
                if trips.is_empty() {
                    continue;
                }
                let weights =
                    CsrMatrix::from_triplets(1, cols, &trips).expect("chain weight matrix");
                // A CSR drops explicit zeros, so a generated `0.0` weight silently becomes a
                // structural absence and the chain comes back empty. `Adjunction::right_adjunct`
                // and `Adjunction::counit` report that input as an error rather than returning a
                // value, so a law sweep must not contain it by accident; the error paths are
                // covered on purpose in `hkt_adjunction_law_tests`.
                assert_eq!(
                    weights.values().len(),
                    trips.len(),
                    "generated chain lost entries to zero weights"
                );
                out.push(Case {
                    label: format!("seed={seed} vertices={n} grade={grade} {kind}"),
                    value: Chain::new(complex.clone(), grade, weights),
                });
            }
        }
    }
    out
}

/// Equality that tolerates the float noise a lawful reassociation can introduce, and nothing more.
///
/// `NaN` is never equal to itself, so a law that produces one is reported as a failure rather than
/// silently passing an `assert!(x != x)`-shaped comparison. An infinity against a finite value is
/// reported the same way. Both tolerance terms evaluate to infinity there, and `inf <= inf` holds,
/// so without the finiteness check an overflowed law result would compare equal to anything.
pub fn approx_eq(a: f64, b: f64, tol: f64) -> bool {
    // Exact equality first: it settles `inf == inf` and `0.0 == -0.0` before any arithmetic runs.
    if a == b {
        return true;
    }
    // Catches `NaN` on either side as well, since `NaN` fails the equality above.
    if !a.is_finite() || !b.is_finite() {
        return false;
    }
    let diff = (a - b).abs();
    diff <= tol || diff <= tol * a.abs().max(b.abs())
}

/// Slice-wise [`approx_eq`], including a length check.
pub fn approx_eq_slice(a: &[f64], b: &[f64], tol: f64) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| approx_eq(*x, *y, tol))
}
