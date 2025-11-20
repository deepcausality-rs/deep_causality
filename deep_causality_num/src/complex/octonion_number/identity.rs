/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::complex::octonion_number::Octonion;
use crate::float::Float;
use crate::identity::one::{ConstOne, One};
use crate::identity::zero::{ConstZero, Zero};

// Zero
impl<F: Float> Zero for Octonion<F> {
    fn zero() -> Self {
        Octonion::new(
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
        )
    }

    fn is_zero(&self) -> bool {
        self.s.is_zero()
            && self.e1.is_zero()
            && self.e2.is_zero()
            && self.e3.is_zero()
            && self.e4.is_zero()
            && self.e5.is_zero()
            && self.e6.is_zero()
            && self.e7.is_zero()
    }
}

// ConstZero
impl<F: Float + ConstZero> ConstZero for Octonion<F> {
    const ZERO: Self = Octonion {
        s: F::ZERO,
        e1: F::ZERO,
        e2: F::ZERO,
        e3: F::ZERO,
        e4: F::ZERO,
        e5: F::ZERO,
        e6: F::ZERO,
        e7: F::ZERO,
    };
}

// One
impl<F: Float> One for Octonion<F> {
    fn one() -> Self {
        Octonion::new(
            F::one(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
        )
    }

    fn is_one(&self) -> bool {
        self.s.is_one()
            && self.e1.is_zero()
            && self.e2.is_zero()
            && self.e3.is_zero()
            && self.e4.is_zero()
            && self.e5.is_zero()
            && self.e6.is_zero()
            && self.e7.is_zero()
    }
}

// ConstOne
impl<F: Float + ConstOne + ConstZero> ConstOne for Octonion<F> {
    const ONE: Self = Octonion {
        s: F::ONE,
        e1: F::ZERO,
        e2: F::ZERO,
        e3: F::ZERO,
        e4: F::ZERO,
        e5: F::ZERO,
        e6: F::ZERO,
        e7: F::ZERO,
    };
}
