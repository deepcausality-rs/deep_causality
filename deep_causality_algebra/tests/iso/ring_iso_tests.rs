/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `deep_causality_algebra::iso::RingIso<T>` and the helper
//! `assert_ring_iso_from_laws`.
//!
//! Uses `IdRingWrap(f64)` as a local newtype that implements `Ring` via its
//! parent traits (`AbelianGroup`, `MulMonoid`, `Distributive`). The identity
//! iso to/from f64 trivially preserves both addition and multiplication.
//!
//! Coverage:
//! - Success: identity iso passes both homomorphism checks.
//! - Trait impl chain: `impl GroupIso<f64>` + `impl RingIso<f64>` for the
//!   newtype, exercising the trait-inheritance machinery.
//! - Panic (addition branch): broken `From` that adds a constant breaks the
//!   additive homomorphism first; helper panics on the `RingIso addition
//!   homomorphism failed` message.
//! - Panic (multiplication branch): broken `From` that scales by a constant
//!   breaks the multiplicative homomorphism while preserving addition;
//!   helper panics on the `RingIso multiplication homomorphism failed` message.

use core::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};
use deep_causality_algebra::iso::test_support::assert_ring_iso_from_laws;
use deep_causality_algebra::{
    AbelianGroup, Associative, Commutative, Distributive, GroupIso, RingIso,
};
use deep_causality_num::{One, Zero};

#[derive(Clone, Copy, PartialEq, Debug)]
struct IdRingWrap(f64);

impl Add for IdRingWrap {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        IdRingWrap(self.0 + rhs.0)
    }
}

impl AddAssign for IdRingWrap {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Sub for IdRingWrap {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        IdRingWrap(self.0 - rhs.0)
    }
}

impl SubAssign for IdRingWrap {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl Mul for IdRingWrap {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        IdRingWrap(self.0 * rhs.0)
    }
}

impl MulAssign for IdRingWrap {
    fn mul_assign(&mut self, rhs: Self) {
        self.0 *= rhs.0;
    }
}

impl Zero for IdRingWrap {
    fn zero() -> Self {
        IdRingWrap(0.0)
    }
    fn is_zero(&self) -> bool {
        self.0 == 0.0
    }
    fn set_zero(&mut self) {
        self.0 = 0.0;
    }
}

impl One for IdRingWrap {
    fn one() -> Self {
        IdRingWrap(1.0)
    }
    fn is_one(&self) -> bool {
        self.0 == 1.0
    }
}

impl Associative for IdRingWrap {}
impl Commutative for IdRingWrap {}
impl Distributive for IdRingWrap {}

impl AbelianGroup for IdRingWrap {}

impl From<f64> for IdRingWrap {
    fn from(x: f64) -> Self {
        IdRingWrap(x)
    }
}

impl From<IdRingWrap> for f64 {
    fn from(w: IdRingWrap) -> Self {
        w.0
    }
}

impl GroupIso<f64> for IdRingWrap {}
impl RingIso<f64> for IdRingWrap {}

#[test]
fn ring_iso_laws_hold_for_identity_iso() {
    assert_ring_iso_from_laws::<IdRingWrap, f64>(IdRingWrap(3.0), IdRingWrap(5.0));
    assert_ring_iso_from_laws::<IdRingWrap, f64>(IdRingWrap(0.0), IdRingWrap(0.0));
    assert_ring_iso_from_laws::<IdRingWrap, f64>(IdRingWrap(-2.0), IdRingWrap(7.0));
    assert_ring_iso_from_laws::<IdRingWrap, f64>(IdRingWrap(1.0), IdRingWrap(1.0));
}

#[test]
fn ring_iso_laws_hold_in_reverse_direction() {
    assert_ring_iso_from_laws::<f64, IdRingWrap>(3.0, 5.0);
    assert_ring_iso_from_laws::<f64, IdRingWrap>(-4.0, 9.0);
}

// A wrapper that breaks the additive ring homomorphism. From<BadAddRing> for
// f64 adds 1.0, so f64::from(a + b) = (a + b) + 1.0 differs from
// f64::from(a) + f64::from(b) = (a + 1.0) + (b + 1.0). The helper panics on the
// addition check first.
#[derive(Clone, Copy, PartialEq, Debug)]
struct BadAddRing(f64);

impl Add for BadAddRing {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        BadAddRing(self.0 + rhs.0)
    }
}

impl AddAssign for BadAddRing {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Sub for BadAddRing {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        BadAddRing(self.0 - rhs.0)
    }
}

impl SubAssign for BadAddRing {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl Mul for BadAddRing {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        BadAddRing(self.0 * rhs.0)
    }
}

impl MulAssign for BadAddRing {
    fn mul_assign(&mut self, rhs: Self) {
        self.0 *= rhs.0;
    }
}

impl Zero for BadAddRing {
    fn zero() -> Self {
        BadAddRing(0.0)
    }
    fn is_zero(&self) -> bool {
        self.0 == 0.0
    }
    fn set_zero(&mut self) {
        self.0 = 0.0;
    }
}

impl One for BadAddRing {
    fn one() -> Self {
        BadAddRing(1.0)
    }
    fn is_one(&self) -> bool {
        self.0 == 1.0
    }
}

impl Associative for BadAddRing {}
impl Commutative for BadAddRing {}
impl Distributive for BadAddRing {}

impl AbelianGroup for BadAddRing {}

impl From<f64> for BadAddRing {
    fn from(x: f64) -> Self {
        BadAddRing(x)
    }
}

impl From<BadAddRing> for f64 {
    fn from(w: BadAddRing) -> Self {
        w.0 + 1.0
    }
}

#[test]
#[should_panic(expected = "RingIso addition homomorphism failed")]
fn ring_iso_laws_panic_on_broken_addition() {
    assert_ring_iso_from_laws::<BadAddRing, f64>(BadAddRing(3.0), BadAddRing(5.0));
}

// A wrapper that preserves addition but breaks multiplication. From<BadMulRing>
// for f64 doubles the value, which preserves addition (doubling distributes
// over +) but breaks multiplication: f64::from(a * b) = 2.0 * a * b differs from
// f64::from(a) * f64::from(b) = (2.0 * a) * (2.0 * b) = 4.0 * a * b for a * b != 0.
#[derive(Clone, Copy, PartialEq, Debug)]
struct BadMulRing(f64);

impl Add for BadMulRing {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        BadMulRing(self.0 + rhs.0)
    }
}

impl AddAssign for BadMulRing {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Sub for BadMulRing {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        BadMulRing(self.0 - rhs.0)
    }
}

impl SubAssign for BadMulRing {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl Mul for BadMulRing {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        BadMulRing(self.0 * rhs.0)
    }
}

impl MulAssign for BadMulRing {
    fn mul_assign(&mut self, rhs: Self) {
        self.0 *= rhs.0;
    }
}

impl Zero for BadMulRing {
    fn zero() -> Self {
        BadMulRing(0.0)
    }
    fn is_zero(&self) -> bool {
        self.0 == 0.0
    }
    fn set_zero(&mut self) {
        self.0 = 0.0;
    }
}

impl One for BadMulRing {
    fn one() -> Self {
        BadMulRing(1.0)
    }
    fn is_one(&self) -> bool {
        self.0 == 1.0
    }
}

impl Associative for BadMulRing {}
impl Commutative for BadMulRing {}
impl Distributive for BadMulRing {}

impl AbelianGroup for BadMulRing {}

impl From<f64> for BadMulRing {
    fn from(x: f64) -> Self {
        BadMulRing(x)
    }
}

impl From<BadMulRing> for f64 {
    fn from(w: BadMulRing) -> Self {
        w.0 * 2.0
    }
}

#[test]
#[should_panic(expected = "RingIso multiplication homomorphism failed")]
fn ring_iso_laws_panic_on_broken_multiplication() {
    // Use non-zero, non-one values so a*b != a+b and the doubling break
    // surfaces on the multiplication assertion. The doubled-From preserves
    // addition (T::from(a+b) == 2*(a+b) == 2*a + 2*b == T::from(a) + T::from(b))
    // but breaks multiplication.
    assert_ring_iso_from_laws::<BadMulRing, f64>(BadMulRing(3.0), BadMulRing(5.0));
}
