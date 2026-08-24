/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::algebra::operator::{Additive, Multiplicative, Operator};

use crate::Float106;

/// Marker trait: Promises that `a * b == b * a`.
///
/// The compiler cannot check this, so implementing it is a promise by the developer.
///
/// DO NOT IMPLEMENT for `Quaternion` or `Octonion`, whose multiplication does not commute.
///
/// # Which types promise it
///
/// Past the primitives below, four types carry `Commutative`, each in the crate that defines it:
/// `Complex<T>` in `deep_causality_num_complex`, `Dual<T>` in `deep_causality_num_dual`,
/// `Rational<T>` in `deep_causality_num_rational`, and `CausalTensor<T>` and
/// `CausalTensorTrain<T>` in `deep_causality_tensor`. The two tensor impls are conditional on
/// their element type — multiplication there is element-wise, so the container commutes exactly
/// when its elements do, and a tensor of quaternions does not. This crate adds the monoid carriers
/// `Conjunction`, `Disjunction`, `Count` and `Prob`, which need the marker to reach
/// [`CommutativeMonoid`](crate::CommutativeMonoid).
///
/// Two absences are deliberate. `Quaternion<T>` does not commute: `i * j` is `k`, and `j * i` is
/// `-k`. `Octonion<T>` inherits that failure. A third absence is structural: `CsrMatrix<T>` in
/// `deep_causality_sparse` stops at [`AbelianGroup`](crate::AbelianGroup) and carries none of the
/// multiplicative markers.
///
/// For `f32`, `f64` and `Float106` the promise covers the finite values. See the scope note on
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
pub trait Commutative<O: Operator> {}

// The real scalars.
impl Commutative<Multiplicative> for f32 {}
impl Commutative<Multiplicative> for f64 {}
impl Commutative<Multiplicative> for Float106 {}

// The integers. ℤ is a commutative ring, so all three laws hold.
impl Commutative<Multiplicative> for i8 {}
impl Commutative<Multiplicative> for i16 {}
impl Commutative<Multiplicative> for i32 {}
impl Commutative<Multiplicative> for i64 {}
impl Commutative<Multiplicative> for i128 {}
impl Commutative<Multiplicative> for isize {}

// The naturals. ℕ is a commutative semiring: it has no additive inverses, but the three
// multiplicative laws are unaffected by that.
impl Commutative<Multiplicative> for u8 {}
impl Commutative<Multiplicative> for u16 {}
impl Commutative<Multiplicative> for u32 {}
impl Commutative<Multiplicative> for u64 {}
impl Commutative<Multiplicative> for u128 {}
impl Commutative<Multiplicative> for usize {}

// Additive commutativity. This one does survive the machine: IEEE 754 addition is exactly
// commutative on the finite values, unlike associativity above.
// The real scalars.
impl Commutative<Additive> for f32 {}
impl Commutative<Additive> for f64 {}
impl Commutative<Additive> for Float106 {}

// The integers, ℤ.
impl Commutative<Additive> for i8 {}
impl Commutative<Additive> for i16 {}
impl Commutative<Additive> for i32 {}
impl Commutative<Additive> for i64 {}
impl Commutative<Additive> for i128 {}
impl Commutative<Additive> for isize {}

// The naturals, ℕ.
impl Commutative<Additive> for u8 {}
impl Commutative<Additive> for u16 {}
impl Commutative<Additive> for u32 {}
impl Commutative<Additive> for u64 {}
impl Commutative<Additive> for u128 {}
impl Commutative<Additive> for usize {}
