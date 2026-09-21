/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{GeoSpace, VerticalDatum};

impl GeoSpace {
    pub fn lat(&self) -> f64 {
        self.lat
    }

    pub fn alt(&self) -> f64 {
        self.alt
    }

    pub fn lon(&self) -> f64 {
        self.lon
    }

    pub fn datum(&self) -> VerticalDatum {
        self.datum
    }
}
