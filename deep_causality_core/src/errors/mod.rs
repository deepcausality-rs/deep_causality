/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use alloc::string::String;
use core::fmt::{Debug, Display, Formatter};

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct CausalityError(pub String);

impl CausalityError {
    pub fn new(message: String) -> Self {
        CausalityError(message)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for CausalityError {}

impl Display for CausalityError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}
