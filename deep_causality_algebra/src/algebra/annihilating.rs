/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Float106;

/// Marker trait: promises that zero **annihilates** under multiplication, so that
/// `0 * a == a * 0 == 0` for every `a`.
///
/// # Why this needs its own marker
///
/// In a [`Ring`](crate::Ring) this law is a *theorem*, not an axiom:
///
/// ```text
/// 0·a = (0 + 0)·a = 0·a + 0·a        by the additive identity and distributivity
///   0 = 0·a                          by adding −(0·a) to both sides
/// ```
///
/// That last step consumes an additive inverse. A [`Semiring`](crate::Semiring) has none, so the
/// derivation is unavailable and annihilation has to be assumed separately. It is the one semiring
/// axiom that does not follow from the others, which is precisely why it cannot be left implicit.
///
/// Without this marker a `Semiring` bound would assert a law nobody had promised — the same defect
/// that [`Invertible`](crate::Invertible) exists to prevent one rung higher up, where `Div` alone
/// does not imply inversion.
///
/// # Implementing
///
/// Implement it for any type whose zero really does annihilate, which is every numeric type in the
/// workspace. It is listed per type rather than inferred, for the reason given in
/// [`Commutative`](crate::Commutative).
pub trait Annihilating {}

// The real scalars.
impl Annihilating for f32 {}
impl Annihilating for f64 {}
impl Annihilating for Float106 {}

// The integers. Here the law is derivable, but the marker is still stated so that `ℤ` and `ℕ`
// present the same surface and generic semiring code accepts both.
impl Annihilating for i8 {}
impl Annihilating for i16 {}
impl Annihilating for i32 {}
impl Annihilating for i64 {}
impl Annihilating for i128 {}
impl Annihilating for isize {}

// The naturals. This is the case that needs the promise: ℕ has no additive inverses, so nothing
// derives `0 * a = 0` for it.
impl Annihilating for u8 {}
impl Annihilating for u16 {}
impl Annihilating for u32 {}
impl Annihilating for u64 {}
impl Annihilating for u128 {}
impl Annihilating for usize {}
