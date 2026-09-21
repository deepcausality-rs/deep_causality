/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{EuclideanSpacetime, SpaceTemporal};
use deep_causality_algebra::RealField;

impl<R: RealField> SpaceTemporal for EuclideanSpacetime<R> {
    fn t(&self) -> &R {
        &self.t
    }
}
