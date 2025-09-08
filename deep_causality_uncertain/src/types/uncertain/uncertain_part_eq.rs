/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::Uncertain;

impl<T> PartialEq for Uncertain<T> {
    fn eq(&self, other: &Self) -> bool {
        // ID is generated so two Uncertain instances with the same root node will have
        // different ID's. Thus the comparison on the root node only
        self.root_node == other.root_node
    }
}
