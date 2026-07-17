/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

pub(crate) mod condensed;
pub(crate) mod earth;
pub(crate) mod electro_weak;
pub(crate) mod electromagnetic;
pub(crate) mod hypersonic;
/// Common physical constants used throughout the library.
///
/// Values are taken from the [CODATA 2022](https://physics.nist.gov/cuu/Constants/) recommended values.
///
/// Note:
/// * "Exact" values are defined by international agreement (SI definition).
/// * Other values are experimental measurements with associated uncertainties (ignored here for standard f64 precision).
pub(crate) mod nuclear;
pub(crate) mod particle;
pub(crate) mod propulsion;
pub(crate) mod thermodynamics;
pub(crate) mod universal;

pub use condensed::*;
pub use earth::*;
pub use electro_weak::*;
pub use electromagnetic::*;
pub use hypersonic::*;
pub use nuclear::*;
pub use particle::*;
pub use propulsion::*;
pub use thermodynamics::*;
pub use universal::*;

use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

/// Casts an `f64` physical constant to a target real-field precision `R`.
///
/// Physical constants are defined in `f64` (the precision at which the vast
/// majority of CODATA and experimental values are published). Generic,
/// real-field kernels need those same constants at their own precision `R`
/// (e.g. `f32` or `f64`). This helper is the single primitive that performs
/// that conversion, and the typed constant accessors (for example
/// [`graphene_lattice_const`] and [`reduced_planck_constant`]) are thin
/// wrappers over it.
///
/// # Panics
/// Panics only if `R::from_f64` returns `None`, which is impossible for the
/// standard real fields (`f32`, `f64`): every finite, in-range `f64` constant
/// converts. The panic therefore signals a logic error in a custom `R`
/// implementation, not a runtime failure mode.
#[inline]
pub fn real_from_f64<R: RealField + FromPrimitive>(x: f64) -> R {
    R::from_f64(x).expect("physical constant out of range for target real field")
}
