/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Float106;

/// Marker trait: Promises that `(a * b) * c == a * (b * c)`.
///
/// The compiler cannot check this, so implementing it is a promise by the developer.
///
/// DO NOT IMPLEMENT for `Octonion`, whose multiplication is not associative.
///
/// Non-primitive types carry their own impls in the crate that defines them: `Complex<T>`,
/// `Quaternion<T>` and `Octonion<T>` in `deep_causality_num_complex`, `Dual<T>` in
/// `deep_causality_num_dual`, `Rational<T>` in `deep_causality_num_rational`, and the tensor and
/// sparse-matrix types in their own crates.///
/// # Why these are written out one by one
///
/// This trait was once blanket-implemented over `Num`, which is unsealed: any downstream type
/// implementing `Num` silently acquired this law without anyone promising it, and could then enter
/// `CommutativeRing` and `Field` on a claim nobody made. A marker whose whole purpose is to record
/// an unverifiable promise cannot be handed out by inference.
///
/// Listing the types is the point, not an accident of style. Each line is one deliberate
/// assertion about one type, and the repetition is the cost of making the promise explicit.
/// The workspace also forbids macros in library code (`AGENTS.md`), so the list stays literal.
pub trait Associative {}

// The real scalars.
impl Associative for f32 {}
impl Associative for f64 {}
impl Associative for Float106 {}

// The integers. ℤ is a commutative ring, so all three laws hold.
impl Associative for i8 {}
impl Associative for i16 {}
impl Associative for i32 {}
impl Associative for i64 {}
impl Associative for i128 {}
impl Associative for isize {}

// The naturals. ℕ is a commutative semiring: it has no additive inverses, but the three
// multiplicative laws are unaffected by that.
impl Associative for u8 {}
impl Associative for u16 {}
impl Associative for u32 {}
impl Associative for u64 {}
impl Associative for u128 {}
impl Associative for usize {}
