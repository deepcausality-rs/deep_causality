/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::ops::{Add, Div, Mul, Rem, Sub};

use crate::complex::octonion_number::{Octonion, OctonionNumber};
use crate::float::Float;

// Add
impl<F: Float> Add for Octonion<F> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            s: self.s + rhs.s,
            e1: self.e1 + rhs.e1,
            e2: self.e2 + rhs.e2,
            e3: self.e3 + rhs.e3,
            e4: self.e4 + rhs.e4,
            e5: self.e5 + rhs.e5,
            e6: self.e6 + rhs.e6,
            e7: self.e7 + rhs.e7,
        }
    }
}

// Sub
impl<F: Float> Sub for Octonion<F> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            s: self.s - rhs.s,
            e1: self.e1 - rhs.e1,
            e2: self.e2 - rhs.e2,
            e3: self.e3 - rhs.e3,
            e4: self.e4 - rhs.e4,
            e5: self.e5 - rhs.e5,
            e6: self.e6 - rhs.e6,
            e7: self.e7 - rhs.e7,
        }
    }
}

// Mul (Octonion Product)
impl<F: Float> Mul for Octonion<F> {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        let s = self.s * rhs.s
            - (self.e1 * rhs.e1
                + self.e2 * rhs.e2
                + self.e3 * rhs.e3
                + self.e4 * rhs.e4
                + self.e5 * rhs.e5
                + self.e6 * rhs.e6
                + self.e7 * rhs.e7);
        let e1 = self.s * rhs.e1 + rhs.s * self.e1 + self.e2 * rhs.e4
            - self.e4 * rhs.e2
            - self.e3 * rhs.e7
            + self.e7 * rhs.e3
            + self.e5 * rhs.e6
            - self.e6 * rhs.e5;
        let e2 = self.s * rhs.e2 + rhs.s * self.e2 - self.e1 * rhs.e4
            + self.e4 * rhs.e1
            + self.e3 * rhs.e5
            - self.e5 * rhs.e3
            - self.e6 * rhs.e7
            + self.e7 * rhs.e6;
        let e3 = self.s * rhs.e3 + rhs.s * self.e3 + self.e1 * rhs.e7
            - self.e7 * rhs.e1
            - self.e2 * rhs.e5
            + self.e5 * rhs.e2
            + self.e4 * rhs.e6
            - self.e6 * rhs.e4;
        let e4 = self.s * rhs.e4 + rhs.s * self.e4 + self.e1 * rhs.e2
            - self.e2 * rhs.e1
            - self.e3 * rhs.e6
            + self.e6 * rhs.e3
            - self.e5 * rhs.e7
            + self.e7 * rhs.e5;
        let e5 = self.s * rhs.e5 + rhs.s * self.e5 - self.e1 * rhs.e6
            + self.e6 * rhs.e1
            + self.e2 * rhs.e3
            - self.e3 * rhs.e2
            + self.e4 * rhs.e7
            - self.e7 * rhs.e4;
        let e6 = self.s * rhs.e6 + rhs.s * self.e6 + self.e1 * rhs.e5
            - self.e5 * rhs.e1
            - self.e2 * rhs.e7
            + self.e7 * rhs.e2
            - self.e3 * rhs.e4
            + self.e4 * rhs.e3;
        let e7 = self.s * rhs.e7 + rhs.s * self.e7 - self.e1 * rhs.e3
            + self.e3 * rhs.e1
            + self.e2 * rhs.e6
            - self.e6 * rhs.e2
            - self.e4 * rhs.e5
            + self.e5 * rhs.e4;
        Self::new(s, e1, e2, e3, e4, e5, e6, e7)
    }
}

// Mul (Scalar)
impl<F: Float> Mul<F> for Octonion<F> {
    type Output = Self;
    fn mul(self, scalar: F) -> Self {
        Octonion {
            s: self.s * scalar,
            e1: self.e1 * scalar,
            e2: self.e2 * scalar,
            e3: self.e3 * scalar,
            e4: self.e4 * scalar,
            e5: self.e5 * scalar,
            e6: self.e6 * scalar,
            e7: self.e7 * scalar,
        }
    }
}

// Div
#[allow(clippy::suspicious_arithmetic_impl)]
impl<F: Float> Div for Octonion<F> {
    type Output = Self;
    fn div(self, other: Self) -> Self {
        self * other.inverse()
    }
}

// Div (Scalar)
impl<F: Float> Div<F> for Octonion<F> {
    type Output = Self;
    fn div(self, scalar: F) -> Self {
        let inv_scalar = F::one() / scalar;
        self * inv_scalar
    }
}

// Rem
impl<F: Float> Rem for Octonion<F> {
    type Output = Self;
    fn rem(self, _other: Self) -> Self {
        self // Placeholder
    }
}
