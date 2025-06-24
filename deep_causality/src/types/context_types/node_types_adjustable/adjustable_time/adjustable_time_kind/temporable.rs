// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use crate::prelude::{AdjustableTimeKind, Temporal, TimeScale};

impl Temporal<f64> for AdjustableTimeKind {
    fn time_scale(&self) -> TimeScale {
        match self {
            AdjustableTimeKind::Discrete(t) => t.time_scale(),
            AdjustableTimeKind::Euclidean(t) => t.time_scale(),
            AdjustableTimeKind::Entropic(t) => t.time_scale(),
            AdjustableTimeKind::Lorentzian(t) => t.time_scale(),
            // AdjustableTimeKind::Symbolic(t) => t.time_scale(),
        }
    }

    fn time_unit(&self) -> f64 {
        match self {
            AdjustableTimeKind::Lorentzian(t) => t.time_unit(),
            AdjustableTimeKind::Euclidean(t) => t.time_unit(),
            AdjustableTimeKind::Discrete(t) => t.time_unit() as f64,
            AdjustableTimeKind::Entropic(t) => t.time_unit() as f64,
            // AdjustableTimeKind::Symbolic(t) => t.time_unit() as f64,
        }
    }
}
