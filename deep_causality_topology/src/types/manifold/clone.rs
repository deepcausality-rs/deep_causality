/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Manifold;

impl<T> Manifold<T>
where
    T: Clone,
{
    /// Creates a shallow clone of the Manifold.
    pub fn clone_shallow(&self) -> Self {
        Manifold {
            complex: self.complex.clone(),
            data: self.data.clone(),
            cursor: 0,
        }
    }
}
