/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::VerticalDatum;

/// A space node as a store holds it: one variant per `SpaceKind` variant. Every position is
/// named, never positional.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SpaceRecord {
    Geo {
        lat: f64,
        lon: f64,
        alt: f64,
        datum: VerticalDatum,
    },
    Ecef {
        x: f64,
        y: f64,
        z: f64,
    },
    Euclidean {
        x: f64,
        y: f64,
        z: f64,
    },
    Ned {
        north: f64,
        east: f64,
        down: f64,
    },
}

impl SpaceRecord {
    /// The variant's name, for an error that says what was found.
    pub const fn kind_name(&self) -> &'static str {
        match self {
            Self::Geo { .. } => "Geo",
            Self::Ecef { .. } => "Ecef",
            Self::Euclidean { .. } => "Euclidean",
            Self::Ned { .. } => "Ned",
        }
    }
}
