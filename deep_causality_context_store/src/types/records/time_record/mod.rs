/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::TimeScale;

/// A time node as a store holds it: one variant per `TimeKind` variant.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TimeRecord {
    /// The absolute time of classical spacetime.
    Newtonian {
        scale: TimeScale,
        value: f64,
    },
    /// The coordinate time of an inertial frame of Minkowski spacetime.
    Minkowski {
        scale: TimeScale,
        value: f64,
    },
    Discrete {
        scale: TimeScale,
        tick: u64,
    },
    Entropic {
        tick: u64,
    },
}

impl TimeRecord {
    /// The variant's name, for an error that says what was found.
    pub const fn kind_name(&self) -> &'static str {
        match self {
            Self::Newtonian { .. } => "Newtonian",
            Self::Minkowski { .. } => "Minkowski",
            Self::Discrete { .. } => "Discrete",
            Self::Entropic { .. } => "Entropic",
        }
    }
}
