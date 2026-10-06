/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::utils::seconds::seconds;
use crate::{MinkowskiSpacetime, SpaceTemporalInterval};
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

impl<R: RealField + FromPrimitive> SpaceTemporalInterval for MinkowskiSpacetime<R> {
    /// `t` in seconds, or NaN when `time_scale` names no duration.
    fn time(&self) -> R {
        seconds(self.t, self.time_scale).unwrap_or_else(R::nan)
    }

    fn position(&self) -> [R; 3] {
        [self.x, self.y, self.z]
    }
}
