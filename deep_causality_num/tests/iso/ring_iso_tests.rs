/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `deep_causality_num::iso::RingIso<T>` and the helper
//! `assert_ring_iso_from_laws`.
//!
//! Uses `IdRingWrap(i32)` as a local newtype that implements `Ring` via its
//! parent traits (`AbelianGroup`, `MulMonoid`, `Distributive`). The identity
//! iso to/from i32 trivially preserves both addition and multiplication.
//!
//! Coverage:
//! - Success: identity iso passes both homomorphism checks.
//! - Trait impl chain: `impl GroupIso<i32>` + `impl RingIso<i32>` for the
//!   newtype, exercising the trait-inheritance machinery.
//! - Panic (addition branch): broken `From` that adds a constant breaks the
//!   additive homomorphism first; helper panics on the `RingIso addition
//!   homomorphism failed` message.
//! - Panic (multiplication branch): broken `From` that scales by a constant
//!   breaks the multiplicative homomorphism while preserving addition;
//!   helper panics on the `RingIso multiplication homomorphism failed` message.

use core::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};
use deep_causality_num::iso::test_support::assert_ring_iso_from_laws;
use deep_causality_num::{
    AbelianGroup, Associative, Commutative, Distributive, GroupIso, One, RingIso, Zero,
};

#[derive(Clone, Copy, PartialEq, Debug)]
struct IdRingWrap(i32);

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
        IdRingWrap(0)
    }
    fn is_zero(&self) -> bool {
        self.0 == 0
    }
    fn set_zero(&mut self) {
        self.0 = 0;
    }
}

impl One for IdRingWrap {
    fn one() -> Self {
        IdRingWrap(1)
    }
    fn is_one(&self) -> bool {
        self.0 == 1
    }
}

impl Associative for IdRingWrap {}
impl Commutative for IdRingWrap {}
impl Distributive for IdRingWrap {}

impl AbelianGroup for IdRingWrap {}

impl From<i32> for IdRingWrap {
    fn from(x: i32) -> Self {
        IdRingWrap(x)
    }
}

impl From<IdRingWrap> for i32 {
    fn from(w: IdRingWrap) -> Self {
        w.0
    }
}

impl GroupIso<i32> for IdRingWrap {}
impl RingIso<i32> for IdRingWrap {}

#[test]
fn ring_iso_laws_hold_for_identity_iso() {
    assert_ring_iso_from_laws::<IdRingWrap, i32>(IdRingWrap(3), IdRingWrap(5));
    assert_ring_iso_from_laws::<IdRingWrap, i32>(IdRingWrap(0), IdRingWrap(0));
    assert_ring_iso_from_laws::<IdRingWrap, i32>(IdRingWrap(-2), IdRingWrap(7));
    assert_ring_iso_from_laws::<IdRingWrap, i32>(IdRingWrap(1), IdRingWrap(1));
}

#[test]
fn ring_iso_laws_hold_in_reverse_direction() {
    assert_ring_iso_from_laws::<i32, IdRingWrap>(3, 5);
    assert_ring_iso_from_laws::<i32, IdRingWrap>(-4, 9);
}

// A wrapper that breaks the additive ring homomorphism. From<BadAddRing> for
// i32 adds 1, so i32::from(a + b) = (a + b) + 1 differs from
// i32::from(a) + i32::from(b) = (a + 1) + (b + 1). The helper panics on the
// addition check first.
#[derive(Clone, Copy, PartialEq, Debug)]
struct BadAddRing(i32);

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
        BadAddRing(0)
    }
    fn is_zero(&self) -> bool {
        self.0 == 0
    }
    fn set_zero(&mut self) {
        self.0 = 0;
    }
}

impl One for BadAddRing {
    fn one() -> Self {
        BadAddRing(1)
    }
    fn is_one(&self) -> bool {
        self.0 == 1
    }
}

impl Associative for BadAddRing {}
impl Commutative for BadAddRing {}
impl Distributive for BadAddRing {}

impl AbelianGroup for BadAddRing {}

impl From<i32> for BadAddRing {
    fn from(x: i32) -> Self {
        BadAddRing(x)
    }
}

impl From<BadAddRing> for i32 {
    fn from(w: BadAddRing) -> Self {
        w.0 + 1
    }
}

#[test]
#[should_panic(expected = "RingIso addition homomorphism failed")]
fn ring_iso_laws_panic_on_broken_addition() {
    assert_ring_iso_from_laws::<BadAddRing, i32>(BadAddRing(3), BadAddRing(5));
}

// A wrapper that preserves addition but breaks multiplication. From<BadMulRing>
// for i32 doubles the value, which preserves addition (doubling distributes
// over +) but breaks multiplication: i32::from(a * b) = 2 * a * b differs from
// i32::from(a) * i32::from(b) = (2 * a) * (2 * b) = 4 * a * b for a * b != 0.
#[derive(Clone, Copy, PartialEq, Debug)]
struct BadMulRing(i32);

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
        BadMulRing(0)
    }
    fn is_zero(&self) -> bool {
        self.0 == 0
    }
    fn set_zero(&mut self) {
        self.0 = 0;
    }
}

impl One for BadMulRing {
    fn one() -> Self {
        BadMulRing(1)
    }
    fn is_one(&self) -> bool {
        self.0 == 1
    }
}

impl Associative for BadMulRing {}
impl Commutative for BadMulRing {}
impl Distributive for BadMulRing {}

impl AbelianGroup for BadMulRing {}

impl From<i32> for BadMulRing {
    fn from(x: i32) -> Self {
        BadMulRing(x)
    }
}

impl From<BadMulRing> for i32 {
    fn from(w: BadMulRing) -> Self {
        w.0 * 2
    }
}

#[test]
#[should_panic(expected = "RingIso multiplication homomorphism failed")]
fn ring_iso_laws_panic_on_broken_multiplication() {
    // Use non-zero, non-one values so a*b != a+b and the doubling break
    // surfaces on the multiplication assertion. The doubled-From preserves
    // addition (T::from(a+b) == 2*(a+b) == 2*a + 2*b == T::from(a) + T::from(b))
    // but breaks multiplication.
    assert_ring_iso_from_laws::<BadMulRing, i32>(BadMulRing(3), BadMulRing(5));
}
