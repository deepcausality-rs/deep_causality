/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{GaugeGroup, LatticeGaugeField};

impl<G: GaugeGroup, const D: usize, T: std::fmt::Debug> std::fmt::Display
    for LatticeGaugeField<G, D, T>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "LatticeGaugeField<{}, {}D, {}>(links={}, β={:?})",
            G::name(),
            D,
            std::any::type_name::<T>().split("::").last().unwrap_or("?"),
            self.num_links(),
            self.beta
        )
    }
}
