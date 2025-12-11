/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::nuclear::physics;
use crate::{AmountOfSubstance, Energy, EnergyDensity, HalfLife, Mass, PhysicalVector, Time};
use deep_causality_core::{CausalityError, PropagatingEffect};

/// Causal wrapper for [`physics::radioactive_decay_kernel`].
pub fn radioactive_decay(
    n0: &AmountOfSubstance,
    half_life: &HalfLife,
    time: &Time,
) -> PropagatingEffect<AmountOfSubstance> {
    match physics::radioactive_decay_kernel(n0, half_life, time) {
        Ok(n) => PropagatingEffect::pure(n),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`physics::binding_energy_kernel`].
pub fn binding_energy(mass_defect: &Mass) -> PropagatingEffect<Energy> {
    match physics::binding_energy_kernel(mass_defect) {
        Ok(e) => PropagatingEffect::pure(e),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`physics::hadronization_kernel`].
pub fn hadronization(
    energy_density: &[EnergyDensity],
    threshold: f64,
    dim: usize,
) -> PropagatingEffect<Vec<PhysicalVector>> {
    match physics::hadronization_kernel(energy_density, threshold, dim) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}
