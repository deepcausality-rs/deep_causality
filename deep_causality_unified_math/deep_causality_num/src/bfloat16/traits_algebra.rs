/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Identity and numeric trait implementations for `BFloat16`.
//!
//! Only the traits below `Float` in the tower are implemented here: `Zero`, `One` and `Num`.
//! `Real`, `RealField`, `DivisionAlgebra` and the rest come from the blanket implementations
//! over `Float` in `deep_causality_algebra`, once that crate carries the law markers for the
//! type, which it does beside those for `f32`, `f64` and `Float106`.

use crate::BFloat16;
use crate::{Num, One, Zero};

impl Zero for BFloat16 {
    #[inline]
    fn zero() -> Self {
        Self::ZERO
    }

    /// `true` for both zeros; a bit-pattern comparison would miss `-0.0`.
    #[inline]
    fn is_zero(&self) -> bool {
        self.to_f32() == 0.0
    }
}

impl One for BFloat16 {
    #[inline]
    fn one() -> Self {
        Self::ONE
    }

    #[inline]
    fn is_one(&self) -> bool {
        self.to_f32() == 1.0
    }
}

impl Num for BFloat16 {}
