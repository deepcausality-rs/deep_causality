/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Complex, ConstOne, ConstZero, Float, One, Zero};

// Implement Zero trait
impl<F> Zero for Complex<F>
where
    F: Float,
{
    #[inline]
    fn zero() -> Self {
        Self::new(F::zero(), F::zero())
    }

    #[inline]
    fn is_zero(&self) -> bool {
        self.re.is_zero() && self.im.is_zero()
    }
}

impl<F> ConstZero for Complex<F>
where
    F: Float + ConstZero,
{
    const ZERO: Self = Self {
        re: F::ZERO,
        im: F::ZERO,
    };
}

// Implement One trait
impl<F> One for Complex<F>
where
    F: Float + One,
{
    #[inline]
    fn one() -> Self {
        Self::new(F::one(), F::zero())
    }

    #[inline]
    fn is_one(&self) -> bool {
        self.re.is_one() && self.im.is_zero()
    }
}

impl<F> ConstOne for Complex<F>
where
    F: Float + ConstOne + ConstZero,
{
    const ONE: Self = Self {
        re: F::ONE,
        im: F::ZERO,
    };
}
