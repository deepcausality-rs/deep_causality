/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::ProposedAction;
use deep_causality_core::Identifiable;

impl Identifiable for ProposedAction {
    fn id(&self) -> u64 {
        self.action_id
    }
}
