// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use std::fmt::{Debug, Display, Formatter};

use super::*;

impl<T> Display for Space<T>
where
    T: Default + Debug + Add<T, Output = T> + Sub<T, Output = T> + Mul<T, Output = T>,
{
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "Spaceoid: id={:?}, x={:?}, y={:?}, z={:?}",
            self.id, self.x, self.y, self.z
        )
    }
}
