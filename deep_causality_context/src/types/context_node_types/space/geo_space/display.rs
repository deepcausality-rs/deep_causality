/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::GeoSpace;
use core::fmt;
use core::fmt::{Display, Formatter};
use deep_causality_algebra::RealField;

/// Implements human-readable formatting for a geographic spatial context.
///
/// Format:
/// ```text
/// GeoSpace(id=1, lat=52.5200, lon=13.4050, alt=34.0000, datum=WGS84)
/// ```
impl<R: RealField + Display> Display for GeoSpace<R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "GeoSpace(id={}, lat={:.4}, lon={:.4}, alt={:.4}, datum={})",
            self.id, self.lat, self.lon, self.alt, self.datum
        )
    }
}
