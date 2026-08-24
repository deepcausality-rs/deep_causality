/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::AddGroup;
use crate::algebra::operator::{Additive, Multiplicative};
use crate::{
    AbelianGroup, Annihilating, Associative, Commutative, Distributive, Field, Float, Invertible,
    Real,
};

/// An ordered `Field` that is also an analytic real scalar.
///
/// `RealField` is exactly a [`Real`] that is also a [`Field`]: it adds field
/// invertibility (a total multiplicative inverse / division) on top of the analytic
/// surface (ordering, `sqrt`/`exp`/`ln`/`sin`/…, constants) provided by [`Real`].
///
/// All analytic operations are declared on [`Real`] and inherited here via the
/// supertrait, so every existing `T: RealField` bound resolves the same method set
/// it always did. The split lets analytic-but-non-field types (for example dual
/// numbers, used for automatic differentiation) implement [`Real`] without falsely
/// claiming `Field`/`RealField`.
///
/// This trait abstracts over concrete floating-point types like `f32` and `f64`.
pub trait RealField: Real + Field {}

// A number is an Abelian group under addition exactly when it has additive inverses, and
// `Neg` is the witness for those. That bound is load-bearing rather than incidental: the
// unsigned integers are a `Num` and do satisfy the commutative, associative, and distributive
// laws, but `3u64 - 5u64` has no value in `u64`, so ℕ is a commutative *monoid* under
// addition and never a group. `Neg` is exactly the property separating ℤ from ℕ, so requiring
// it here keeps the unsigned types out of `AbelianGroup`, and therefore out of `Ring`,
// `CommutativeRing`, and `Field` — none of which they satisfy.
//
// `Num` is kept in the bound, rather than the weaker `AddGroup`, because it is what makes this
// blanket disjoint from the concrete impls for `Complex<T>`, `Dual<T>`, `Quaternion<T>`, and
// the tensor types: none of those implement `Num`. `AddGroup` alone would overlap them, and it
// would not exclude the unsigned types either, since its inverse axiom rests on `Sub` merely
// existing.
impl<T> AbelianGroup for T where T: AddGroup + Commutative<Additive> {}

// Every `Float` that carries the law markers is a `RealField`. `Invertible` is what separates ℝ
// from ℤ here: it promises that `/` really inverts, which integer division does not.
impl<T> RealField for T where
    T: Float
        + Commutative<Multiplicative>
        + Commutative<Additive>
        + Associative<Multiplicative>
        + Associative<Additive>
        + Distributive
        + Annihilating
        + Invertible
{
}
