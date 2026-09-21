/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::ContextoidId;
use crate::MinkowskiSpacetime;
use deep_causality_core::Identifiable;

impl Identifiable for MinkowskiSpacetime {
    fn id(&self) -> ContextoidId {
        self.id
    }
}
