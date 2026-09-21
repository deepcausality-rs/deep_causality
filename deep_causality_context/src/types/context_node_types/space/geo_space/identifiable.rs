/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::ContextoidId;
use crate::GeoSpace;
use deep_causality_core::Identifiable;

impl Identifiable for GeoSpace {
    fn id(&self) -> ContextoidId {
        self.id
    }
}
