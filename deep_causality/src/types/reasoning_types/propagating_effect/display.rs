/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::PropagatingEffect;
use std::fmt::{Display, Formatter};

impl Display for PropagatingEffect {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // Delegate to custom debug implementation to prevent infinite recursion.
        write!(f, "{self:?}")
    }
}
