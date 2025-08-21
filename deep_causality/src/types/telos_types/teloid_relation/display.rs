/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::TeloidRelation;

impl std::fmt::Display for TeloidRelation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TeloidRelation::Inherits => write!(f, "Inherits"),
            TeloidRelation::Defeats => write!(f, "Defeats"),
        }
    }
}
