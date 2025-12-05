/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::TeloidModal;
use std::fmt::{Display, Formatter};

impl Display for TeloidModal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TeloidModal::Obligatory => write!(f, "Obligatory"),
            TeloidModal::Impermissible => write!(f, "Impermissible"),
            TeloidModal::Optional(cost) => write!(f, "Optional({cost})"),
        }
    }
}
