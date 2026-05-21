/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::kernels::astro::mechanics;
use crate::kernels::dynamics::{Length, Mass, Speed};
use core::fmt::Debug;
use deep_causality_core::{CausalityError, PropagatingEffect};
use deep_causality_num::{FromPrimitive, RealField};

/// Causal wrapper for [`mechanics::orbital_velocity_kernel`].
pub fn orbital_velocity<R>(mass: &Mass<R>, radius: &Length<R>) -> PropagatingEffect<Speed<R>>
where
    R: RealField + FromPrimitive + Debug,
{
    match mechanics::orbital_velocity_kernel(mass, radius) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`mechanics::escape_velocity_kernel`].
pub fn escape_velocity<R>(mass: &Mass<R>, radius: &Length<R>) -> PropagatingEffect<Speed<R>>
where
    R: RealField + FromPrimitive + Debug,
{
    match mechanics::escape_velocity_kernel(mass, radius) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`mechanics::schwarzschild_radius_kernel`].
pub fn schwarzschild_radius<R>(mass: &Mass<R>) -> PropagatingEffect<Length<R>>
where
    R: RealField + FromPrimitive + Debug,
{
    match mechanics::schwarzschild_radius_kernel(mass) {
        Ok(r) => PropagatingEffect::pure(r),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}
