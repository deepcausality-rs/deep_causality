/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::GeoSpace;
use deep_causality_algebra::RealField;
use std::fmt;
use std::fmt::{Display, Formatter};

/// Implements human-readable formatting for a geographic spatial context.
///
/// Format:
/// ```text
/// GeoSpace(id="S1", lat=52.52, lon=13.405, alt=34.0, datum=WGS84)
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
