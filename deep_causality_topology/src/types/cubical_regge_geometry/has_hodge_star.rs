/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Cubical backend for the `HasHodgeStar<R>` capability trait — Phase R4.3.
//!
//! Computes the discrete Hodge ⋆ on grade-`k` forms over a `LatticeComplex<D, R>` as
//! a square diagonal `num_cells(k) × num_cells(k)` sparse matrix, following the
//! FEEC/DEC mass-matrix convention used by the existing simplicial path
//! (`ReggeGeometry<R>::hodge_star_operators`): the matrix acts multiplicatively on
//! k-form coefficients, with each diagonal entry equal to the dual-/primal-cell
//! volume ratio for that cell.
//!
//! # The closed form
//!
//! For an axis-aligned cubical k-cell at position `p` with orientation bitmask `o`
//! (k bits set), the dual (D−k)-cell has the complementary orientation `~o &
//! ((1<<D)−1)`. Under the four edge-length uniformity tiers:
//!
//! - **`UnitEdge`** — every primal and dual volume is `1`, so the Hodge ⋆ is the
//!   identity matrix at every grade `k ∈ [0, D]`.
//! - **`Uniform { length }`** — primal vol = `length^k`, dual vol = `length^(D-k)`,
//!   diagonal entry = `length^(D-2k)`. Same value for every k-cell at the given
//!   grade — the matrix is a scalar multiple of the identity.
//! - **`PerAxis { lengths }`** — primal vol = product of `lengths[a]` for `a` in
//!   the orientation bits, dual vol = product of `lengths[a]` for `a` in the
//!   complementary bits, diagonal entry = dual / primal. Varies per cell only if
//!   the lengths differ between axes.
//! - **`PerEdge`** — the dual-cell volume requires half-edge averages of incident
//!   top cubes, not a per-axis lookup. Deferred to R4.4 (the risk-mitigation gate
//!   in `tasks.md` Block R4.4); this impl panics with a clear message if
//!   invoked on a `PerEdge` geometry.
//!
//! # Verified expectations (design.md Decision 4)
//!
//! 2D `PerAxis` with axes `[a, b]`:
//! - ⋆_0 entry = `a · b` (vertex → 2-cube ratio).
//! - ⋆_1 entries = `b/a` for edges along axis 0, `a/b` for edges along axis 1.
//! - ⋆_2 entry = `1 / (a · b)` (2-cube → vertex ratio).

use super::{CubicalReggeGeometry, EdgeLengths};
use crate::traits::has_hodge_star::HasHodgeStar;
use crate::types::lattice_complex::LatticeComplex;
use crate::traits::chain_complex::ChainComplex;
use deep_causality_num::{FromPrimitive, RealField};
use deep_causality_sparse::CsrMatrix;
use std::borrow::Cow;

impl<const D: usize, R> HasHodgeStar<R> for CubicalReggeGeometry<D, R>
where
    R: RealField + FromPrimitive,
{
    type Complex = LatticeComplex<D, R>;

    fn hodge_star_matrix<'a>(
        &'a self,
        complex: &'a Self::Complex,
        k: usize,
    ) -> Cow<'a, CsrMatrix<R>> {
        // Out-of-range grade: return an empty 0x0 matrix. Mirrors the simplicial
        // `Manifold::hodge_star` behaviour for k > max_dim.
        if k > D {
            return Cow::Owned(CsrMatrix::new());
        }

        let n = complex.num_cells(k);
        let mut triplets: Vec<(usize, usize, R)> = Vec::with_capacity(n);

        match &self.edge_lengths {
            EdgeLengths::UnitEdge => {
                // Identity matrix: every diagonal entry is 1.
                for i in 0..n {
                    triplets.push((i, i, R::one()));
                }
            }
            EdgeLengths::Uniform { length } => {
                // Diagonal entry = length^(D - 2k). For k = D/2 with even D this
                // is 1; otherwise it is length raised to a signed integer power.
                let entry = pow_signed(*length, (D as isize) - 2 * (k as isize));
                for i in 0..n {
                    triplets.push((i, i, entry));
                }
            }
            EdgeLengths::PerAxis { lengths } => {
                // Per-cell diagonal: depends on which axes the cell occupies.
                let all_axes_mask: u32 = if D >= 32 { u32::MAX } else { (1u32 << D) - 1 };
                for (i, cell) in complex.cells(k).enumerate() {
                    let orientation = cell.orientation();
                    let complement = (!orientation) & all_axes_mask;

                    let mut primal = R::one();
                    let mut dual = R::one();
                    for (axis, length) in lengths.iter().enumerate() {
                        let bit = 1u32 << axis;
                        if (orientation & bit) != 0 {
                            primal *= *length;
                        }
                        if (complement & bit) != 0 {
                            dual *= *length;
                        }
                    }
                    triplets.push((i, i, dual / primal));
                }
            }
            EdgeLengths::PerEdge { .. } => {
                panic!(
                    "CubicalReggeGeometry::hodge_star_matrix on PerEdge geometry is \
                     deferred to R4.4 (per-edge dual-cell closed form requires \
                     half-edge averages of incident top cubes — see design.md \
                     Decision 4 and Risk 1). This panic is the explicit risk-gate \
                     marker; R4.4 replaces it with the real implementation."
                );
            }
        }

        let matrix = CsrMatrix::from_triplets(n, n, &triplets)
            .expect("Diagonal triplets always satisfy CSR validity for square shape.");
        Cow::Owned(matrix)
    }
}

/// Raise `base` to a signed integer power. `base^0 = R::one()`; negative exponents
/// are computed as the reciprocal of the positive power. Used for the `Uniform`
/// tier where the Hodge ⋆ scalar is `length^(D - 2k)`, which is negative for `k > D/2`.
fn pow_signed<R>(base: R, exp: isize) -> R
where
    R: RealField,
{
    if exp == 0 {
        return R::one();
    }
    let n = exp.unsigned_abs();
    let mut acc = R::one();
    for _ in 0..n {
        acc *= base;
    }
    if exp < 0 {
        R::one() / acc
    } else {
        acc
    }
}
