/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{TeloidID, TeloidModal, Verdict};

impl Verdict {
    /// Returns the outcome of the verdict.
    ///
    /// # Returns
    ///
    /// The `TeloidModal` representing the outcome.
    pub fn outcome(&self) -> TeloidModal {
        self.outcome
    }

    /// Returns a reference to the justification for the verdict.
    ///
    /// # Returns
    ///
    /// A reference to a `Vec<TeloidID>` containing the justification.
    pub fn justification(&self) -> &Vec<TeloidID> {
        &self.justification
    }
}
