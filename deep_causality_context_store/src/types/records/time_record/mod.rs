/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::TimeScale;

/// A time node as a store holds it: one variant per `TimeKind` variant.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TimeRecord {
    Euclidean { scale: TimeScale, value: f64 },
    Lorentzian { scale: TimeScale, value: f64 },
    Discrete { scale: TimeScale, tick: u64 },
    Entropic { tick: u64 },
}

impl TimeRecord {
    /// The variant's name, for an error that says what was found.
    pub const fn kind_name(&self) -> &'static str {
        match self {
            Self::Euclidean { .. } => "Euclidean",
            Self::Lorentzian { .. } => "Lorentzian",
            Self::Discrete { .. } => "Discrete",
            Self::Entropic { .. } => "Entropic",
        }
    }
}
