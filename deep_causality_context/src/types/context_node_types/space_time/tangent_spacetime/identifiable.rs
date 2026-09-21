/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::TangentSpacetime;
use deep_causality_core::Identifiable;

impl Identifiable for TangentSpacetime {
    fn id(&self) -> u64 {
        self.id
    }
}
