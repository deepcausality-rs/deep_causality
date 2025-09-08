/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{ActionError, CausalAction};

impl CausalAction {
    pub fn action(&self) -> fn() -> Result<(), ActionError> {
        self.action
    }

    pub fn descr(&self) -> &'static str {
        self.descr
    }

    pub fn version(&self) -> usize {
        self.version
    }
}
