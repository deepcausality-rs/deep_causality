/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::CausalTensor;

impl<T: std::fmt::Display> std::fmt::Display for CausalTensor<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CausalTensor {{ data: [")?;
        for (i, item) in self.data.iter().enumerate() {
            write!(f, "{}", item)?;
            if i < self.data.len() - 1 {
                write!(f, ", ")?;
            }
        }
        write!(
            f,
            "], shape: {:?}, strides: {:?} }}",
            self.shape, self.strides
        )
    }
}
