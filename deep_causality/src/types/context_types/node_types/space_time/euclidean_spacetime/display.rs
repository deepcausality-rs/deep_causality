// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::fmt;
use crate::prelude::EuclideanSpacetime;

impl fmt::Display for EuclideanSpacetime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "EuclideanSpacetime(id={}, x={:.3}, y={:.3}, z={:.3}, t={} {:?})",
            self.id, self.coords[0], self.coords[1], self.coords[2], self.t, self.time_scale
        )
    }
}
