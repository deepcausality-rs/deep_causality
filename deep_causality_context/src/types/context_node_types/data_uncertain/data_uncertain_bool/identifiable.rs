/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::ContextoidId;
use crate::types::context_node_types::data_uncertain::data_uncertain_bool::UncertainBoolData;
use deep_causality_core::Identifiable;
use deep_causality_uncertain::RandScalar;

impl<R: RandScalar> Identifiable for UncertainBoolData<R> {
    fn id(&self) -> ContextoidId {
        self.id
    }
}
