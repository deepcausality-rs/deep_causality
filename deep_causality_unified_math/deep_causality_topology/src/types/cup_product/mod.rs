/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The cup product on cochains, generic over any complex whose cells split.
//!
//! On each `(p+q)`-cell the product is the signed sum, over that cell's
//! splittings, of the left cell's `α` value times the right cell's `β` value:
//!
//! ```text
//! (α ∪ β)(c) = Σ_{split ∈ c.split(p)} sign · α(split.left) · β(split.right)
//! ```
//!
//! The splitting rule comes from [`SplittableCell`], so one implementation
//! serves every complex family. That genericity is the point rather than a
//! convenience: the construction this serves (Haruna, arXiv:2511.15224) applies
//! to general CSS codes, and qLDPC codes carry arbitrary structure with no
//! geometry to lean on. A cup product specialised to lattices would reproduce
//! the toric code and reach nothing past it.
//!
//! # Relationship to `Topology::cup_product`
//!
//! [`Topology::cup_product`](crate::Topology::cup_product) is the older,
//! simplicial-only Alexander–Whitney product over a bundled
//! `Topology<R, G>` cochain. It uses the same convention as this one. This module
//! adds what quantum error correction needs and that surface cannot express:
//! the cubical case, genericity over [`CellularComplex`], and the `n`-fold form.
//!
//! # Scalars
//!
//! The bound is [`CommutativeRing`] plus `Copy`, not `RealField`. The product
//! adds, subtracts and multiplies coefficients and needs a zero; it never
//! divides, never orders, and never calls an analytic function. Bounding at the
//! weakest structure that carries the operation is what the algebra tower is
//! for, and it leaves the door open to coefficient rings a cohomology
//! computation may actually want, `Z_N` among them, without a second
//! implementation.
//!
//! # Cochain representation
//!
//! A `k`-cochain is a flat slice indexed by cell index within the complex's
//! `k`-skeleton, matching the convention already used across
//! `deep_causality_physics` for velocity one-forms and pressure zero-forms.
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
use crate::traits::cellular_complex::CellularComplex;
use crate::types::cochain::Cochain;
use deep_causality_algebra::CommutativeRing;
use std::collections::HashMap;

/// The cup product of a `p`-cochain with a `q`-cochain, yielding a
/// `(p+q)`-cochain over the same complex.
///
/// Each [`Cochain`] carries its own degree, so the product is three parameters
/// rather than five and a degree cannot be paired with the wrong values.
///
/// # Errors
///
/// Returns [`TopologyErrorEnum::InvalidGradeOperation`] when the degrees sum
/// past the complex's maximum cell dimension, since the caller has asked for a
/// cochain in a degree the complex does not have, and
/// [`TopologyErrorEnum::DimensionMismatch`] when a cochain's length does not
/// equal the number of cells of its stated degree.
pub fn cup_product<K, R>(
    complex: &K,
    alpha: &Cochain<R>,
    beta: &Cochain<R>,
) -> Result<Cochain<R>, TopologyError>
where
    K: CellularComplex,
    K::CellType: SplittableCell,
    R: CommutativeRing + Copy,
{
    let (alpha_degree, beta_degree) = (alpha.degree(), beta.degree());
    let (alpha, beta) = (alpha.values(), beta.values());
    // `checked_add` rather than `+`: the degrees are caller-supplied, and an
    // overflowing sum would panic in debug and wrap in release, the wrapped
    // value then passing the maximum-dimension check below.
    let Some(target) = alpha_degree.checked_add(beta_degree) else {
        return Err(TopologyError(TopologyErrorEnum::InvalidGradeOperation(
            format!(
                "cup product of degrees {alpha_degree} and {beta_degree} overflows the degree type"
            ),
        )));
    };
    if target > complex.max_dim() {
        return Err(TopologyError(TopologyErrorEnum::InvalidGradeOperation(
            format!(
                "cup product of degrees {alpha_degree} and {beta_degree} lands in degree {target}, \
                 but the complex has maximum dimension {}",
                complex.max_dim()
            ),
        )));
    }
    Cochain::new(alpha.to_vec(), alpha_degree)
        .check_len(complex.num_cells(alpha_degree), "left")?;
    Cochain::new(beta.to_vec(), beta_degree).check_len(complex.num_cells(beta_degree), "right")?;

    // The splitting of a lattice cell wraps on periodic axes, so it needs the
    // ambient layout. Simplicial cells ignore it.
    let layout = complex.uniform_lattice_layout();

    let left_index: HashMap<K::CellType, usize> = complex
        .cells(alpha_degree)
        .enumerate()
        .map(|(i, c)| (c, i))
        .collect();
    let right_index: HashMap<K::CellType, usize> = complex
        .cells(beta_degree)
        .enumerate()
        .map(|(i, c)| (c, i))
        .collect();

    let mut out = vec![R::zero(); complex.num_cells(target)];
    for (i, cell) in complex.cells(target).enumerate() {
        let mut acc = R::zero();
        for split in cell.split(alpha_degree, layout.as_ref()) {
            // A split term whose partner lies outside the complex, which happens
            // at the boundary of a non-periodic axis, contributes nothing.
            let (Some(&l), Some(&r)) =
                (left_index.get(split.left()), right_index.get(split.right()))
            else {
                continue;
            };
            let term = alpha[l] * beta[r];
            if split.sign() >= 0 {
                acc = acc + term;
            } else {
                acc = acc - term;
            }
        }
        out[i] = acc;
    }
    Ok(Cochain::new(out, target))
}

/// The `n`-fold cup product: a left fold of [`cup_product`] over `factors`.
///
/// A slice of one type rather than a slice of `(cochain, degree)` tuples paired
/// by convention, which is the other half of what [`Cochain`] buys.
///
/// Associativity is what makes this well defined without new machinery, and it
/// is what yields the triple product on a three-dimensional complex, the degree
/// where the multi-controlled gates live.
///
/// # Errors
///
/// Returns [`TopologyErrorEnum::InvalidInput`] when `factors` is empty, since
/// there is no unit cochain to return, and otherwise propagates every error
/// [`cup_product`] can raise. A single factor is returned unchanged, after its
/// length is validated.
pub fn cup_product_n<K, R>(complex: &K, factors: &[Cochain<R>]) -> Result<Cochain<R>, TopologyError>
where
    K: CellularComplex,
    K::CellType: SplittableCell,
    R: CommutativeRing + Copy,
{
    let Some((first, rest)) = factors.split_first() else {
        return Err(TopologyError(TopologyErrorEnum::InvalidInput(
            "n-fold cup product needs at least one factor; there is no unit cochain to return"
                .to_string(),
        )));
    };
    let first_degree = &first.degree();
    // The grade contract is checked here as well as inside `cup_product`,
    // because a single factor never reaches the binary path. Without this, an
    // empty cochain at a degree above the complex's dimension would be accepted:
    // `num_cells` reports zero cells for such a degree, so the length check
    // alone passes and the function would return `Ok` where the binary API
    // rejects the same request.
    if *first_degree > complex.max_dim() {
        return Err(TopologyError(TopologyErrorEnum::InvalidGradeOperation(
            format!(
                "factor of degree {first_degree} exceeds the complex's maximum dimension {}",
                complex.max_dim()
            ),
        )));
    }
    first.check_len(complex.num_cells(*first_degree), "first")?;

    let mut acc = first.clone();
    for cochain in rest {
        acc = cup_product(complex, &acc, cochain)?;
    }
    Ok(acc)
}
