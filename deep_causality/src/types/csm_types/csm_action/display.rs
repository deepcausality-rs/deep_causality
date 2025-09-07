/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::CausalAction;

impl std::fmt::Display for CausalAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CausalAction {{ descr: \"{}\", version: {} }}",
            self.descr, self.version
        )
    }
}
