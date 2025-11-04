use std::fmt::Display;

use crate::float::Float;
use crate::quaternion::Quaternion;

// Display
impl<F: Float + Display> Display for Quaternion<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} + {}i + {}j + {}k", self.w, self.x, self.y, self.z)
    }
}
