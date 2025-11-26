/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use core::ops::{AddAssign, DivAssign, MulAssign, RemAssign, SubAssign};

use crate::complex::octonion_number::Octonion;
use crate::float::Float;

/// Implements the addition assignment operator (`+=`) for two `Octonion` numbers.
///
/// Each component of `self` is added to the corresponding component of `other`.
/// `self = self + other`
///
/// # Arguments
/// * `self` - The left-hand side `Octonion` to be modified.
/// * `other` - The right-hand side `Octonion` to add.
///
/// # Examples
/// ```
/// use deep_causality_num::Octonion;
///
/// let mut o1 = Octonion::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0);
/// let o2 = Octonion::new(9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0);
/// o1 += o2;
/// assert_eq!(o1.s, 10.0);
/// assert_eq!(o1.e1, 12.0);
/// ```
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

/// Implements the subtraction assignment operator (`-=`) for two `Octonion` numbers.
///
/// Each component of `other` is subtracted from the corresponding component of `self`.
/// `self = self - other`
///
/// # Arguments
/// * `self` - The left-hand side `Octonion` to be modified.
/// * `other` - The right-hand side `Octonion` to subtract.
///
/// # Examples
/// ```
/// use deep_causality_num::Octonion;
///
/// let mut o1 = Octonion::new(9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0);
/// let o2 = Octonion::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0);
/// o1 -= o2;
/// assert_eq!(o1.s, 8.0);
/// assert_eq!(o1.e1, 8.0);
/// ```
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

/// Implements the multiplication assignment operator (`*=`) for an `Octonion` and another `Octonion`.
///
/// `self = self * other`
///
/// # Arguments
/// * `self` - The left-hand side `Octonion` to be modified.
/// * `other` - The right-hand side `Octonion` to multiply by.
///
/// # Examples
/// ```
/// use deep_causality_num::Octonion;
/// use deep_causality_num::Zero;
///
/// let mut o = Octonion::new(1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0); // 1
/// let e1 = Octonion::new(0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
/// o *= e1; // o becomes e1
/// assert_eq!(o, e1);
/// ```
impl<F: Float> MulAssign for Octonion<F> {
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other;
    }
}

/// Implements the division assignment operator (`/=`) for an `Octonion` and another `Octonion`.
///
/// `self = self / other` (right division)
///
/// # Arguments
/// * `self` - The left-hand side `Octonion` to be modified.
/// * `other` - The right-hand side `Octonion` to divide by.
///
/// # Examples
/// ```
/// use deep_causality_num::Octonion;
/// use deep_causality_num::{One, Zero};
///
/// let mut o = Octonion::one(); // 1
/// let e1 = Octonion::new(0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
/// let neg_e1 = -e1;
/// o /= e1; // o becomes 1 / e1 = -e1
/// assert_eq!(o, neg_e1);
/// ```
impl<F: Float> DivAssign for Octonion<F> {
    fn div_assign(&mut self, other: Self) {
        *self = *self / other;
    }
}

/// Implements the remainder assignment operator (`%=`) for two `Octonion` numbers.
///
/// For octonions, the remainder operation is not a standard mathematical concept.
/// This implementation provides a placeholder behavior, effectively doing nothing.
///
/// # Arguments
/// * `self` - The left-hand side `Octonion` to be modified.
/// * `_other` - The right-hand side `Octonion` (ignored).
///
/// # Examples
/// ```
/// use deep_causality_num::Octonion;
///
/// let mut o1 = Octonion::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0);
/// let o2 = Octonion::new(9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0);
/// o1 %= o2;
/// // Placeholder behavior: o1 remains unchanged.
/// assert_eq!(o1.s, 1.0);
/// ```
impl<F: Float> RemAssign for Octonion<F> {
    fn rem_assign(&mut self, _other: Self) {
        // Remainder for octonions is not a standard mathematical operation.
        // Current implementation does nothing.
    }
}

/// Implements scalar multiplication assignment (`*=`) for an `Octonion` by a scalar of type `F`.
///
/// Each component of `self` is multiplied by the `scalar` value.
/// `self = self * scalar`
///
/// # Arguments
/// * `self` - The `Octonion` to be modified.
/// * `scalar` - The scalar value of type `F`.
///
/// # Examples
/// ```
/// use deep_causality_num::Octonion;
///
/// let mut o = Octonion::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0);
/// let scalar = 2.0;
/// o *= scalar;
/// assert_eq!(o.s, 2.0);
/// assert_eq!(o.e1, 4.0);
/// ```
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

/// Implements scalar division assignment (`/=`) for an `Octonion` by a scalar of type `F`.
///
/// Each component of `self` is divided by the `scalar` value.
/// `self = self / scalar`
///
/// # Arguments
/// * `self` - The `Octonion` to be modified.
/// * `scalar` - The scalar value of type `F`.
///
/// # Examples
/// ```
/// use deep_causality_num::Octonion;
///
/// let mut o = Octonion::new(2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0);
/// let scalar = 2.0;
/// o /= scalar;
/// assert_eq!(o.s, 1.0);
/// assert_eq!(o.e1, 2.0);
/// ```
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

/// Implements the remainder assignment operator (`%=`) for an `Octonion` and a scalar.
///
/// For octonions, the remainder operation is not a standard mathematical concept.
/// This implementation provides a placeholder behavior, effectively doing nothing.
///
/// # Arguments
/// * `self` - The `Octonion` to be modified.
/// * `_scalar` - The scalar value (ignored).
///
/// # Examples
/// ```
/// use deep_causality_num::Octonion;
///
/// let mut o = Octonion::new(10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0);
/// let scalar = 3.0;
/// o %= scalar;
/// // Placeholder behavior: o remains unchanged.
/// assert_eq!(o.s, 10.0);
/// ```
impl<F: Float> RemAssign<F> for Octonion<F> {
    fn rem_assign(&mut self, _scalar: F) {
        // Remainder for octonions is not a standard mathematical operation.
        // Current implementation does nothing.
    }
}
