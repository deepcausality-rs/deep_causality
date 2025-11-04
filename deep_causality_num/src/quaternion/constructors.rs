use crate::alias::Vector3;
use crate::float::Float;
use crate::quaternion::Quaternion;

impl<F> Quaternion<F>
where
    F: Float,
{
    /// Creates a new quaternion from its scalar and vector components.
    ///
    /// # Arguments
    ///
    /// * `w` - The scalar component.
    /// * `x` - The `i` component of the vector part.
    /// * `y` - The `j` component of the vector part.
    /// * `z` - The `k` component of the vector part.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::Quaternion;
    ///
    /// let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    /// assert_eq!(q.w, 1.0);
    /// assert_eq!(q.x, 2.0);
    /// assert_eq!(q.y, 3.0);
    /// assert_eq!(q.z, 4.0);
    /// ```
    pub fn new(w: F, x: F, y: F, z: F) -> Self {
        Quaternion { w, x, y, z }
    }

    /// Returns the identity quaternion (1 + 0i + 0j + 0k).
    ///
    /// The identity quaternion represents no rotation.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::Quaternion;
    ///
    /// let identity_q = Quaternion::<f64>::identity();
    /// assert_eq!(identity_q.w, 1.0);
    /// assert_eq!(identity_q.x, 0.0);
    /// assert_eq!(identity_q.y, 0.0);
    /// assert_eq!(identity_q.z, 0.0);
    /// ```
    pub fn identity() -> Self {
        Quaternion {
            w: F::one(),
            x: F::zero(),
            y: F::zero(),
            z: F::zero(),
        }
    }

    /// Creates a quaternion from an axis-angle representation.
    ///
    /// The axis vector is expected to be a unit vector. If the axis has zero length,
    /// an identity quaternion is returned.
    ///
    /// # Arguments
    ///
    /// * `axis` - A 3-element array representing the rotation axis.
    /// * `angle` - The rotation angle in radians.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::Quaternion;
    /// use std::f64::consts::FRAC_PI_2;
    ///
    /// // 90 degrees around the X-axis
    /// let q = Quaternion::from_axis_angle([1.0, 0.0, 0.0], FRAC_PI_2);
    /// // Expected values for a 90-degree rotation around X-axis
    /// assert!((q.w - (FRAC_PI_2 / 2.0).cos()).abs() < 1e-9);
    /// assert!((q.x - (FRAC_PI_2 / 2.0).sin()).abs() < 1e-9);
    /// assert!((q.y - 0.0).abs() < 1e-9);
    /// assert!((q.z - 0.0).abs() < 1e-9);
    /// ```
    pub fn from_axis_angle(axis: Vector3<F>, angle: F) -> Self {
        let half_angle = angle / (F::one() + F::one());
        let s = half_angle.sin();
        let c = half_angle.cos();
        let len = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
        if len.is_zero() {
            return Quaternion::identity();
        }
        let inv_len = F::one() / len;
        Quaternion {
            w: c,
            x: axis[0] * inv_len * s,
            y: axis[1] * inv_len * s,
            z: axis[2] * inv_len * s,
        }
    }

    /// Creates a quaternion from Euler angles (roll, pitch, yaw).
    ///
    /// The Euler angles are applied in the order: yaw (Z-axis), pitch (Y-axis), roll (X-axis).
    /// All angles should be in radians.
    ///
    /// # Arguments
    ///
    /// * `roll` - Rotation around the X-axis (in radians).
    /// * `pitch` - Rotation around the Y-axis (in radians).
    /// * `yaw` - Rotation around the Z-axis (in radians).
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::Quaternion;
    /// use std::f64::consts::FRAC_PI_2;
    ///
    /// // 90 degrees around the Y-axis
    /// let q = Quaternion::from_euler_angles(0.0, FRAC_PI_2, 0.0);
    /// // Expected values for a 90-degree rotation around Y-axis
    /// assert!((q.w - (FRAC_PI_2 / 2.0).cos()).abs() < 1e-9);
    /// assert!((q.x - 0.0).abs() < 1e-9);
    /// assert!((q.y - (FRAC_PI_2 / 2.0).sin()).abs() < 1e-9);
    /// assert!((q.z - 0.0).abs() < 1e-9);
    /// ```
    pub fn from_euler_angles(roll: F, pitch: F, yaw: F) -> Self {
        let cy = (yaw * F::from(0.5).unwrap()).cos();
        let sy = (yaw * F::from(0.5).unwrap()).sin();
        let cp = (pitch * F::from(0.5).unwrap()).cos();
        let sp = (pitch * F::from(0.5).unwrap()).sin();
        let cr = (roll * F::from(0.5).unwrap()).cos();
        let sr = (roll * F::from(0.5).unwrap()).sin();

        Quaternion {
            w: cr * cp * cy + sr * sp * sy,
            x: sr * cp * cy - cr * sp * sy,
            y: cr * sp * cy + sr * cp * sy,
            z: cr * cp * sy - sr * sp * cy,
        }
    }
}
