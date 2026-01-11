/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{GaugeField, GaugeGroup};

impl<G: GaugeGroup, T, A, F> std::fmt::Display for GaugeField<G, T, A, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "GaugeField<{}>(metric={:?}, conn={:?}, field_strength={:?})",
            G::name(),
            self.metric,
            self.connection.shape(),
            self.field_strength.shape()
        )
    }
}
