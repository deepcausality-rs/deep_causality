/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::TimeScale;

/// A spacetime node as a store holds it: one variant per `SpaceTimeKind` variant.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SpaceTimeRecord {
    Euclidean {
        x: f64,
        y: f64,
        z: f64,
        t: f64,
        scale: TimeScale,
    },
    Lorentzian {
        x: f64,
        y: f64,
        z: f64,
        t: f64,
        scale: TimeScale,
    },
    /// Position, time, the tangent vector and the local metric tensor, row-major.
    Tangent {
        x: f64,
        y: f64,
        z: f64,
        t: f64,
        dt: f64,
        dx: f64,
        dy: f64,
        dz: f64,
        metric: [[f64; 4]; 4],
    },
}

impl SpaceTimeRecord {
    /// The variant's name, for an error that says what was found.
    pub const fn kind_name(&self) -> &'static str {
        match self {
            Self::Euclidean { .. } => "Euclidean",
            Self::Lorentzian { .. } => "Lorentzian",
            Self::Tangent { .. } => "Tangent",
        }
    }
}
