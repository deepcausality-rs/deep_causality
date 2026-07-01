/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

pub const GRAPHENE_LATTICE_CONST: f64 = 0.246e-9; // Approximate experimental value for graphene's lattice constant (in meters)

use deep_causality_num::{FromPrimitive, RealField};

/// Returns [`GRAPHENE_LATTICE_CONST`] at the target real-field precision `R`.
///
/// Companion accessor that lets real-field kernels obtain the graphene lattice
/// constant in their own precision without hand-casting the `f64` value. See
/// [`crate::real_from_f64`] for the conversion contract.
#[inline]
pub fn graphene_lattice_const<R: RealField + FromPrimitive>() -> R {
    crate::constants::real_from_f64(GRAPHENE_LATTICE_CONST)
}
