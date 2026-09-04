/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{BFloat16, Float106};

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
/// # Scope: the finite floats
///
/// The intended model of `f32`, `f64`, `BFloat16` and `Float106` is ℝ. `NaN` and the infinities are artifacts
/// of the machine representation rather than real numbers, and every law in this tower is asserted
/// over the finite values. Annihilation is the plainest case: `0.0 * f64::NAN` is `NaN`, and so is
/// `0.0 * f64::INFINITY`, neither of which is zero.
///
/// The same boundary applies to the rest of the tower, so no marker carries a finite-value caveat
/// of its own. [`Commutative`](crate::Commutative) states `a * b == b * a`, yet
/// `NaN * 1.0 == 1.0 * NaN` is `false`, because `NaN` compares unequal to itself.
/// [`Distributive`](crate::Distributive) fails the moment a `NaN` reaches either side of the
/// equation. `a + (-a) == 0` fails at `NaN` and at the infinities, which takes
/// [`AbelianGroup`](crate::AbelianGroup) with it.
///
/// Generic code that depends on the laws therefore has to keep its floats finite.
/// [`Real::is_finite`](crate::Real::is_finite) is the predicate that decides it.
///
/// # Implementing
///
/// Implement it for any type whose zero really does annihilate. Past the primitives below, seven
/// types carry the marker, each in the crate that defines it: `Complex<T>`, `Quaternion<T>` and
/// `Octonion<T>` in `deep_causality_num_complex`, `Dual<T>` in `deep_causality_num_dual`,
/// `Rational<T>` in `deep_causality_num_rational`, and `CausalTensor<T>` and
/// `CausalTensorTrain<T>` in `deep_causality_tensor`. `CsrMatrix<T>` in `deep_causality_sparse`
/// does not: it stops at [`AbelianGroup`](crate::AbelianGroup) and carries none of the
/// multiplicative markers.
///
/// The types are listed one by one rather than inferred, for the reason given in
/// [`Commutative`](crate::Commutative).
pub trait Annihilating {}

// The real scalars.
impl Annihilating for f32 {}
impl Annihilating for f64 {}
impl Annihilating for Float106 {}
impl Annihilating for BFloat16 {}

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

// 𝔽₂. `0 · a = 0`, since multiplication is conjunction.
impl Annihilating for deep_causality_num::Gf2 {}
