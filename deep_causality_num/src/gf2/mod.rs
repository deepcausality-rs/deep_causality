/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! 𝔽₂, the two-element field, and the tower's first finite field.
//!
//! # Why a scalar, when the matrices are bit-packed
//!
//! `deep_causality_linear` stores an 𝔽₂ matrix one bit per entry, because at n=2048 that runs 3.2×
//! faster than a byte-per-entry scalar on one eighth the memory. That measurement decides the
//! **storage**. It says nothing about the **element type**: a packed matrix still has to answer
//! `get` with a value, and that value is an element of 𝔽₂.
//!
//! Both facts hold at once — pack the bits, name the element. `Gf2` is the name.
//!
//! # Why it lives here
//!
//! `deep_causality_linear` bounds every operation on a trait the tower already publishes and defines
//! no scalar of its own, which is the discipline the `deep_causality_num` split was about. 𝔽₂ was
//! the one scalar the tower did not carry, so it is added here rather than invented there. The law
//! markers are implemented in `deep_causality_algebra`, the same arrangement `i8` has: the type is
//! foreign to that crate, and `impl IntegralDomain for i8 {}` is written there regardless.
//!
//! # The arithmetic
//!
//! Addition is exclusive-or and multiplication is conjunction. Every element is its own additive
//! inverse, so negation is the identity and subtraction is addition. The only unit is `1`, and it is
//! its own multiplicative inverse.

mod display;
mod ops_arithmetic;
mod traits_num;

/// An element of 𝔽₂, the field with two elements.
///
/// Represented as a `bool`: `false` is the additive identity, `true` the multiplicative identity.
///
/// # Arithmetic
///
/// | operation | 𝔽₂ | `bool` |
/// |---|---|---|
/// | `a + b` | `a ⊕ b` | `^` |
/// | `a - b` | `a ⊕ b` | `^` |
/// | `-a` | `a` | identity |
/// | `a · b` | `a ∧ b` | `&` |
/// | `a / b` | `a`, for `b = 1` | panics for `b = 0` |
///
/// Addition and subtraction coincide because `1 + 1 = 0`, so every element is its own additive
/// inverse. That is the fact `deep_causality_algebra::DivisibleByIntegers` exists to keep out of
/// generic code that halves: over 𝔽₂, `one + one` is `zero`, and dividing by it divides by zero.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Gf2(bool);

impl Gf2 {
    /// The additive identity, `0`.
    pub const ZERO: Self = Gf2(false);

    /// The multiplicative identity, `1`.
    pub const ONE: Self = Gf2(true);

    /// Wraps a bit as an element of 𝔽₂: `false` is `0`, `true` is `1`.
    #[inline]
    pub const fn new(bit: bool) -> Self {
        Gf2(bit)
    }

    /// Returns the element as a bit: `0` is `false`, `1` is `true`.
    #[inline]
    pub const fn bit(&self) -> bool {
        self.0
    }

    /// Reduces an integer modulo 2.
    ///
    /// This is the conversion the boundary operators need. `deep_causality_topology` stores its
    /// boundary matrices as `CsrMatrix<i8>` with entries in `{-1, 0, 1}`, and both `-1` and `1`
    /// are the 𝔽₂ one — `-1 ≡ 1 (mod 2)`.
    #[inline]
    pub const fn from_i64_mod2(value: i64) -> Self {
        Gf2(value % 2 != 0)
    }
}
