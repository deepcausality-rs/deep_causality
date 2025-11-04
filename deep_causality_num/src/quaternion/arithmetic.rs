use std::iter::{Product, Sum};
use std::ops::{Add, Div, Mul, Rem, Sub};

use crate::float::Float;
use crate::identity::one::One;
use crate::identity::zero::Zero;
use crate::quaternion::Quaternion;

// Add
impl<F: Float> Add for Quaternion<F> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Quaternion {
            w: self.w + other.w,
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

// Sub
impl<F: Float> Sub for Quaternion<F> {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Quaternion {
            w: self.w - other.w,
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

// Mul (Quaternion multiplication)
impl<F: Float> Mul for Quaternion<F> {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Quaternion {
            w: self.w * other.w - self.x * other.x - self.y * other.y - self.z * other.z,
            x: self.w * other.x + self.x * other.w + self.y * other.z - self.z * other.y,
            y: self.w * other.y - self.x * other.z + self.y * other.w + self.z * other.x,
            z: self.w * other.z + self.x * other.y - self.y * other.x + self.z * other.w,
        }
    }
}

// Mul (Scalar multiplication)
impl<F: Float> Mul<F> for Quaternion<F> {
    type Output = Self;

    fn mul(self, scalar: F) -> Self {
        Quaternion {
            w: self.w * scalar,
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
}

// Div (Quaternion division)
#[allow(clippy::suspicious_arithmetic_impl)]
impl<F: Float> Div for Quaternion<F> {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        self * other.inverse()
    }
}

// Div (Scalar division)
impl<F: Float> Div<F> for Quaternion<F> {
    type Output = Self;

    fn div(self, scalar: F) -> Self {
        if scalar.is_zero() {
            Quaternion::new(F::nan(), F::nan(), F::nan(), F::nan())
        } else {
            Quaternion {
                w: self.w / scalar,
                x: self.x / scalar,
                y: self.y / scalar,
                z: self.z / scalar,
            }
        }
    }
}

// Rem (Placeholder for now, as quaternion remainder is not standard)
impl<F: Float> Rem for Quaternion<F> {
    type Output = Self;

    fn rem(self, _other: Self) -> Self {
        // Quaternion remainder is not a standard operation.
        // Returning self for now, or could panic/return an error.
        // This might need further clarification in the spec.
        self
    }
}

// Sum
impl<F: Float> Sum for Quaternion<F> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Quaternion::zero(), |acc, x| acc + x)
    }
}

// Product
impl<F: Float> Product for Quaternion<F> {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Quaternion::one(), |acc, x| acc * x)
    }
}
