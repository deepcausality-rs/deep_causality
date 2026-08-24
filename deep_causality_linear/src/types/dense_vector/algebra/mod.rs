/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Where `DenseVector` sits in the algebra tower.
//!
//! # A vector has no multiplication
//!
//! It carries the additive law markers and no multiplicative ones, because there is no product of
//! two vectors that returns a vector. The dot product returns a scalar and the outer product returns
//! a matrix; neither is a `Mul` on this type, so claiming `Associative<Multiplicative>` would be
//! claiming a law about an operation that does not exist.
//!
//! That places it at `AbelianGroup` and no higher on the multiplicative side, which is correct — a
//! vector space is not a ring.
//!
//! # `Module<R>` is the point
//!
//! [`Module<R: Ring>`](deep_causality_algebra::Module) is the tower's name for a vector space, and
//! it is what this type is. Stating it at the general level — a module over a *ring* — is what
//! admits ℤ: `DenseVector<i64>` scaled by `i64` is a module and is not a vector space, and a `Field`
//! bound would have excluded it for no reason. The census found 60 rank-1 constructions, and
//! `deep_causality_topology`'s chains are integer.

use crate::types::dense_vector::DenseVector;
use deep_causality_algebra::{Additive, Annihilating, Associative, Commutative};

impl<T> Associative<Additive> for DenseVector<T> where T: Associative<Additive> + Clone {}
impl<T> Commutative<Additive> for DenseVector<T> where T: Commutative<Additive> + Clone {}
impl<T> Annihilating for DenseVector<T> where T: Annihilating + Clone {}

// Deliberately absent: every multiplicative marker, and `Distributive`. There is no `Mul` on this
// type for them to be about.
