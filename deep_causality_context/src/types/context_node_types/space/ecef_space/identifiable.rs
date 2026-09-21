/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::EcefSpace;
use deep_causality_core::Identifiable;

impl Identifiable for EcefSpace {
    fn id(&self) -> u64 {
        self.id
    }
}
