/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Where `PackedGf2` sits in the tower.
//!
//! The scalar 𝔽₂ is a commutative field. The matrix over it is neither commutative nor a domain,
//! and that distinction is what the operator-parameterised markers exist to keep straight: a law is
//! a property of a set together with an operation, and `(𝔽₂, ×)` commuting says nothing about
//! `(𝔽₂ᵐˣⁿ, ×)`.

use deep_causality_algebra::{
    Additive, Annihilating, Associative, Commutative, Distributive, Module, Multiplicative, Ring,
};
use deep_causality_linear::PackedGf2;

fn admits_associative_add<T: Associative<Additive>>() {}
fn admits_commutative_add<T: Commutative<Additive>>() {}
fn admits_associative_mul<T: Associative<Multiplicative>>() {}
fn admits_distributive<T: Distributive>() {}
fn admits_annihilating<T: Annihilating>() {}
fn admits_ring<T: Ring>() {}
fn admits_module<M: Module<R>, R: Ring>() {}

#[test]
fn test_packed_gf2_carries_the_markers_that_reach_ring() {
    admits_associative_add::<PackedGf2<u64>>();
    admits_commutative_add::<PackedGf2<u64>>();
    admits_associative_mul::<PackedGf2<u64>>();
    admits_distributive::<PackedGf2<u64>>();
    admits_annihilating::<PackedGf2<u64>>();
}

#[test]
fn test_packed_gf2_reaches_ring_and_module_at_every_word_width() {
    admits_ring::<PackedGf2<u8>>();
    admits_ring::<PackedGf2<u64>>();
    admits_module::<PackedGf2<u64>, deep_causality_num::Gf2>();
}
