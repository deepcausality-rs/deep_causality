/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{ActionParameterValue, ProposedAction};
use std::collections::HashMap;

impl ProposedAction {
    pub fn action_id(&self) -> u64 {
        self.action_id
    }

    pub fn action_name(&self) -> &str {
        &self.action_name
    }

    pub fn parameters(&self) -> &HashMap<String, ActionParameterValue> {
        &self.parameters
    }
}
