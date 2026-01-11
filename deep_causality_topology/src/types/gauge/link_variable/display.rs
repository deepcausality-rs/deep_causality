/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{GaugeGroup, LinkVariable};

impl<G: GaugeGroup, T: std::fmt::Display + Clone> std::fmt::Display for LinkVariable<G, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LinkVariable<{}>({})  ", G::name(), G::matrix_dim())
    }
}
