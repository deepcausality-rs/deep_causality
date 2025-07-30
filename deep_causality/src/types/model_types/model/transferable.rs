/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//!
//! Implementation of the Transferable trait for the Model struct.
//!
use crate::types::model_types::model::Model;
use crate::{
    Assumable, AssumptionError, Datable, PropagatingEffect, SpaceTemporal, Spatial, Symbolic,
    Temporal, Transferable,
};

impl<D, S, T, ST, SYM, VS, VT> Transferable for Model<D, S, T, ST, SYM, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    fn verify_assumptions(&self, effect: &[PropagatingEffect]) -> Result<bool, AssumptionError> {
        if effect.is_empty() {
            return Err(AssumptionError::NoDataToTestDefined);
        }

        if self.assumptions.is_none() {
            return Err(AssumptionError::NoAssumptionsDefined);
        }

         let assumptions = self.assumptions.as_ref().unwrap();
           for assumption in assumptions.iter() {
                match assumption.verify_assumption(effect) {
                    Ok(true) => continue,          // Assumption holds, continue checking
                    Ok(false) => return Ok(false), // Assumption failed
                    Err(e) => return Err(e),       // An error occurred during evaluation
                }
            }
            Ok(true) // All assumptions passed

    }
}
