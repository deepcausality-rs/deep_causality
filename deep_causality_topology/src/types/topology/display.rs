/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::Topology;
use core::fmt::{Display, Formatter};

impl<T> Display for Topology<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "CausalTopology:")?;
        writeln!(f, "  Grade: {}", self.grade)?;
        writeln!(f, "  Cursor: {}", self.cursor)?;
        writeln!(f, "  Data: {}", self.data)?;
        // We probably don't want to print the entire CausalComplex here as it can be very large.
        // Just indicating its presence might be enough, or adding a debug-only print for it.
        // For now, let's omit detailed complex info in Display.
        Ok(())
    }
}
