/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use super::{Rational, RationalScalar};
use crate::{One, Zero};

impl<T: RationalScalar> Zero for Rational<T> {
    /// The additive identity, `0/1`.
    #[inline]
    fn zero() -> Self {
        Self::from_integer(T::zero())
    }

    /// A rational is zero exactly when its numerator is, since the denominator is never zero.
    #[inline]
    fn is_zero(&self) -> bool {
        self.numer().is_zero()
    }
}

impl<T: RationalScalar> One for Rational<T> {
    /// The multiplicative identity, `1/1`.
    #[inline]
    fn one() -> Self {
        Self::from_integer(T::one())
    }

    /// A rational is one exactly when it is `1/1`. Canonical form makes this a direct check:
    /// no other stored pair represents the value one.
    #[inline]
    fn is_one(&self) -> bool {
        *self.numer() == T::one() && *self.denom() == T::one()
    }
}

impl<T: RationalScalar> Default for Rational<T> {
    /// Zero, matching the additive identity.
    #[inline]
    fn default() -> Self {
        Self::zero()
    }
}
