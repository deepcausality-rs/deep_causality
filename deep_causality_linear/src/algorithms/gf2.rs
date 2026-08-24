/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Exact linear algebra over 𝔽₂.
//!
//! These are thin names over the generic elimination in
//! [`elimination`](crate::algorithms::elimination), fixed to the bit-packed representation. They
//! exist as separate functions rather than as a documentation note because the choice of field must
//! be visible at the call site: rank over ℝ, rank over ℤ and rank over 𝔽₂ are three different
//! questions, and conflating two of them is what `qcl-gaps.md` G-02 records.

use crate::errors::linear_error::LinearError;
use crate::types::packed_gf2::PackedGf2;
use deep_causality_num::NaturalNumber;

/// The rank over 𝔽₂, exactly.
///
/// Takes no tolerance and applies none. Every non-zero element of 𝔽₂ is its own inverse, so the
/// elimination divides by nothing that could be near zero.
pub fn rank_gf2<W>(m: &PackedGf2<W>) -> Result<usize, LinearError>
where
    W: NaturalNumber,
{
    let _ = m;
    todo!("rank_gf2")
}

/// A basis of the kernel over 𝔽₂.
///
/// Has exactly `cols - rank` elements, and `m · v = 0` over 𝔽₂ for each. This and
/// [`image_basis_gf2`] are what `qcl-gaps.md` G-01 names as its closure condition: 𝔽₂ homology with
/// representatives needs `ker ∂₁ / im ∂₂` as spanning sets rather than as dimensions.
pub fn kernel_basis_gf2<W>(m: &PackedGf2<W>) -> Result<PackedGf2<W>, LinearError>
where
    W: NaturalNumber,
{
    let _ = m;
    todo!("kernel_basis_gf2")
}

/// A basis of the image over 𝔽₂.
///
/// Has exactly `rank` elements, and every column of `m` is an 𝔽₂ sum of them.
pub fn image_basis_gf2<W>(m: &PackedGf2<W>) -> Result<PackedGf2<W>, LinearError>
where
    W: NaturalNumber,
{
    let _ = m;
    todo!("image_basis_gf2")
}
