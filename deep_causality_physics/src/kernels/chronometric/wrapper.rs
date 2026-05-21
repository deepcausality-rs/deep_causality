/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use core::fmt::Debug;

use deep_causality_core::{CausalityError, PropagatingEffect};
use deep_causality_num::RealField;

use crate::{CentralBody, SpaceTimeCoordinate, solve_gm_analytical_kernel};

/// Causal wrapper for [`solve_gm_analytical_kernel`].
///
/// Produces a [`PropagatingEffect`] containing the recovered GM (m³/s²) on
/// success, or a propagating causality error on failure. This is the entry
/// point for embedding the J2-corrected weak-field GM inversion into a
/// causaloid graph or causal monad pipeline.
///
/// # Type Parameter
///
/// - `R`: Real field type (e.g., `f64`, `Float106`). Callers select precision
///   by choosing `R`; the kernel is generic over the same trait.
pub fn solve_gm_analytical<R>(
    coord_a: &SpaceTimeCoordinate<R>,
    coord_b: &SpaceTimeCoordinate<R>,
    body: &CentralBody<R>,
) -> PropagatingEffect<R>
where
    R: RealField + From<f64> + Default + Debug,
{
    match solve_gm_analytical_kernel(coord_a, coord_b, body) {
        Ok(gm) => PropagatingEffect::pure(gm),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}
