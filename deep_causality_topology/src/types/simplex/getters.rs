/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Simplex;

impl Simplex {
    pub fn vertices(&self) -> &Vec<usize> {
        &self.vertices
    }
}
