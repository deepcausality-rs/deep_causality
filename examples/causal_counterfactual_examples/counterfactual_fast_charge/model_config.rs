/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The cell, the charger, the cooling environments and the counterfactual questions.

use crate::model_types::{COOLANT_C, COOLING_PER_MIN, CoolingContext, FloatType};
use deep_causality_context::{
    Context, ContextIndexError, Contextoid, ContextoidType, ContextuableGraph, Data,
};

/// Cell capacity, ampere-hours.
pub const CELL_CAPACITY_AH: FloatType = 50.0;
/// Joule heating per minute per ampere squared, °C.
pub const HEATING_C_PER_A2: FloatType = 0.00048;
/// Charge at plug-in, %.
pub const START_CHARGE_PCT: FloatType = 20.0;

/// Charger current below the derating temperature, A.
pub const FULL_CURRENT_A: FloatType = 50.0;
/// Charger current between the derating and the cut-off temperature, A.
pub const HALF_CURRENT_A: FloatType = 25.0;
/// Cell temperature at which the charger halves its current, °C.
pub const DERATE_C: FloatType = 45.0;
/// Cell temperature at which the charger stops, °C.
pub const CUTOFF_C: FloatType = 55.0;
/// Charge above which the charger holds the cell voltage and the current tapers to zero at 100 %.
pub const TAPER_FROM_PCT: FloatType = 80.0;

/// Length of every run, minutes.
pub const RUN_MINUTES: usize = 30;
/// Minute at which each counterfactual replaces one channel.
pub const BRANCH_MINUTE: usize = 10;

/// Value counterfactual: the cell temperature at the branch minute, °C.
pub const HOT_CELL_C: FloatType = 52.0;
/// State counterfactual: the charge at the branch minute, %.
pub const HIGH_CHARGE_PCT: FloatType = 75.0;

/// Coolant at full flow.
pub fn full_flow_cooling() -> Result<CoolingContext, ContextIndexError> {
    cooling("full flow", 25.0, 0.08)
}

/// Context counterfactual: the coolant pump at a quarter of its flow.
pub fn quarter_flow_cooling() -> Result<CoolingContext, ContextIndexError> {
    cooling("quarter flow", 25.0, 0.02)
}

fn cooling(
    name: &str,
    coolant_c: FloatType,
    cooling_per_min: FloatType,
) -> Result<CoolingContext, ContextIndexError> {
    let facts = [(COOLANT_C, coolant_c), (COOLING_PER_MIN, cooling_per_min)];
    let mut context = Context::with_capacity(1, name, facts.len());
    for (id, value) in facts {
        context.add_node(Contextoid::new(
            id,
            ContextoidType::Datoid(Data::new(id, value)),
        ))?;
    }
    Ok(context)
}
