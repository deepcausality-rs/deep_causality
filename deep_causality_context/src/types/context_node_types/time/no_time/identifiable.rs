/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{ContextoidId, NoTime};
use deep_causality_core::Identifiable;

impl Identifiable for NoTime {
    /// Zero. Every instance is the same value, so they share one id.
    fn id(&self) -> ContextoidId {
        0
    }
}
