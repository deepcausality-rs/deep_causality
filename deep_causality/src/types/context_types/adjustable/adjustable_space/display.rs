// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use std::fmt::{Display, Formatter};

use super::*;

impl<T> Display for AdjustableSpace<T>
    where T: Copy + Default + Display,
{
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "AdjustableSpace<{}> {{ id: {}, x: {}, y: {}, z: {} }}",
               T::default(), self.id, self.x, self.y, self.z)
    }
}
