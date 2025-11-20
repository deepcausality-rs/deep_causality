/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::ops::{AddAssign, DivAssign, MulAssign, SubAssign};

use crate::complex::octonion_number::Octonion;
use crate::float::Float;

// AddAssign
impl<F: Float + AddAssign> AddAssign for Octonion<F> {
    fn add_assign(&mut self, other: Self) {
        self.s += other.s;
        self.e1 += other.e1;
        self.e2 += other.e2;
        self.e3 += other.e3;
        self.e4 += other.e4;
        self.e5 += other.e5;
        self.e6 += other.e6;
        self.e7 += other.e7;
    }
}

// SubAssign
impl<F: Float + SubAssign> SubAssign for Octonion<F> {
    fn sub_assign(&mut self, other: Self) {
        self.s -= other.s;
        self.e1 -= other.e1;
        self.e2 -= other.e2;
        self.e3 -= other.e3;
        self.e4 -= other.e4;
        self.e5 -= other.e5;
        self.e6 -= other.e6;
        self.e7 -= other.e7;
    }
}

// MulAssign
impl<F: Float + MulAssign> MulAssign for Octonion<F> {
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other;
    }
}

// DivAssign
impl<F: Float + DivAssign> DivAssign for Octonion<F> {
    fn div_assign(&mut self, other: Self) {
        *self = *self / other;
    }
}
