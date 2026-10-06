/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::CausalSetSpacetime;
use crate::ContextoidId;
use deep_causality_core::Identifiable;

impl Identifiable for CausalSetSpacetime {
    fn id(&self) -> ContextoidId {
        self.id
    }
}
