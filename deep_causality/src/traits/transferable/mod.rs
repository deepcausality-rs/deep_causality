/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//!
//! Trait for model verification.
//!
use crate::{AssumptionError, PropagatingEffect};

pub trait Transferable {
    /// Verifies the model's assumptions against a given PropagatingEffect.
    ///
    /// # Arguments
    /// * `effect` - The output of a model run or sample data to be tested.
    ///
    /// # Returns
    /// * `Ok(true)` if all assumptions hold true.
    /// * `Ok(false)` if any assumption fails evaluation.
    /// * `Err(AssumptionError)` if the model has no assumptions or an evaluation error occurs.
    fn verify_assumptions(&self, effect: &[PropagatingEffect]) -> Result<bool, AssumptionError>;
}
