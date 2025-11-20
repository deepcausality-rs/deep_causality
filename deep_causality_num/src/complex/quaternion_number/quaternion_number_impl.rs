use crate::complex::quaternion_number::Quaternion;
use crate::float::Float;
use crate::{Matrix3, QuaternionNumber, Vector3};

impl<F> QuaternionNumber<F> for Quaternion<F>
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
    fn conjugate(&self) -> Self {
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
    fn norm_sqr(&self) -> F {
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
    fn norm(&self) -> F {
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
    fn normalize(&self) -> Self {
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
    fn inverse(&self) -> Self {
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
    fn dot(&self, other: &Self) -> F {
        self.w * other.w + self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Converts the quaternion to an axis-angle representation.
    ///
    /// Returns a tuple containing a 3-element array representing the rotation axis
    /// and the rotation angle in radians.
    ///
    /// If the quaternion is an identity quaternion (or very close to it),
    /// the angle will be 0 and the axis will be an arbitrary unit vector (e.g., `[1.0, 0.0, 0.0]`).
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::Quaternion;
    /// use std::f64::consts::FRAC_PI_2;
    ///
    /// let q = Quaternion::from_axis_angle([1.0, 0.0, 0.0], FRAC_PI_2);
    /// let (axis, angle) = q.to_axis_angle();
    ///
    /// assert!((axis[0] - 1.0).abs() < 1e-9);
    /// assert!((axis[1] - 0.0).abs() < 1e-9);
    /// assert!((axis[2] - 0.0).abs() < 1e-9);
    /// assert!((angle - FRAC_PI_2).abs() < 1e-9);
    /// ```
    fn to_axis_angle(&self) -> (Vector3<F>, F) {
        let two = F::one() + F::one();
        let mut q = *self;

        // Ensure w is non-negative to get angle in [0, PI]
        if q.w < F::zero() {
            q = -q;
        }

        let angle = two * q.w.acos();

        let s = (F::one() - q.w * q.w).sqrt();
        if s < F::epsilon() {
            // Angle is 0 or 2PI, axis is arbitrary (or undefined).
            // For 0 angle, return identity axis. For 2PI, it's also identity.
            ([F::one(), F::zero(), F::zero()], F::zero())
        } else {
            let inv_s = F::one() / s;
            ([q.x * inv_s, q.y * inv_s, q.z * inv_s], angle)
        }
    }

    /// Converts the quaternion to a 3x3 rotation matrix.
    ///
    /// The resulting matrix can be used to rotate 3D vectors.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::Quaternion;
    /// use std::f64::consts::FRAC_PI_2;
    ///
    /// // 90 degrees around the X-axis
    /// let q = Quaternion::from_axis_angle([1.0, 0.0, 0.0], FRAC_PI_2);
    /// let mat = q.to_rotation_matrix();
    ///
    /// // Expected rotation matrix for 90 degrees around X-axis
    /// // [ 1,  0,  0 ]
    /// // [ 0,  0, -1 ]
    /// // [ 0,  1,  0 ]
    /// assert!((mat[0][0] - 1.0).abs() < 1e-9);
    /// assert!((mat[1][1] - 0.0).abs() < 1e-9);
    /// assert!((mat[1][2] - (-1.0)).abs() < 1e-9);
    /// assert!((mat[2][1] - 1.0).abs() < 1e-9);
    /// ```
    fn to_rotation_matrix(&self) -> Matrix3<F> {
        let two = F::one() + F::one();
        let x2 = self.x * two;
        let y2 = self.y * two;
        let z2 = self.z * two;

        let xx = self.x * x2;
        let xy = self.x * y2;
        let xz = self.x * z2;
        let yy = self.y * y2;
        let yz = self.y * z2;
        let zz = self.z * z2;
        let wx = self.w * x2;
        let wy = self.w * y2;
        let wz = self.w * z2;

        [
            [F::one() - (yy + zz), xy - wz, xz + wy],
            [xy + wz, F::one() - (xx + zz), yz - wx],
            [xz - wy, yz + wx, F::one() - (xx + yy)],
        ]
    }

    /// Performs spherical linear interpolation (SLERP) between two quaternions.
    ///
    /// SLERP interpolates along the shortest arc on the unit sphere between two quaternions.
    /// The parameter `t` is typically in the range `[0, 1]`.
    ///
    /// # Arguments
    ///
    /// * `other` - The target quaternion for interpolation.
    /// * `t` - The interpolation parameter, where `t=0` returns `self` and `t=1` returns `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::Quaternion;
    /// use std::f64::consts::FRAC_PI_2;
    ///
    /// let q1 = Quaternion::<f64>::identity();
    /// let q2 = Quaternion::from_axis_angle([1.0, 0.0, 0.0], FRAC_PI_2);
    ///
    /// // Interpolate halfway between q1 and q2
    /// let slerp_q = q1.slerp(&q2, 0.5);
    /// let expected_q = Quaternion::from_axis_angle([1.0, 0.0, 0.0], FRAC_PI_2 / 2.0);
    ///
    /// assert!((slerp_q.w - expected_q.w).abs() < 1e-9);
    /// assert!((slerp_q.x - expected_q.x).abs() < 1e-9);
    /// assert!((slerp_q.y - expected_q.y).abs() < 1e-9);
    /// assert!((slerp_q.z - expected_q.z).abs() < 1e-9);
    /// ```
    fn slerp(&self, other: &Self, t: F) -> Self {
        let q1 = *self;
        let mut q2 = *other;

        let mut dot = q1.dot(&q2);

        // We can negate one of the quaternions to take the short way.
        if dot < F::zero() {
            q2 = -q2;
            dot = -dot;
        }

        // If the quaternions are very close, use linear interpolation to avoid division by zero.
        if dot > F::one() - F::epsilon() {
            return (q1 * (F::one() - t) + q2 * t).normalize();
        }
        // Clamp dot to avoid NaN from acos due to floating point inaccuracies
        dot = dot.clamp(-F::one(), F::one());
        let theta = dot.acos();
        let sin_theta = theta.sin();

        let s1 = ((F::one() - t) * theta).sin() / sin_theta;
        let s2 = (t * theta).sin() / sin_theta;

        (q1 * s1) + (q2 * s2)
    }
}
