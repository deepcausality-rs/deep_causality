/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Topology;

impl<T: Clone> Topology<T> {
    pub fn clone_shallow(&self) -> Self {
        self.clone()
    }
}
