/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Float106;

/// Marker trait: promises that `Div` computes the true **multiplicative inverse**, so that
/// `a * (Self::one() / a) == Self::one()` for every non-zero `a`.
///
/// This is the field-division law, and it is the one axiom that separates a `Field` from a
/// `CommutativeRing` that merely happens to have a `/` operator. The compiler cannot check it, so
/// implementing this trait is a promise by the developer.
///
/// The motivating counter-example is the integers. `i64` supplies both `Div` and `DivAssign`, but
/// `/` on `ℤ` is truncating Euclidean quotient, not inversion: `1 / 5 == 0`, so `5 * (1 / 5) == 0`,
/// not `1`. Without this marker, `ℤ` would satisfy `InvMonoid` structurally and the tower would
/// conclude that `ℤ` is a `Field` — which is false, and is exactly the counter-example named in
/// [`Field`](crate::Field)'s own documentation.
///
/// DO NOT IMPLEMENT for any integer type, signed or unsigned.
///
/// `Complex<T>` and `Quaternion<T>` carry their own impls in `deep_causality_num_complex`, and
/// `Rational<T>` in `deep_causality_num_rational`: ℂ, ℍ and ℚ all genuinely invert. `Dual<T>` does
/// not, and must not have one — `ε` is a zero divisor.
pub trait Invertible {}

// The real scalars divide exactly, up to the rounding inherent in the representation. Rounding is
// a precision matter rather than an algebraic one, so it does not disturb the law.
//
// Written out per type rather than blanket-implemented over `Float`: the marker records a promise
// the compiler cannot verify, and `Float` is unsealed, so inference must not hand it out.
impl Invertible for f32 {}
impl Invertible for f64 {}
impl Invertible for Float106 {}

// 𝔽₂. The single non-zero element is `1`, and `1 · (1 / 1) = 1`, so field division holds exactly.
// Unlike the reals, no rounding qualifies the promise: the whole of the law is one case, and it is
// checked in the tests rather than argued for.
impl Invertible for deep_causality_num::Gf2 {}
