/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{GeoSpace, VerticalDatum};
use deep_causality_algebra::RealField;

impl<R: RealField> GeoSpace<R> {
    pub fn lat(&self) -> R {
        self.lat
    }

    pub fn alt(&self) -> R {
        self.alt
    }

    pub fn lon(&self) -> R {
        self.lon
    }

    pub fn datum(&self) -> VerticalDatum {
        self.datum
    }
}
