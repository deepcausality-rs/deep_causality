/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::MetricFamily;
use core::fmt;

impl<R: fmt::Display> fmt::Display for MetricFamily<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetricFamily::Flat => write!(f, "Flat"),
            MetricFamily::Schwarzschild { mass } => write!(f, "Schwarzschild(mass={})", mass),
            MetricFamily::Kerr { mass, spin } => {
                write!(f, "Kerr(mass={}, spin={})", mass, spin)
            }
            MetricFamily::Flrw {
                scale_factor,
                curvature_k,
            } => write!(f, "FLRW(a={}, k={})", scale_factor, curvature_k),
        }
    }
}
