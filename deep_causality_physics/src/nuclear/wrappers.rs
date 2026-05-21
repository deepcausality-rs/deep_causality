/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::nuclear::physics;
use crate::{AmountOfSubstance, Energy, HalfLife, Mass, Time};
use core::fmt::Debug;
use deep_causality_core::{CausalityError, PropagatingEffect};
use deep_causality_num::{FromPrimitive, RealField};

/// Causal wrapper for [`physics::radioactive_decay_kernel`].
pub fn radioactive_decay<R>(
    n0: &AmountOfSubstance<R>,
    half_life: &HalfLife<R>,
    time: &Time<f64>,
) -> PropagatingEffect<AmountOfSubstance<R>>
where
    R: RealField + FromPrimitive + Debug,
{
    match physics::radioactive_decay_kernel(n0, half_life, time) {
        Ok(n) => PropagatingEffect::pure(n),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`physics::binding_energy_kernel`].
pub fn binding_energy(mass_defect: &Mass<f64>) -> PropagatingEffect<Energy<f64>> {
    match physics::binding_energy_kernel(mass_defect) {
        Ok(e) => PropagatingEffect::pure(e),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}
