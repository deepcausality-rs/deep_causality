/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::astro::mechanics;
use crate::dynamics::{Length, Mass, Speed};
use deep_causality_core::{CausalityError, PropagatingEffect};

/// Causal wrapper for [`mechanics::orbital_velocity_kernel`].
pub fn orbital_velocity(mass: &Mass, radius: &Length) -> PropagatingEffect<Speed> {
    match mechanics::orbital_velocity_kernel(mass, radius) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`mechanics::escape_velocity_kernel`].
pub fn escape_velocity(mass: &Mass, radius: &Length) -> PropagatingEffect<Speed> {
    match mechanics::escape_velocity_kernel(mass, radius) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`mechanics::schwarzschild_radius_kernel`].
pub fn schwarzschild_radius(mass: &Mass) -> PropagatingEffect<Length> {
    match mechanics::schwarzschild_radius_kernel(mass) {
        Ok(r) => PropagatingEffect::pure(r),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}
