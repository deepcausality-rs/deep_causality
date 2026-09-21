/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::fmt::{Display, Formatter};

/// The reference an altitude is measured against.
///
/// An altitude is a number and a reference, and the number alone does not locate anything. Two
/// readings of `34.0` against different members of this enum name different points, which is why
/// the datum is stored beside the altitude rather than fixed for a whole type.
///
/// The members are grouped by the height type each belongs under, following the ISO 19111
/// classification:
///
/// - **Ellipsoidal height** — measured along the ellipsoid normal: [`VerticalDatum::WGS84`].
/// - **Gravity-related height** — measured against an equipotential surface of the gravity field:
///   [`VerticalDatum::EGM96`] and [`VerticalDatum::EGM2008`].
/// - **Neither** — [`VerticalDatum::ISA`] is a pressure expressed in length units, and
///   [`VerticalDatum::Terrain`] is measured against a surface that varies with position and with
///   the model that produced it.
#[derive(Debug, Default, Copy, Clone, Hash, Eq, PartialEq)]
#[repr(u8)]
pub enum VerticalDatum {
    /// Height above the WGS84 reference ellipsoid, measured along the ellipsoid normal.
    ///
    /// This is what a GNSS receiver computes before any geoid model is applied. It is a geometric
    /// quantity and does not tell you which way water flows.
    #[default]
    WGS84,
    /// Height above the EGM96 geoid, a 1996 gravity model on a 15-arc-minute grid.
    ///
    /// Gravity-related, so it approximates height above mean sea level.
    EGM96,
    /// Height above the EGM2008 geoid, a 2008 gravity model on a 1-arc-minute grid.
    ///
    /// Gravity-related, like [`VerticalDatum::EGM96`], and the finer of the two models.
    EGM2008,
    /// Pressure altitude under the International Standard Atmosphere.
    ///
    /// This is not a height. It is the altitude at which the ISA model predicts the measured
    /// static pressure, so it tracks the weather rather than the ground, and two aircraft holding
    /// the same ISA altitude on different days are at different geometric heights.
    ISA,
    /// Height above ground level.
    ///
    /// The reference surface varies with horizontal position, so this datum is meaningful only at
    /// the point of measurement. It also depends on the terrain model used, which this enum does
    /// not name: two terrain heights are comparable only when they came from the same model.
    Terrain,
}

impl Display for VerticalDatum {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
