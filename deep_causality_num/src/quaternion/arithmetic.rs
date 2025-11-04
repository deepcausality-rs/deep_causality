use std::iter::{Product, Sum};
use std::ops::{Add, Div, Mul, Rem, Sub};

use crate::Quaternion;
use crate::{Float, One, Zero};

// Add
impl<F: Float> Add for Quaternion<F> {
    type Output = Self;

    /// Performs quaternion addition.
    ///
    /// For two quaternions `q1 = w1 + x1i + y1j + z1k` and `q2 = w2 + x2i + y2j + z2k`,
    /// their sum is `(w1+w2) + (x1+x2)i + (y1+y2)j + (z1+z2)k`.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::Quaternion;
    /// use std::ops::Add;
    ///
    /// let q1 = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    /// let q2 = Quaternion::new(5.0, 6.0, 7.0, 8.0);
    /// let sum_q = q1.add(q2);
    /// assert_eq!(sum_q, Quaternion::new(6.0, 8.0, 10.0, 12.0));
    /// ```
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

    /// Performs quaternion subtraction.
    ///
    /// For two quaternions `q1 = w1 + x1i + y1j + z1k` and `q2 = w2 + x2i + y2j + z2k`,
    /// their difference is `(w1-w2) + (x1-x2)i + (y1-y2)j + (z1-z2)k`.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::Quaternion;
    /// use std::ops::Sub;
    ///
    /// let q1 = Quaternion::new(5.0, 6.0, 7.0, 8.0);
    /// let q2 = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    /// let diff_q = q1.sub(q2);
    /// assert_eq!(diff_q, Quaternion::new(4.0, 4.0, 4.0, 4.0));
    /// ```
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

    /// Performs quaternion multiplication (Hamilton product).
    ///
    /// Quaternion multiplication is non-commutative.
    /// For two quaternions `q1 = w1 + x1i + y1j + z1k` and `q2 = w2 + x2i + y2j + z2k`,
    /// their product `q1 * q2` is:
    /// `(w1w2 - x1x2 - y1y2 - z1z2)`
    /// `+ (w1x2 + x1w2 + y1z2 - z1y2)i`
    /// `+ (w1y2 - x1z2 + y1w2 + z1x2)j`
    /// `+ (w1z2 + x1y2 - y1x2 + z1w2)k`
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::Quaternion;
    /// use std::ops::Mul;
    ///
    /// let q1 = Quaternion::new(1.0, 0.0, 0.0, 0.0); // Identity
    /// let q2 = Quaternion::new(0.0, 1.0, 0.0, 0.0); // i
    /// let product_q = q1.mul(q2);
    /// assert_eq!(product_q, Quaternion::new(0.0, 1.0, 0.0, 0.0)); // 1 * i = i
    ///
    /// let q_i = Quaternion::new(0.0, 1.0, 0.0, 0.0);
    /// let q_j = Quaternion::new(0.0, 0.0, 1.0, 0.0);
    /// let q_k = Quaternion::new(0.0, 0.0, 0.0, 1.0);
    ///
    /// assert_eq!(q_i * q_j, q_k); // i * j = k
    /// assert_eq!(q_j * q_i, -q_k); // j * i = -k
    /// ```
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

    /// Performs scalar multiplication on a quaternion.
    ///
    /// Each component of the quaternion is multiplied by the scalar value.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::Quaternion;
    /// use std::ops::Mul;
    ///
    /// let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    /// let scalar = 2.0;
    /// let product_q = q.mul(scalar);
    /// assert_eq!(product_q, Quaternion::new(2.0, 4.0, 6.0, 8.0));
    /// ```
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

    /// Performs quaternion division.
    ///
    /// Division is implemented as `self * other.inverse()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::Quaternion;
    /// use std::ops::Div;
    ///
    /// let q1 = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    /// let q2 = Quaternion::new(0.5, 0.0, 0.0, 0.0);
    /// let div_q = q1.div(q2);
    /// assert_eq!(div_q, Quaternion::new(2.0, 4.0, 6.0, 8.0));
    /// ```
    fn div(self, other: Self) -> Self {
        self * other.inverse()
    }
}

// Div (Scalar division)
impl<F: Float> Div<F> for Quaternion<F> {
    type Output = Self;

    /// Performs scalar division on a quaternion.
    ///
    /// Each component of the quaternion is divided by the scalar value.
    /// If the scalar is zero, a quaternion with `NaN` components is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::Quaternion;
    /// use std::ops::Div;
    ///
    /// let q = Quaternion::new(2.0, 4.0, 6.0, 8.0);
    /// let scalar = 2.0;
    /// let div_q = q.div(scalar);
    /// assert_eq!(div_q, Quaternion::new(1.0, 2.0, 3.0, 4.0));
    ///
    /// let q_inf = Quaternion::<f64>::new(1.0, 1.0, 1.0, 1.0).div(0.0);
    /// assert!(q_inf.w.is_infinite());
    /// ```
    fn div(self, scalar: F) -> Self {
        let inv_scalar = F::one() / scalar;
        Quaternion {
            w: self.w * inv_scalar,
            x: self.x * inv_scalar,
            y: self.y * inv_scalar,
            z: self.z * inv_scalar,
        }
    }}

// Rem (Placeholder for now, as quaternion remainder is not standard)
impl<F: Float> Rem for Quaternion<F> {
    type Output = Self;

    /// Placeholder for quaternion remainder operation. Not supposed to be used.
    ///
    /// Quaternion remainder is not a standard mathematical operation.
    /// This implementation returns only self. The trait is required for the Num trait.
    fn rem(self, _other: Self) -> Self {
        // Quaternion remainder is not a standard operation.
        // Returning self for now, or could panic/return an error.
        // This might need further clarification in the spec.
        self
    }
}

// Sum
impl<F: Float> Sum for Quaternion<F> {
    /// Computes the sum of an iterator of quaternions.
    ///
    /// This method is part of the `Sum` trait, allowing quaternions to be summed
    /// using `Iterator::sum()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::{Quaternion, Zero};
    ///
    /// let quaternions = vec![
    ///     Quaternion::new(1.0, 1.0, 1.0, 1.0),
    ///     Quaternion::new(2.0, 2.0, 2.0, 2.0),
    ///     Quaternion::new(3.0, 3.0, 3.0, 3.0),
    /// ];
    /// let total_sum: Quaternion<f64> = quaternions.into_iter().sum();
    /// assert_eq!(total_sum, Quaternion::new(6.0, 6.0, 6.0, 6.0));
    /// ```
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Quaternion::zero(), |acc, x| acc + x)
    }
}

// Product
impl<F: Float> Product for Quaternion<F> {
    /// Computes the product of an iterator of quaternions.
    ///
    /// This method is part of the `Product` trait, allowing quaternions to be multiplied
    /// using `Iterator::product()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::{Quaternion, One};
    ///
    /// let quaternions = vec![
    ///     Quaternion::new(1.0, 0.0, 0.0, 0.0), // Identity
    ///     Quaternion::new(0.0, 1.0, 0.0, 0.0), // i
    ///     Quaternion::new(0.0, 0.0, 1.0, 0.0), // j
    /// ];
    /// let total_product: Quaternion<f64> = quaternions.into_iter().product();
    /// assert_eq!(total_product, Quaternion::new(0.0, 0.0, 0.0, 1.0)); // 1 * i * j = k
    /// ```
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Quaternion::one(), |acc, x| acc * x)
    }
}
