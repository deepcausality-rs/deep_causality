/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! This module defines the LogAppend trait.

/// Trait for types that can append log entries from another instance of themselves.
pub trait LogAppend {
    /// Appends the log entries from `other` into `self`.
    fn append(&mut self, other: &mut Self);
}
