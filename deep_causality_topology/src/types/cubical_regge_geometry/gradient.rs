/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Regge-action gradient for `CubicalReggeGeometry<D, R, S>` — Phase R6.
//!
//! Closed form on axis-aligned cubical lattices. The dihedral angle is
//! identically `π/2` regardless of edge lengths (see the `curvature` module
//! header), so the deficit angle `(4 − n) · π/2` carries no edge-length
//! sensitivity — `n` is the incident top-cube count, a purely combinatorial
//! quantity. All edge-length sensitivity of the action flows through the
//! hinge volume `vol(h)`:
//!
//! ```text
//! S_R = Σ_h  vol(h) · deficit(h)
//!
//! ∂S_R/∂L_i = Σ_{h : edge i is in h's axes}  (∂vol(h)/∂L_i) · deficit(h)
//!           = Σ_{h : edge i is in h's axes}  (vol(h) / L_i) · deficit(h)
//! ```
//!
//! by the product rule (`∂(L_a · L_b · …)/∂L_i = (L_a · L_b · …) / L_i` if
//! `L_i` is one of the factors; `0` otherwise).
//!
//! ## Per-dimension behaviour
//!
//! - **D = 2** — hinges are vertices (0-cells) with empty orientation; the
//!   hinge volume is the empty product `R::one()`, so `∂vol/∂L_i = 0` for
//!   every edge and the gradient is identically zero. The 2D action depends
//!   only on the deficit pattern, not on edge lengths.
//! - **D = 3** — hinges are edges (1-cells); each hinge contributes to
//!   exactly one edge (itself) with `∂vol/∂L_h = 1`. So
//!   `gradient[edge_id] = deficit(edge_id)`.
//! - **D = 4** — hinges are squares (2-cells); each square has two axes in
//!   its orientation and contributes to two edges. The contribution to each
//!   edge is the *other* axis length times the deficit.
//!
//! ## Locality
//!
//! Each edge `i`'s gradient entry depends only on the O(2^(D−1)) hinges that
//! contain edge `i` (a hinge can contain an edge only if the hinge's
//! orientation includes the edge's axis, and the hinge is one of the
//! finitely many (D−2)-cells incident to that edge). The total cost is
//! `O(num_edges · 2^D)` per gradient evaluation. This locality is what makes
//! the R6.4 single-edge Metropolis update efficient: `ΔS` from perturbing
//! one edge is `gradient[i] · δL_i + O(δL_i²)`, evaluable in O(2^D).
//!
//! ## Verification (R6.3)
//!
//! Property tests in
//! `tests/types/cubical_regge_geometry/regge_gradient_tests.rs` confirm the
//! closed form against a central finite-difference estimate
//! `(S(L + ε·δ_i) − S(L − ε·δ_i)) / (2ε)` to ~5 significant figures, and
//! verify the unit-edge equilibrium claim that the unit-length configuration
//! is a stationary point only when there is no boundary curvature.

use super::{CubicalReggeGeometry, Euclidean, Lorentzian, SignatureMarker};
use crate::traits::chain_complex::ChainComplex;
use crate::types::lattice_complex::LatticeComplex;
use deep_causality_num::{Complex, FromPrimitive, RealField};

impl<const D: usize, R: RealField + FromPrimitive, S: SignatureMarker>
    CubicalReggeGeometry<D, R, S>
{
    /// Real-valued hinge gradient sum, shared by the Euclidean and Lorentzian
    /// variants. Returns `∂(hinge_action_sum)/∂L_i` for every edge `i` in the
    /// lattice's canonical `iter_cells(1)` ordering.
    ///
    /// The deficit angle is signature-independent (purely combinatorial on
    /// axis-aligned cubical), so the per-edge derivative does not depend on
    /// `S`. The Euclidean wrapper returns this directly; the Lorentzian
    /// wrapper multiplies by `i` per the Wick-rotated action convention.
    pub(super) fn hinge_gradient_sum(&self, complex: &LatticeComplex<D, R>) -> Vec<R> {
        let num_edges = complex.num_cells(1);
        let mut grad = vec![R::zero(); num_edges];

        if D < 2 {
            return grad;
        }

        for (hinge_id, hinge) in complex.iter_cells(D - 2).enumerate() {
            let deficit = self.deficit_angle(complex, hinge_id);
            if deficit == R::zero() {
                continue;
            }
            let vol = self.cell_volume(complex, &hinge);
            let position = *hinge.position();
            let orientation = hinge.orientation();

            // Distribute (vol/L_axis) · deficit to each edge in the hinge's axes.
            for axis in 0..D {
                if (orientation & (1u32 << axis)) != 0 {
                    let edge_length = self.axis_length_at_position(complex, position, axis);
                    if edge_length == R::zero() {
                        continue;
                    }
                    let edge_id = complex.edge_index(position, axis);
                    if edge_id < num_edges {
                        grad[edge_id] += (vol / edge_length) * deficit;
                    }
                }
            }
        }

        grad
    }
}

/// Euclidean-only `regge_gradient` returning `Vec<R>`.
impl<const D: usize, R: RealField + FromPrimitive> CubicalReggeGeometry<D, R, Euclidean> {
    /// Closed-form gradient of the discrete Regge action with respect to each
    /// edge length, indexed by `edge_index` in the lattice's canonical
    /// `iter_cells(1)` order.
    ///
    /// Result length always equals `complex.num_cells(1)`. For `D < 2` the
    /// vector is all zeros (the action is empty). For `D = 2` the vector is
    /// also all zeros (hinges are vertices with empty product volume = 1).
    /// For `D ≥ 3` the entries are generically non-zero on open lattices
    /// (where boundary hinges carry deficit) and zero on periodic lattices
    /// (where every hinge has deficit 0).
    pub fn regge_gradient(&self, complex: &LatticeComplex<D, R>) -> Vec<R> {
        self.hinge_gradient_sum(complex)
    }
}

/// Lorentzian-only `regge_gradient` returning `Vec<Complex<R>>`.
///
/// Under the Wick-rotated convention `S_L = i · S_E` (see
/// `regge_action_lorentzian`), the Lorentzian gradient is also purely
/// imaginary: `∂S_L/∂L_i = i · ∂S_E/∂L_i`. Every returned entry has
/// `re = 0` and `im = ∂(hinge_action_sum)/∂L_i`.
impl<const D: usize, R: RealField + FromPrimitive> CubicalReggeGeometry<D, R, Lorentzian> {
    /// Closed-form gradient of the discrete Lorentzian Regge action with
    /// respect to each edge length, indexed by `edge_index`. Purely imaginary
    /// under the chosen Wick-rotation convention.
    pub fn regge_gradient(&self, complex: &LatticeComplex<D, R>) -> Vec<Complex<R>> {
        self.hinge_gradient_sum(complex)
            .into_iter()
            .map(|im| Complex { re: R::zero(), im })
            .collect()
    }
}
