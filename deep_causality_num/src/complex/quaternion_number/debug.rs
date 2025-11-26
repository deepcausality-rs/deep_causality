use crate::{Float, Quaternion};
use core::fmt::Debug;

// Debug
impl<F: Float + Debug> Debug for Quaternion<F> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Quaternion")
            .field("w", &self.w)
            .field("x", &self.x)
            .field("y", &self.y)
            .field("z", &self.z)
            .finish()
    }
}
