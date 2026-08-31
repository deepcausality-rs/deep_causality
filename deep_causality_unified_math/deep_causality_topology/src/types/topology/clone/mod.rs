/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Clone implementations for Topology.

use crate::Topology;

impl<R: Clone, G: Clone> Topology<R, G> {
    pub fn clone_shallow(&self) -> Self {
        self.clone()
    }
}
