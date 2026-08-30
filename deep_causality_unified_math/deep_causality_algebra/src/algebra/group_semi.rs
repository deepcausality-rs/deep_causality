/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::algebra::operator::{Additive, Multiplicative};
use crate::{AddMagma, MulMagma};

use crate::algebra::associative::Associative;

/// Represents an **Additive Semigroup**.
///
/// A semigroup is an algebraic structure with a single associative binary
/// operation. Unlike a monoid, a semigroup does NOT require an identity element.
///
/// # Mathematical Definition
///
/// A set `S` with a binary operation `+` is an additive semigroup if:
/// 1. **Closure:** `a + b` is in `S` for all `a, b` in `S`. (Implicit in Rust).
/// 2. **Associativity:** `(a + b) + c = a + (b + c)` for all `a, b, c` in `S`.
///
/// # Examples
/// - Positive integers under addition (no zero identity).
/// - Non-empty strings under concatenation.
pub trait AddSemigroup: AddMagma + Associative<Additive> {}

// Blanket implementation
impl<T> AddSemigroup for T where T: AddMagma + Associative<Additive> {}

/// Represents a **Multiplicative Semigroup**.
///
/// A semigroup is an algebraic structure with a single associative binary
/// operation. Unlike a monoid, a semigroup does NOT require an identity element.
///
/// # Mathematical Definition
///
/// A set `S` with a binary operation `*` is a multiplicative semigroup if:
/// 1. **Closure:** `a * b` is in `S` for all `a, b` in `S`. (Implicit in Rust).
/// 2. **Associativity:** `(a * b) * c = a * (b * c)` for all `a, b, c` in `S`.
///
/// # Hierarchy
/// ```text
/// Magma (closure only)
///   ↓
/// Semigroup (+ associativity)
///   ↓
/// Monoid (+ identity)
/// ```
pub trait MulSemigroup: MulMagma + Associative<Multiplicative> {}

// Blanket implementation
impl<T> MulSemigroup for T where T: MulMagma + Associative<Multiplicative> {}
