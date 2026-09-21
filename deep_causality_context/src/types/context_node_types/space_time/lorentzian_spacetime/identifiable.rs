/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::ContextoidId;
use crate::LorentzianSpacetime;
use deep_causality_core::Identifiable;

impl Identifiable for LorentzianSpacetime {
    fn id(&self) -> ContextoidId {
        self.id
    }
}
