/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Where `CsrMatrix` sits in the algebra tower.
//!
//! The same claims as `DenseMatrix`, for the same reasons: a matrix ring over a ring is a ring, and
//! matrix multiplication associates without commuting. Sparsity is a storage decision and changes no
//! algebraic law.
//!
//! `Distributive` and `Annihilating` were added to the sparse crate before this move, closing the
//! gap that had left the type at `AbelianGroup`.

use crate::types::csr_matrix::CsrMatrix;
use deep_causality_algebra::{
    Additive, Annihilating, Associative, Commutative, Distributive, Multiplicative,
};

impl<T> Associative<Additive> for CsrMatrix<T> where T: Associative<Additive> + Copy {}
impl<T> Commutative<Additive> for CsrMatrix<T> where T: Commutative<Additive> + Copy {}
impl<T> Associative<Multiplicative> for CsrMatrix<T> where T: Associative<Multiplicative> + Copy {}
impl<T> Distributive for CsrMatrix<T> where T: Distributive + Copy {}
impl<T> Annihilating for CsrMatrix<T> where T: Annihilating + Copy {}

// Deliberately absent: `Commutative<Multiplicative>` and `IntegralDomain`.
