/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::ops::{AddAssign, DivAssign, MulAssign, RemAssign, SubAssign};

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
impl<F: Float> MulAssign for Octonion<F> {
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other;
    }
}

// DivAssign
impl<F: Float> DivAssign for Octonion<F> {
    fn div_assign(&mut self, other: Self) {
        *self = *self / other;
    }
}

// RemAssign
impl<F: Float> RemAssign for Octonion<F> {
    fn rem_assign(&mut self, _other: Self) {
        // Remainder for octonions is not a standard mathematical operation.
        // Current implementation does nothing.
    }
}

// MulAssign (scalar)
impl<F: Float + MulAssign> MulAssign<F> for Octonion<F> {
    fn mul_assign(&mut self, scalar: F) {
        self.s *= scalar;
        self.e1 *= scalar;
        self.e2 *= scalar;
        self.e3 *= scalar;
        self.e4 *= scalar;
        self.e5 *= scalar;
        self.e6 *= scalar;
        self.e7 *= scalar;
    }
}

// DivAssign (scalar)
impl<F: Float + DivAssign> DivAssign<F> for Octonion<F> {
    fn div_assign(&mut self, scalar: F) {
        self.s /= scalar;
        self.e1 /= scalar;
        self.e2 /= scalar;
        self.e3 /= scalar;
        self.e4 /= scalar;
        self.e5 /= scalar;
        self.e6 /= scalar;
        self.e7 /= scalar;
    }
}

// RemAssign (scalar)
impl<F: Float> RemAssign<F> for Octonion<F> {
    fn rem_assign(&mut self, _scalar: F) {
        // Remainder for octonions is not a standard mathematical operation.
        // Current implementation does nothing.
    }
}
