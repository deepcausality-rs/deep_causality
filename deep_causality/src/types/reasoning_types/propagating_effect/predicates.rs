/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::CausalPropagatingEffect;
use std::fmt::Debug;

impl<Value: Debug, Error: Debug, Log: AsRef<str>> CausalPropagatingEffect<Value, Error, Log> {
    pub fn is_ok(&self) -> bool {
        self.error.is_none()
    }

    pub fn is_err(&self) -> bool {
        self.error.is_some()
    }

    pub fn has_log(&self) -> bool {
        !self.logs.is_empty()
    }
}
