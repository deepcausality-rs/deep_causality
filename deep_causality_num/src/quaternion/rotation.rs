/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Float;
use crate::{Matrix3, Quaternion, Vector3};

impl<F> Quaternion<F>
where
    F: Float,
{
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
    pub fn to_axis_angle(&self) -> (Vector3<F>, F) {
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
    pub fn to_rotation_matrix(&self) -> Matrix3<F> {
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
    pub fn slerp(&self, other: &Self, t: F) -> Self {
        let q1 = *self;
        let mut q2 = *other;

        let mut dot = q1.dot(&q2);

        // If the quaternions are very close (dot product near 1 or -1)
        if dot.abs() > F::one() - F::epsilon() {
            // If they are antipodal and t = 0.5, return identity
            if dot < F::zero() && t == F::from(0.5).unwrap() {
                return Quaternion::identity();
            }
            // Otherwise, they are either identical or antipodal but t is not 0.5.
            // In either case, linear interpolation is a good approximation.
            return (q1 * (F::one() - t) + q2 * t).normalize();
        }

        // If the dot product is negative, the quaternions are "opposite"
        // and slerp will take the long way around.
        // We can negate one of the quaternions to take the short way.
        if dot < F::zero() {
            q2 = -q2;
            dot = -dot;
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
