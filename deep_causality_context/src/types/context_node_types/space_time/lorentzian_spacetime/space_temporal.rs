/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{LorentzianSpacetime, SpaceTemporal};
use deep_causality_algebra::RealField;

impl<R: RealField> SpaceTemporal for LorentzianSpacetime<R> {
    fn t(&self) -> &R {
        &self.t
    }
}
