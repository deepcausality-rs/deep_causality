/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::materials::mechanics;
use crate::{Stress, Temperature};
use deep_causality_core::{CausalityError, PropagatingEffect};
use deep_causality_tensor::CausalTensor;

/// Causal wrapper for [`mechanics::hookes_law_kernel`].
pub fn hookes_law(
    stiffness: &CausalTensor<f64>,
    strain: &CausalTensor<f64>,
) -> PropagatingEffect<CausalTensor<f64>> {
    match mechanics::hookes_law_kernel(stiffness, strain) {
        Ok(s) => PropagatingEffect::pure(s),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`mechanics::von_mises_stress_kernel`].
pub fn von_mises_stress(stress: &CausalTensor<f64>) -> PropagatingEffect<Stress> {
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
