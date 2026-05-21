/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::materials::mechanics;
use crate::{Strain, StiffnessTensor, Stress, StressTensor, Temperature};
use core::fmt::Debug;
use deep_causality_core::{CausalityError, PropagatingEffect};
use deep_causality_num::{FromPrimitive, RealField};
use deep_causality_tensor::CausalTensor;

/// Causal wrapper for [`mechanics::hookes_law_kernel`].
pub fn hookes_law<R>(
    stiffness: &StiffnessTensor<R>,
    strain: &Strain<R>,
) -> PropagatingEffect<StressTensor<R>>
where
    R: RealField + Default + Debug,
{
    match mechanics::hookes_law_kernel(stiffness, strain) {
        Ok(s) => PropagatingEffect::pure(s),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`mechanics::von_mises_stress_kernel`].
pub fn von_mises_stress<R>(stress: &StressTensor<R>) -> PropagatingEffect<Stress<R>>
where
    R: RealField + Default + FromPrimitive + Debug,
{
    match mechanics::von_mises_stress_kernel(stress) {
        Ok(s) => PropagatingEffect::pure(s),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`mechanics::thermal_expansion_kernel`].
pub fn thermal_expansion(
    coeff: f64,
    delta_temp: Temperature,
) -> PropagatingEffect<CausalTensor<f64>> {
    match mechanics::thermal_expansion_kernel(coeff, delta_temp) {
        Ok(t) => PropagatingEffect::pure(t),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}
