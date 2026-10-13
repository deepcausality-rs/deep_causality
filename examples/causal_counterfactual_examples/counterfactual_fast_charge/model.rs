/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The charging law and the one-minute causal step.

use crate::model_config::{
    CELL_CAPACITY_AH, CUTOFF_C, DERATE_C, FULL_CURRENT_A, HALF_CURRENT_A, HEATING_C_PER_A2,
    START_CHARGE_PCT, TAPER_FROM_PCT,
};
use crate::model_types::{COOLANT_C, COOLING_PER_MIN, Charge, CoolingContext, FloatType, read};
use deep_causality_core::{CausalEffect, CausalFlow, CausalityError, EffectLog};

/// The charger's current. Temperature sets the ceiling: full current below 45 °C, half below
/// 55 °C, none from 55 °C up. Charge shapes it: above 80 % the charger holds the cell voltage and the
/// current falls linearly to zero at 100 %.
pub fn charge_current_a(cell_c: FloatType, charge_pct: FloatType) -> FloatType {
    let ceiling = if cell_c < DERATE_C {
        FULL_CURRENT_A
    } else if cell_c < CUTOFF_C {
        HALF_CURRENT_A
    } else {
        0.0
    };
    let taper = ((100.0 - charge_pct) / (100.0 - TAPER_FROM_PCT)).clamp(0.0, 1.0);
    ceiling * taper
}

/// One minute of charging. The current adds charge and heats the cell; the coolant removes heat
/// in proportion to the cell's rise above the coolant temperature, both read from the context.
pub fn one_minute(run: Charge) -> Charge {
    run.step_mut(|cell_c, charge_pct, cooling| {
        let cooling = cooling.ok_or_else(CausalityError::MissingContext)?;
        let coolant_c = read(cooling, COOLANT_C)?;
        let cooling_per_min = read(cooling, COOLING_PER_MIN)?;
        let amps = charge_current_a(cell_c, *charge_pct);
        *charge_pct = (*charge_pct + amps / CELL_CAPACITY_AH * 100.0 / 60.0).min(100.0);
        Ok(cell_c + HEATING_C_PER_A2 * amps * amps - cooling_per_min * (cell_c - coolant_c))
    })
}

/// The cell plugged in at the coolant temperature, 20 % charged, in `cooling`.
pub fn plug_in(cooling: CoolingContext) -> Result<Charge, CausalityError> {
    let cell_c = read(&cooling, COOLANT_C)?;
    Ok(CausalFlow::from_parts(
        Ok(CausalEffect::value(cell_c)),
        START_CHARGE_PCT,
        Some(cooling),
        EffectLog::new(),
    ))
}
