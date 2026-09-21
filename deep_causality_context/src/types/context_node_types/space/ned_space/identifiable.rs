/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::ContextoidId;
use crate::NedSpace;
use deep_causality_algebra::RealField;
use deep_causality_core::Identifiable;

impl<R: RealField> Identifiable for NedSpace<R> {
    fn id(&self) -> ContextoidId {
        self.id
    }
}
