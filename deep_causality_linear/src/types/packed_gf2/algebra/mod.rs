/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Where `PackedGf2` sits in the algebra tower.
//!
//! # The scalar is fixed, so the markers carry no scalar bound
//!
//! Unlike the other three containers, this one is not generic over its scalar — every entry is a
//! [`Gf2`](deep_causality_num::Gf2). The law markers therefore state the matrix's laws outright
//! rather than inheriting them from a type parameter, because 𝔽₂ satisfies every one of them and
//! that is checked exhaustively in `deep_causality_algebra`: the field has two elements, so
//! associativity, commutativity and distributivity are eight triples rather than a promise.
//!
//! # The same two absences
//!
//! Matrix multiplication over 𝔽₂ does not commute — `[[1,1],[0,1]]` and `[[1,0],[1,1]]` are the
//! witness — and a matrix over 𝔽₂ with a zero row is a zero divisor. So `Commutative<Multiplicative>`
//! and `IntegralDomain` are absent here for exactly the reasons they are absent on the dense matrix,
//! even though the *scalar* 𝔽₂ has both.
//!
//! That is the distinction the operator-parameterised markers exist to keep straight: a law is a
//! property of a set together with an operation, and `(𝔽₂, ×)` commuting says nothing about
//! `(𝔽₂ᵐˣⁿ, ×)`.

use crate::types::packed_gf2::PackedGf2;
use deep_causality_algebra::{
    Additive, Annihilating, Associative, Commutative, Distributive, Multiplicative,
};
use deep_causality_num::NaturalNumber;

impl<W> Associative<Additive> for PackedGf2<W> where W: NaturalNumber {}
impl<W> Commutative<Additive> for PackedGf2<W> where W: NaturalNumber {}
impl<W> Associative<Multiplicative> for PackedGf2<W> where W: NaturalNumber {}
impl<W> Distributive for PackedGf2<W> where W: NaturalNumber {}
impl<W> Annihilating for PackedGf2<W> where W: NaturalNumber {}

// Deliberately absent: `Commutative<Multiplicative>` and `IntegralDomain`. See the module header.
