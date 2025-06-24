// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use crate::prelude::EuclideanSpace;
use std::fmt::{Display, Formatter};

impl Display for EuclideanSpace {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "Spaceoid: id={:?}, coordinates (x,y,x)={:?}",
            self.id, self.coords
        )
    }
}
