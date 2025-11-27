/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use core::fmt::{Display, Formatter};
use crate::{Skeleton};

impl Display for Skeleton {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Skeleton(Dim: {}, Num Simplices: {})",
            self.dim,
            self.simplices.len()
        )
    }
}