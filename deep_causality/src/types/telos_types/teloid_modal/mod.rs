/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::fmt::{Display, Formatter};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TeloidModal {
    #[default]
    Obligatory,
    Impermissible,
    Optional(i64), // Optional must be associated with a cost value.
}

impl Display for TeloidModal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TeloidModal::Obligatory => write!(f, "Obligatory"),
            TeloidModal::Impermissible => write!(f, "Impermissible"),
            TeloidModal::Optional(cost) => write!(f, "Optional({cost})"),
        }
    }
}
