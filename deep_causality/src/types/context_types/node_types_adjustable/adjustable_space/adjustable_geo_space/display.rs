// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::fmt;
use std::fmt::{Display, Formatter};
use crate::prelude::AdjustableGeoSpace;

/// Implements human-readable formatting for a geographic spatial context.
///
/// Format:
/// ```text
/// AdjustableGeoSpace(id="S1", lat=52.52, lon=13.405, alt=34.0)
/// ```
impl Display for AdjustableGeoSpace {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AdjustableGeoSpace(id=\"{}\", lat={:.6}, lon={:.6}, alt={}m)",
            self.id, self.lat, self.lon, self.alt
        )
    }
}