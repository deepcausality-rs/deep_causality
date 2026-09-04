/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::algebra::operator::{Additive, Multiplicative, Operator};

use crate::{BFloat16, Float106};

/// Marker trait: Promises that `(a * b) * c == a * (b * c)`.
///
/// The compiler cannot check this, so implementing it is a promise by the developer.
///
/// DO NOT IMPLEMENT for `Octonion`, whose multiplication is not associative.
///
/// # Which types promise it
///
/// Past the primitives below, six types carry `Associative`, each in the crate that defines it:
/// `Complex<T>` and `Quaternion<T>` in `deep_causality_num_complex`, `Dual<T>` in
/// `deep_causality_num_dual`, `Rational<T>` in `deep_causality_num_rational`, and
/// `CausalTensor<T>` and `CausalTensorTrain<T>` in `deep_causality_tensor`. `Quaternion<T>` is on
/// that list although it is missing from [`Commutative`](crate::Commutative): ℍ associates, it
/// just does not commute.
///
/// `Octonion<T>` is absent, and the absence is the definition. 𝕆 satisfies only the weaker
/// alternative law, and associativity fails on any triple of imaginary units that does not lie in
/// a common quaternion subalgebra. A second absence is structural: `CsrMatrix<T>` in
/// `deep_causality_sparse` stops at [`AbelianGroup`](crate::AbelianGroup) and carries none of the
/// multiplicative markers.
///
/// For `f32`, `f64`, `BFloat16` and `Float106` the promise covers the finite values. See the scope note on
/// [`Annihilating`](crate::Annihilating).
///
/// # Why these are written out one by one
///
/// This trait was once blanket-implemented over `Num`, which is unsealed: any downstream type
/// implementing `Num` silently acquired this law without anyone promising it, and could then enter
/// `CommutativeRing` and `Field` on a claim nobody made. A marker whose whole purpose is to record
/// an unverifiable promise cannot be handed out by inference.
///
/// Listing the types is the point, not an accident of style. Each line is one deliberate
/// assertion about one type, and the repetition is the cost of making the promise explicit.
/// `AGENTS.md` also steers library code away from macros, so the list stays literal.
pub trait Associative<O: Operator> {}

// The real scalars.
impl Associative<Multiplicative> for f32 {}
impl Associative<Multiplicative> for f64 {}
impl Associative<Multiplicative> for Float106 {}
impl Associative<Multiplicative> for BFloat16 {}

// The integers. ℤ is a commutative ring, so all three laws hold.
impl Associative<Multiplicative> for i8 {}
impl Associative<Multiplicative> for i16 {}
impl Associative<Multiplicative> for i32 {}
impl Associative<Multiplicative> for i64 {}
impl Associative<Multiplicative> for i128 {}
impl Associative<Multiplicative> for isize {}

// The naturals. ℕ is a commutative semiring: it has no additive inverses, but the three
// multiplicative laws are unaffected by that.
impl Associative<Multiplicative> for u8 {}
impl Associative<Multiplicative> for u16 {}
impl Associative<Multiplicative> for u32 {}
impl Associative<Multiplicative> for u64 {}
impl Associative<Multiplicative> for u128 {}
impl Associative<Multiplicative> for usize {}

// Additive associativity. For the floats the promise is to the model ℝ: floating-point addition
// does not associate — `(0.1+0.2)+0.3 != 0.1+(0.2+0.3)`, and `(1e16 + -1e16) + 1.0` is `1.0`
// where `1e16 + (-1e16 + 1.0)` is `0.0`. Restricting to finite values does not recover it,
// because the cause is rounding rather than a NaN or an infinity.
// The real scalars.
impl Associative<Additive> for f32 {}
impl Associative<Additive> for f64 {}
impl Associative<Additive> for Float106 {}
impl Associative<Additive> for BFloat16 {}

// The integers, ℤ.
impl Associative<Additive> for i8 {}
impl Associative<Additive> for i16 {}
impl Associative<Additive> for i32 {}
impl Associative<Additive> for i64 {}
impl Associative<Additive> for i128 {}
impl Associative<Additive> for isize {}

// The naturals, ℕ.
impl Associative<Additive> for u8 {}
impl Associative<Additive> for u16 {}
impl Associative<Additive> for u32 {}
impl Associative<Additive> for u64 {}
impl Associative<Additive> for u128 {}
impl Associative<Additive> for usize {}

// 𝔽₂. Addition is exclusive-or and multiplication is conjunction, both of which associate
// exactly. Unlike the floats, this is not a promise to a model the representation approximates —
// the field has two elements and the laws are checked exhaustively.
impl Associative<Additive> for deep_causality_num::Gf2 {}
impl Associative<Multiplicative> for deep_causality_num::Gf2 {}
