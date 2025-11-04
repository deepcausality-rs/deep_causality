use std::ops::Neg;

use crate::float::Float;
use crate::quaternion::Quaternion;

// Neg
impl<F: Float> Neg for Quaternion<F> {
    type Output = Self;

    fn neg(self) -> Self {
        Quaternion {
            w: -self.w,
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}
