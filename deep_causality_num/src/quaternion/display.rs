use std::fmt::Display;

use crate::float::Float;
use crate::quaternion::Quaternion;

// Display
impl<F: Float + Display> Display for Quaternion<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.w)?;
        write!(
            f,
            " {} {}i",
            if self.x < F::zero() { "-" } else { "+" },
            self.x.abs()
        )?;
        write!(
            f,
            " {} {}j",
            if self.y < F::zero() { "-" } else { "+" },
            self.y.abs()
        )?;
        write!(
            f,
            " {} {}k",
            if self.z < F::zero() { "-" } else { "+" },
            self.z.abs()
        )
    }
}
