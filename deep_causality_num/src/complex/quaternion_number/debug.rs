use std::fmt::Debug;

use crate::complex::quaternion_number::Quaternion;
use crate::float::Float;

// Debug
impl<F: Float + Debug> Debug for Quaternion<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Quaternion")
            .field("w", &self.w)
            .field("x", &self.x)
            .field("y", &self.y)
            .field("z", &self.z)
            .finish()
    }
}
