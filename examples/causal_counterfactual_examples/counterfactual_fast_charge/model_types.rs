/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Domain types for the fast-charge counterfactuals.

use deep_causality_context::{Context, ContextoidId, Data, NoSpace, NoSpaceTime, NoTime};
use deep_causality_core::{CausalFlow, CausalityError};

/// Switch this alias to `f32` for low precision, `f64` for standard precision,
/// or `Float106` for high precision. Literals in this crate would need lifting
/// through `deep_causality_num::lift` to switch away from `f64`.
pub type FloatType = f64;

/// The cooling environment the cell charges in, one `Data` contextoid per quantity. The context
/// holds no position, clock or event, so its spatial, temporal and spacetime slots are empty.
pub type CoolingContext =
    Context<Data<FloatType>, NoSpace<FloatType>, NoTime, NoSpaceTime<FloatType>>;

/// Contextoid id: coolant inlet temperature, °C.
pub const COOLANT_C: ContextoidId = 1;
/// Contextoid id: fraction of the cell's rise above the coolant temperature removed per minute.
pub const COOLING_PER_MIN: ContextoidId = 2;

/// A fast-charging run. The value is the cell temperature (°C), the state is the charge (%), and
/// the context is the cooling environment.
pub type Charge = CausalFlow<FloatType, FloatType, CoolingContext>;

/// Read the payload of the `Data` contextoid `id` out of the cooling context.
pub fn read(context: &CoolingContext, id: ContextoidId) -> Result<FloatType, CausalityError> {
    context.get_data_by_id(id).ok_or_else(|| {
        CausalityError::MissingParameter(format!(
            "the cooling context holds no Datoid with contextoid id {id}"
        ))
    })
}
