/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::SymbolicTime;
use deep_causality_core::Identifiable;

impl Identifiable for SymbolicTime {
    fn id(&self) -> u64 {
        self.id
    }
}
