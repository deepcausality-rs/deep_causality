/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{ContextoidId, NoSpaceTime};
use deep_causality_algebra::RealField;
use deep_causality_core::Identifiable;

impl<R: RealField> Identifiable for NoSpaceTime<R> {
    /// Zero. Every instance is the same value, so they share one id.
    fn id(&self) -> ContextoidId {
        0
    }
}
