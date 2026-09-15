/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::UncertainBool;
use deep_causality_rand::RandScalar;

impl<R: RandScalar> PartialEq for UncertainBool<R> {
    fn eq(&self, other: &Self) -> bool {
        // The identity of an `UncertainBool` is generated, so two carriers built from the same
        // graph shape have different ids. The comparison is on the graph only.
        self.root_node() == other.root_node()
    }
}
