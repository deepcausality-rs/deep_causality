use std::ops::Neg;

use crate::float::Float;
use crate::quaternion::Quaternion;

// Neg
impl<F: Float> Neg for Quaternion<F> {
    type Output = Self;

    /// Returns the negation of the quaternion.
    ///
    /// For a quaternion `q = w + xi + yj + zk`, its negation is `-w - xi - yj - zk`.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::Quaternion;
    ///
    /// let q = Quaternion::new(1.0, -2.0, 3.0, -4.0);
    /// let neg_q = -q;
    /// assert_eq!(neg_q, Quaternion::new(-1.0, 2.0, -3.0, 4.0));
    /// ```
    fn neg(self) -> Self {
        Quaternion {
            w: -self.w,
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}
