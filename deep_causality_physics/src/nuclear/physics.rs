/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::constants::universal::SPEED_OF_LIGHT;
use crate::dynamics::quantities::Mass;
use crate::error::PhysicsError;
use crate::nuclear::quantities::{AmountOfSubstance, HalfLife};
use crate::quantum::quantities::{Energy, Time};
use deep_causality_core::{CausalityError, PropagatingEffect};

// Kernels

pub fn radioactive_decay_kernel(
    n0: &AmountOfSubstance,
    half_life: &HalfLife,
    time: &Time,
) -> Result<AmountOfSubstance, PhysicsError> {
    // N(t) = N0 * (1/2)^(t / t_half)
    // Decay Law using Half-Life

    if half_life.value() == 0.0 {
        // Avoid division by zero, though HalfLife check usually prevents negative. 0 might be valid "instant decay"?
        // Assume 0 half life means instant decay to 0?
        return AmountOfSubstance::new(0.0);
    }

    let ratio = time.value() / half_life.value();
    let n = n0.value() * 0.5_f64.powf(ratio);
    AmountOfSubstance::new(n)
}

pub fn binding_energy_kernel(mass_defect: &Mass) -> Result<Energy, PhysicsError> {
    // E = m c^2
    // Mass-Energy Equivalence
    let c = SPEED_OF_LIGHT;
    let e = mass_defect.value() * c * c;
    Energy::new(e)
}

// Wrappers

pub fn radioactive_decay(
    n0: &AmountOfSubstance,
    half_life: &HalfLife,
    time: &Time,
) -> PropagatingEffect<AmountOfSubstance> {
    match radioactive_decay_kernel(n0, half_life, time) {
        Ok(n) => PropagatingEffect::pure(n),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn binding_energy(mass_defect: &Mass) -> PropagatingEffect<Energy> {
    match binding_energy_kernel(mass_defect) {
        Ok(e) => PropagatingEffect::pure(e),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}
