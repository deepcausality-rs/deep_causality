/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `deep_causality_num::iso::GroupIso<T>` and the helper
//! `assert_group_iso_from_law`.
//!
//! Strategy: the test crate cannot impl foreign traits on foreign types, so
//! we use a local newtype `IdWrap<i32>` that satisfies `AddGroup` (hence
//! `Group`) and provides bidirectional `From` to/from `i32`. The "iso" is
//! the identity wrap/unwrap, which trivially preserves addition.
//!
//! Coverage:
//! - Success: identity iso passes the homomorphism check.
//! - Trait impl: `impl GroupIso<i32> for IdWrap<i32> {}` compiles, verifying
//!   the trait's where-clauses are satisfiable.
//! - Panic: a separate `BadAddWrap` newtype with a broken `From<BadAddWrap>
//!   for i32` (adds +1) triggers the homomorphism assertion's panic branch.

use core::ops::{Add, AddAssign, Sub, SubAssign};
use deep_causality_num::Zero;
use deep_causality_num::iso::GroupIso;
use deep_causality_num::iso::test_support::assert_group_iso_from_law;

#[derive(Clone, Copy, PartialEq, Debug)]
struct IdWrap(i32);

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
        IdWrap(0)
    }
    fn is_zero(&self) -> bool {
        self.0 == 0
    }
    fn set_zero(&mut self) {
        self.0 = 0;
    }
}

impl From<i32> for IdWrap {
    fn from(x: i32) -> Self {
        IdWrap(x)
    }
}

impl From<IdWrap> for i32 {
    fn from(w: IdWrap) -> Self {
        w.0
    }
}

// The marker impl. This compiles only if `GroupIso<i32>` is well-formed and
// the where-clause bounds are satisfied by `IdWrap` and `i32`.
impl GroupIso<i32> for IdWrap {}

#[test]
fn group_iso_law_holds_for_identity_iso() {
    assert_group_iso_from_law::<IdWrap, i32>(IdWrap(3), IdWrap(5));
    assert_group_iso_from_law::<IdWrap, i32>(IdWrap(0), IdWrap(0));
    assert_group_iso_from_law::<IdWrap, i32>(IdWrap(-1), IdWrap(1));
    assert_group_iso_from_law::<IdWrap, i32>(IdWrap(100), IdWrap(-50));
}

#[test]
fn group_iso_law_holds_in_reverse_direction() {
    assert_group_iso_from_law::<i32, IdWrap>(3, 5);
    assert_group_iso_from_law::<i32, IdWrap>(-7, 11);
}

// A wrapper that breaks the group homomorphism: From<BadAddWrap> for i32
// adds 1, so i32::from(a + b) = (a + b) + 1 differs from
// i32::from(a) + i32::from(b) = (a + 1) + (b + 1) = a + b + 2.
#[derive(Clone, Copy, PartialEq, Debug)]
struct BadAddWrap(i32);

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
        BadAddWrap(0)
    }
    fn is_zero(&self) -> bool {
        self.0 == 0
    }
    fn set_zero(&mut self) {
        self.0 = 0;
    }
}

impl From<i32> for BadAddWrap {
    fn from(x: i32) -> Self {
        BadAddWrap(x)
    }
}

impl From<BadAddWrap> for i32 {
    fn from(w: BadAddWrap) -> Self {
        w.0 + 1
    }
}

#[test]
#[should_panic(expected = "GroupIso homomorphism failed")]
fn group_iso_law_panics_on_broken_homomorphism() {
    // From<BadAddWrap> for i32 adds 1, so the homomorphism breaks.
    assert_group_iso_from_law::<BadAddWrap, i32>(BadAddWrap(3), BadAddWrap(5));
}
