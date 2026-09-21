/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Temporal, TimeKind, TimeScale};
use deep_causality_algebra::RealField;
use deep_causality_num::{FromPrimitive, lift_count};

impl<R: RealField + FromPrimitive> Temporal for TimeKind<R> {
    type TimeUnit = R;
    fn time_scale(&self) -> TimeScale {
        match self {
            TimeKind::Discrete(t) => t.time_scale(),
            TimeKind::Euclidean(t) => t.time_scale(),
            TimeKind::Entropic(t) => t.time_scale(),
            TimeKind::Lorentzian(t) => t.time_scale(),
            // TimeKind::Symbolic(t) => t.time_scale(),
        }
    }

    fn time_unit(&self) -> R {
        match self {
            TimeKind::Lorentzian(t) => t.time_unit(),
            TimeKind::Euclidean(t) => t.time_unit(),
            TimeKind::Discrete(t) => lift_count(t.time_unit()),
            TimeKind::Entropic(t) => lift_count(t.time_unit()),
            // TimeKind::Symbolic(t) => t.time_unit() as f64,
        }
    }
}
