/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The cup product on cochains.
//!
//! The cup product takes a `p`-cochain and a `q`-cochain to a `(p+q)`-cochain.
//! On each `(p+q)`-cell it is the signed sum, over that cell's splittings, of
//! the left cell's `α` value times the right cell's `β` value:
//!
//! ```text
//! (α ∪ β)(c) = Σ_{split ∈ c.split(p)} sign · α(split.left) · β(split.right)
//! ```
//!
//! The splitting rule is supplied by [`SplittableCell`], so one implementation
//! serves every complex family. That genericity is the point rather than a
//! convenience: the construction this serves (Haruna, arXiv:2511.15224) applies
//! to general CSS codes, and qLDPC codes carry arbitrary structure with no
//! geometry to lean on. A cup product specialised to lattices would reproduce
//! the toric code and reach nothing past it.
//!
//! # Cochain representation
//!
//! A `k`-cochain is a flat slice indexed by cell index within the complex's
//! `k`-skeleton, matching the convention already used across
//! `deep_causality_physics` for velocity one-forms and pressure zero-forms.
//! There is deliberately no `Cochain` type: introducing one would force
//! conversions on existing callers to benefit new code.
//!
//! # Laws
//!
//! - **Leibniz**: `δ(α ∪ β) = δα ∪ β + (−1)^p α ∪ δβ` against this crate's own
//!   `coboundary_matrix` (Chen & Tata, arXiv:2106.05274, Prop. 3).
//! - **Associativity**: `(α ∪ β) ∪ γ = α ∪ (β ∪ γ)`, which is what makes the
//!   `n`-fold form a plain fold.
//! - **Graded commutativity on cohomology**: `α ∪ β` and `(−1)^{pq} β ∪ α`
//!   differ by a coboundary, which is why a logical gate built on this depends
//!   on the homology class rather than the representative cycle.

use crate::errors::topology_error::{TopologyError, TopologyErrorEnum};
use crate::traits::cell_splitting::SplittableCell;
use crate::traits::chain_complex::ChainComplex;
use deep_causality_algebra::RealField;

/// The cup product of a `p`-cochain with a `q`-cochain, yielding a
/// `(p+q)`-cochain over the same complex.
///
/// `alpha` and `beta` are indexed by cell index within the `p`- and
/// `q`-skeletons respectively.
///
/// # Errors
///
/// Returns [`TopologyErrorEnum::DimensionMismatch`] when a cochain's length does
/// not equal the number of cells of its stated degree, and
/// [`TopologyErrorEnum::InvalidGradeOperation`] when `alpha_degree + beta_degree`
/// exceeds the complex's maximum cell dimension, since the caller has asked for
/// a cochain in a degree the complex does not have.
pub fn cup_product<K, R>(
    complex: &K,
    alpha: &[R],
    alpha_degree: usize,
    beta: &[R],
    beta_degree: usize,
) -> Result<Vec<R>, TopologyError>
where
    K: ChainComplex,
    K::CellType: SplittableCell,
    R: RealField,
{
    let _ = (complex, alpha, alpha_degree, beta, beta_degree);
    let _ = TopologyErrorEnum::SimplexNotFound;
    todo!("cup-product: binary cup product over a splittable complex")
}

/// The `n`-fold cup product: a left fold of [`cup_product`] over `factors`,
/// each a `(cochain, degree)` pair.
///
/// Associativity is what makes this well defined without new machinery, and it
/// is what yields the triple product on a three-dimensional complex, the degree
/// where the multi-controlled gates live.
///
/// # Errors
///
/// Returns [`TopologyErrorEnum::InvalidInput`] when `factors` is empty, since
/// there is no unit cochain to return, and otherwise propagates every error
/// [`cup_product`] can raise. A single factor is returned unchanged.
pub fn cup_product_n<K, R>(
    complex: &K,
    factors: &[(&[R], usize)],
) -> Result<Vec<R>, TopologyError>
where
    K: ChainComplex,
    K::CellType: SplittableCell,
    R: RealField,
{
    let _ = (complex, factors);
    todo!("cup-product: n-fold cup product as a left fold")
}
