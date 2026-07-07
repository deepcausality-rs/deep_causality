/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `deep_causality_algebra::iso::GroupIso<T>` and the helper
//! `assert_group_iso_from_law`.
//!
//! Strategy: the test crate cannot impl foreign traits on foreign types, so
//! we use a local newtype `IdWrap<f64>` that satisfies `AddGroup` (hence
//! `Group`) and provides bidirectional `From` to/from `f64`. The "iso" is
//! the identity wrap/unwrap, which trivially preserves addition.
//!
//! Coverage:
//! - Success: identity iso passes the homomorphism check.
//! - Trait impl: `impl GroupIso<f64> for IdWrap<f64> {}` compiles, verifying
//!   the trait's where-clauses are satisfiable.
//! - Panic: a separate `BadAddWrap` newtype with a broken `From<BadAddWrap>
//!   for f64` (adds +1.0) triggers the homomorphism assertion's panic branch.

use core::ops::{Add, AddAssign, Sub, SubAssign};
use deep_causality_algebra::iso::GroupIso;
use deep_causality_algebra::iso::test_support::assert_group_iso_from_law;
use deep_causality_num::Zero;

#[derive(Clone, Copy, PartialEq, Debug)]
struct IdWrap(f64);

impl Add for IdWrap {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        IdWrap(self.0 + rhs.0)
    }
}

impl AddAssign for IdWrap {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Sub for IdWrap {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        IdWrap(self.0 - rhs.0)
    }
}

impl SubAssign for IdWrap {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl Zero for IdWrap {
    fn zero() -> Self {
        IdWrap(0.0)
    }
    fn is_zero(&self) -> bool {
        self.0 == 0.0
    }
    fn set_zero(&mut self) {
        self.0 = 0.0;
    }
}

impl From<f64> for IdWrap {
    fn from(x: f64) -> Self {
        IdWrap(x)
    }
}

impl From<IdWrap> for f64 {
    fn from(w: IdWrap) -> Self {
        w.0
    }
}

// The marker impl. This compiles only if `GroupIso<f64>` is well-formed and
// the where-clause bounds are satisfied by `IdWrap` and `f64`.
impl GroupIso<f64> for IdWrap {}

#[test]
fn group_iso_law_holds_for_identity_iso() {
    assert_group_iso_from_law::<IdWrap, f64>(IdWrap(3.0), IdWrap(5.0));
    assert_group_iso_from_law::<IdWrap, f64>(IdWrap(0.0), IdWrap(0.0));
    assert_group_iso_from_law::<IdWrap, f64>(IdWrap(-1.0), IdWrap(1.0));
    assert_group_iso_from_law::<IdWrap, f64>(IdWrap(100.0), IdWrap(-50.0));
}

#[test]
fn group_iso_law_holds_in_reverse_direction() {
    assert_group_iso_from_law::<f64, IdWrap>(3.0, 5.0);
    assert_group_iso_from_law::<f64, IdWrap>(-7.0, 11.0);
}

// A wrapper that breaks the group homomorphism: From<BadAddWrap> for f64
// adds 1.0, so f64::from(a + b) = (a + b) + 1.0 differs from
// f64::from(a) + f64::from(b) = (a + 1.0) + (b + 1.0) = a + b + 2.0.
#[derive(Clone, Copy, PartialEq, Debug)]
struct BadAddWrap(f64);

impl Add for BadAddWrap {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        BadAddWrap(self.0 + rhs.0)
    }
}

impl AddAssign for BadAddWrap {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Sub for BadAddWrap {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        BadAddWrap(self.0 - rhs.0)
    }
}

impl SubAssign for BadAddWrap {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl Zero for BadAddWrap {
    fn zero() -> Self {
        BadAddWrap(0.0)
    }
    fn is_zero(&self) -> bool {
        self.0 == 0.0
    }
    fn set_zero(&mut self) {
        self.0 = 0.0;
    }
}

impl From<f64> for BadAddWrap {
    fn from(x: f64) -> Self {
        BadAddWrap(x)
    }
}

impl From<BadAddWrap> for f64 {
    fn from(w: BadAddWrap) -> Self {
        w.0 + 1.0
    }
}

#[test]
#[should_panic(expected = "GroupIso homomorphism failed")]
fn group_iso_law_panics_on_broken_homomorphism() {
    // From<BadAddWrap> for f64 adds 1.0, so the homomorphism breaks.
    assert_group_iso_from_law::<BadAddWrap, f64>(BadAddWrap(3.0), BadAddWrap(5.0));
}
