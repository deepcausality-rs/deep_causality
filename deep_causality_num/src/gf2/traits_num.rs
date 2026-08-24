/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::gf2::Gf2;
use crate::identity::one::{ConstOne, One};
use crate::identity::zero::{ConstZero, Zero};

impl Zero for Gf2 {
    #[inline]
    fn zero() -> Self {
        Gf2::ZERO
    }

    #[inline]
    fn is_zero(&self) -> bool {
        !self.0
    }
}

impl ConstZero for Gf2 {
    const ZERO: Self = Gf2::ZERO;
}

impl One for Gf2 {
    #[inline]
    fn one() -> Self {
        Gf2::ONE
    }

    #[inline]
    fn is_one(&self) -> bool {
        self.0
    }
}

impl ConstOne for Gf2 {
    const ONE: Self = Gf2::ONE;
}

/// Reads a bit as an element of 𝔽₂.
impl From<bool> for Gf2 {
    #[inline]
    fn from(bit: bool) -> Self {
        Gf2::new(bit)
    }
}

/// Reads an element of 𝔽₂ as a bit.
impl From<Gf2> for bool {
    #[inline]
    fn from(value: Gf2) -> Self {
        value.bit()
    }
}
