/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::TimeScale;

/// A spacetime node as a store holds it: one variant per `SpaceTimeKind` variant.
///
/// Each variant holds an event's coordinates `t, x, y, z`. `Galilean`, `Newtonian` and
/// `Minkowski` count `t` in the unit their `scale` names; `Tangent` counts it in seconds.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SpaceTimeRecord {
    /// An event of Galilean spacetime.
    Galilean {
        t: f64,
        x: f64,
        y: f64,
        z: f64,
        scale: TimeScale,
    },
    /// An event of Newtonian spacetime, in coordinates at rest in its absolute space.
    Newtonian {
        t: f64,
        x: f64,
        y: f64,
        z: f64,
        scale: TimeScale,
    },
    /// An event of Minkowski spacetime, in an inertial frame.
    Minkowski {
        t: f64,
        x: f64,
        y: f64,
        z: f64,
        scale: TimeScale,
    },
    /// An event, its tangent vector and the metric tensor there, in seconds and metres. The tensor
    /// is row-major with indices ordered `t, x, y, z`.
    Tangent {
        t: f64,
        x: f64,
        y: f64,
        z: f64,
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
            Self::Galilean { .. } => "Galilean",
            Self::Newtonian { .. } => "Newtonian",
            Self::Minkowski { .. } => "Minkowski",
            Self::Tangent { .. } => "Tangent",
        }
    }
}
