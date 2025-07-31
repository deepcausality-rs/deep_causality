/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//!
//! Implementation of the Transferable trait for the Model struct.
//!

use crate::types::model_types::model::Model;
use crate::{Assumption, Datable, SpaceTemporal, Spatial, Symbolic, Temporal, Transferable};
use std::sync::Arc;

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
    fn get_assumptions(&self) -> &Option<Arc<Vec<Assumption>>> {
        &self.assumptions
    }

    // verify_assumptions is derived from the Transferable trait. Overwrite for customization.
    // fn verify_assumptions(&self, effect: &[PropagatingEffect]) -> Result<bool, AssumptionError> {}
}
