/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Where ℚ sits in the algebra tower.
//!
//! `Rational<T>` is a **field**, and these four markers plus `Invertible` are what carry it
//! there: the tower derives `Ring`, `CommutativeRing`, and `Field` automatically from them
//! together with the structural `Add`/`Mul`/`Div`/`Zero`/`One` impls.
//!
//! Each is a promise the compiler cannot check, so each is stated deliberately:
//!
//! | Law | Why it holds for `a/b` |
//! | :--- | :--- |
//! | `Commutative`  | `(a/b)·(c/d) = ac/bd = ca/db = (c/d)·(a/b)`, inherited from `T` |
//! | `Associative`  | likewise, from associativity of `T` |
//! | `Distributive` | `(a/b)·((c/d)+(e/f))` expands to the same fraction either way |
//! | `AbelianGroup` | additive inverse `-a/b`, identity `0/1` |
//! | `Annihilating` | `0/1 · a/b = 0/b`, which reduces to `0/1` |
//! | `Invertible`   | every non-zero `a/b` has `b/a`, which is the field axiom |
//!
//! `Invertible` is the one that distinguishes ℚ from ℤ. The integers reach `CommutativeRing` and
//! stop there, because integer `/` truncates rather than inverts. Constructing ℚ as the field of
//! fractions is precisely the act of supplying the missing inverses, so ℚ earns the marker that
//! ℤ cannot have.
//!
//! Deliberately absent: [`Real`](deep_causality_algebra::Real). ℚ is not closed under the
//! analytic operations — there is no rational `sqrt(2)`, `exp(1)`, or `ln(2)` — so claiming the
//! analytic axis would be false. ℚ is a field without being analytic, the mirror of `Dual<T>`,
//! which is analytic without being a field.

use super::{Rational, RationalScalar};
use crate::{AbelianGroup, Annihilating, Associative, Commutative, Distributive, Invertible};

impl<T: RationalScalar> Commutative for Rational<T> {}
impl<T: RationalScalar> Associative for Rational<T> {}
impl<T: RationalScalar> Distributive for Rational<T> {}
impl<T: RationalScalar> AbelianGroup for Rational<T> {}
// `0/1 · a/b = 0/b`, which reduces to `0/1`. In a ring this is a theorem, but `Semiring` takes it
// as an axiom — the derivation needs an additive inverse — so it is stated rather than assumed.
impl<T: RationalScalar> Annihilating for Rational<T> {}
impl<T: RationalScalar> Invertible for Rational<T> {}
