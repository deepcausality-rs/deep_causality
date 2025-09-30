/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::cdl::{Finalized, WithAnalysis};
use crate::{CDL, CdlError, ProcessResultFormatter};

// After results are analyzed
impl CDL<WithAnalysis> {
    /// Formats the analysis into a final, presentable result.
    ///
    /// # Arguments
    /// * `formatter` - An implementation of `ProcessResultFormatter`.
    ///
    /// # Returns
    /// A `CDL` instance in the `Finalized` state, or a `CdlError` if formatting fails.
    pub fn finalize<F>(self, formatter: F) -> Result<CDL<Finalized>, CdlError>
    where
        F: ProcessResultFormatter,
    {
        let formatted_result = formatter.format(&self.state.0)?;
        Ok(CDL {
            state: Finalized(formatted_result),
            config: self.config,
        })
    }
}
