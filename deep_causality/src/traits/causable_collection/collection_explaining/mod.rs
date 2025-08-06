/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
mod causable_explaining;

use crate::{Causable, CausableCollectionAccessor, CausalityError};

pub trait CausableCollectionExplaining<T>: CausableCollectionAccessor<T>
where
    T: Causable,
{
    /// Generates an explanation by concatenating the `explain()` text of all causes.
    ///
    /// Each explanation is formatted and separated by newlines.
    /// It gracefully handles errors from individual `explain` calls by inserting
    /// a placeholder error message.
    fn explain(&self) -> Result<String, CausalityError> {
        // Delegate to private impl in causable_reasoning_explain
        causable_explaining::_explain(self.get_all_items())
    }
}
