use crate::float::Float;
use crate::quaternion::Quaternion;

impl<F> Quaternion<F>
where
    F: Float,
{
    /// Returns the conjugate of the quaternion.
    ///
    /// For a quaternion `q = w + xi + yj + zk`, its conjugate is `w - xi - yj - zk`.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::Quaternion;
    ///
    /// let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    /// let conj_q : Quaternion<f64>  = q.conjugate();
    /// assert_eq!(conj_q, Quaternion::new(1.0, -2.0, -3.0, -4.0));
    /// ```
    pub fn conjugate(&self) -> Self {
        Quaternion {
            w: self.w,
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }

    /// Computes the squared norm (magnitude squared) of the quaternion.
    ///
    /// For a quaternion `q = w + xi + yj + zk`, the squared norm is `w^2 + x^2 + y^2 + z^2`.
    /// This method avoids the square root operation, making it more efficient
    /// when only relative magnitudes are needed.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::Quaternion;
    ///
    /// let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    /// assert_eq!(q.norm_sqr(), 1.0*1.0 + 2.0*2.0 + 3.0*3.0 + 4.0*4.0);
    /// ```
    pub fn norm_sqr(&self) -> F {
        self.w * self.w + self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// Computes the norm (magnitude or absolute value) of the quaternion.
    ///
    /// For a quaternion `q = w + xi + yj + zk`, the norm is `sqrt(w^2 + x^2 + y^2 + z^2)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::Quaternion;
    ///
    /// let q = Quaternion::<f64>::new(1.0, 2.0, 3.0, 4.0);
    /// assert_eq!(q.norm(), (1.0f64*1.0f64 + 2.0f64*2.0f64 + 3.0f64*3.0f64 + 4.0f64*4.0f64).sqrt());
    /// ```
    pub fn norm(&self) -> F {
        self.norm_sqr().sqrt()
    }

    /// Returns a normalized quaternion (unit quaternion).
    ///
    /// A unit quaternion has a norm of 1. If the quaternion is a zero quaternion,
    /// it returns itself to avoid division by zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::Quaternion;
    ///
    /// let q = Quaternion::<f64>::new(1.0, 2.0, 3.0, 4.0);
    /// let normalized_q = q.normalize();
    /// assert!((normalized_q.norm() - 1.0).abs() < 1e-9);
    ///
    /// let zero_q = Quaternion::<f64>::new(0.0, 0.0, 0.0, 0.0);
    /// assert_eq!(zero_q.normalize(), zero_q);
    /// ```
    pub fn normalize(&self) -> Self {
        let n = self.norm();
        if n.is_zero() { *self } else { *self / n }
    }

    /// Returns the inverse of the quaternion.
    ///
    /// For a non-zero quaternion `q`, its inverse `q^-1` is `q.conjugate() / q.norm_sqr()`.
    /// If the quaternion is a zero quaternion, it returns a quaternion with `NaN` components.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::Quaternion;
    ///
    /// let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    /// let inv_q = q.inverse();
    /// // For a unit quaternion, inverse is its conjugate.
    /// // For a general quaternion, q * q.inverse() should be identity.
    /// let identity_q : Quaternion<f64>  = q * inv_q;
    /// assert!((identity_q.w - 1.0).abs() < 1e-9);
    /// assert!((identity_q.x - 0.0).abs() < 1e-9);
    /// assert!((identity_q.y - 0.0).abs() < 1e-9);
    /// assert!((identity_q.z - 0.0).abs() < 1e-9);
    ///
    /// let zero_q = Quaternion::<f64>::new(0.0, 0.0, 0.0, 0.0);
    /// let inv_zero_q = zero_q.inverse();
    /// assert!(inv_zero_q.w.is_nan());
    /// ```
    pub fn inverse(&self) -> Self {
        let n_sqr = self.norm_sqr();
        if n_sqr.is_zero() {
            Quaternion::new(F::nan(), F::nan(), F::nan(), F::nan())
        } else {
            self.conjugate() / n_sqr
        }
    }

    /// Computes the dot product with another quaternion.
    ///
    /// For two quaternions `q1 = w1 + x1i + y1j + z1k` and `q2 = w2 + x2i + y2j + z2k`,
    /// their dot product is `w1*w2 + x1*x2 + y1*y2 + z1*z2`.
    ///
    /// # Arguments
    ///
    /// * `other` - The other quaternion.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::Quaternion;
    ///
    /// let q1 = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    /// let q2 = Quaternion::new(5.0, 6.0, 7.0, 8.0);
    /// assert_eq!(q1.dot(&q2), 1.0*5.0 + 2.0*6.0 + 3.0*7.0 + 4.0*8.0);
    /// ```
    pub fn dot(&self, other: &Self) -> F {
        self.w * other.w + self.x * other.x + self.y * other.y + self.z * other.z
    }
}
