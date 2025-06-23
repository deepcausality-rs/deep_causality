use std::fmt::Display;

// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum SymbolicResult {
    Proven,
    Disproven,
    Undetermined,
}

impl Display for SymbolicResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SymbolicResult::Proven => write!(f, "Proven"),
            SymbolicResult::Disproven => write!(f, "Disproven"),
            SymbolicResult::Undetermined => write!(f, "Undetermined"),
        }
    }
}
