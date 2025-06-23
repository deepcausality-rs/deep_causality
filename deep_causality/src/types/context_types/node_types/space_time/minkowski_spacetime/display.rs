use std::fmt;
use crate::prelude::MinkowskiSpacetime;

impl fmt::Display for MinkowskiSpacetime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MinkowskiSpacetime(id={}, t={:.6}s, x={:.3}, y={:.3}, z={:.3})",
            self.id, self.t, self.x, self.y, self.z
        )
    }
}
