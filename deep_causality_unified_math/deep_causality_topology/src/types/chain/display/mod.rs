/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Chain;
use std::fmt::{Debug, Display, Formatter};

impl<R, G> Display for Chain<R, G>
where
    G: Debug + Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "Chain:")?;
        writeln!(f, "  Grade: {}", self.grade)?;
        writeln!(f, "  Weights: {:?}", self.weights)?; // Using Debug for CsrMatrix
        Ok(())
    }
}
