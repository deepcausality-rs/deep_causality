// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::fmt::{Display, Formatter};

// Internal enum to represent the type of causaloid, which
// is required to dispatch verify and explain method calls to
// either a singleton, a causal collection, or causal graph.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum CausaloidType {
    Singleton,
    Collection,
    Graph,
}

impl Display for CausaloidType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

