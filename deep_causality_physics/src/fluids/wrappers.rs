/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::fluids::mechanics;
use crate::{Density, Length, Pressure, Speed};
use deep_causality_core::{CausalityError, PropagatingEffect};

/// Causal wrapper for [`mechanics::hydrostatic_pressure_kernel`].
pub fn hydrostatic_pressure(
    p0: &Pressure,
    density: &Density,
    depth: &Length,
) -> PropagatingEffect<Pressure> {
    match mechanics::hydrostatic_pressure_kernel(p0, density, depth) {
        Ok(p) => PropagatingEffect::pure(p),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`mechanics::bernoulli_pressure_kernel`].
pub fn bernoulli_pressure(
    p1: &Pressure,
    v1: &Speed,
    h1: &Length,
    v2: &Speed,
    h2: &Length,
    density: &Density,
) -> PropagatingEffect<Pressure> {
    match mechanics::bernoulli_pressure_kernel(p1, v1, h1, v2, h2, density) {
        Ok(p) => PropagatingEffect::pure(p),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}
