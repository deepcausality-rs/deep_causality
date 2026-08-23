/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Float;

/// Marker trait: promises that `Div` computes the true **multiplicative inverse**, so that
/// `a * (Self::one() / a) == Self::one()` for every non-zero `a`.
///
/// This is the field-division law, and it is the one axiom that separates a `Field` from a
/// `CommutativeRing` that merely happens to have a `/` operator. The compiler cannot check it,
/// so implementing this trait is a promise by the developer.
///
/// The motivating counter-example is the integers. `i64` supplies both `Div` and `DivAssign`,
/// but `/` on `ℤ` is truncating Euclidean quotient, not inversion: `1 / 5 == 0`, so
/// `5 * (1 / 5) == 0`, not `1`. Without this marker, `ℤ` would satisfy `InvMonoid` structurally
/// and the tower would conclude that `ℤ` is a `Field` — which is false, and is exactly the
/// counter-example named in [`Field`](crate::Field)'s own documentation.
///
/// IMPLEMENT THIS for `f32`, `f64`, `Float106`, `Complex`, `Quaternion`.
/// DO NOT IMPLEMENT for any integer type.
pub trait Invertible {}

// Every `Float` divides exactly (up to the rounding inherent in the representation, which is a
// precision matter and not an algebraic one). Integers are admitted to the tower as far as
// `CommutativeRing` but deliberately stop short of this marker.
impl<T> Invertible for T where T: Float {}
