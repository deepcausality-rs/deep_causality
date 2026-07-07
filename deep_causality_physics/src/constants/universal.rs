/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// Universal Constants (CODATA 2022)

pub const G: f64 = 9.80665; // m s^-2 (exact)

pub const SPEED_OF_LIGHT: f64 = 299_792_458.0; // m s^-1 (exact)

use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

/// Returns [`SPEED_OF_LIGHT`] (c) at the target real-field precision `R`.
///
/// Companion accessor that lets real-field kernels obtain the speed of light in
/// their own precision without hand-casting the `f64` value. See
/// [`crate::real_from_f64`] for the conversion contract.
#[inline]
pub fn speed_of_light<R: RealField + FromPrimitive>() -> R {
    crate::constants::real_from_f64(SPEED_OF_LIGHT)
}

/// Cosmological constant upper bound (in m⁻²)
///
/// ```text
/// Λ ≈ 1.1 × 10⁻⁵² m⁻² (from observations)
/// ```
pub const COSMOLOGICAL_CONSTANT: f64 = 1.1e-52;

/// Planck length in meters
///
/// ```text
/// l_P = √(ℏG/c³) ≈ 1.616 × 10⁻³⁵ m
/// ```
pub const PLANCK_LENGTH: f64 = 1.616255e-35;
/// Planck mass in kg
///
/// ```text
/// m_P = √(ℏc/G) ≈ 2.176 × 10⁻⁸ kg
/// ```
pub const PLANCK_MASS: f64 = 2.176434e-8;
pub const PLANCK_CONSTANT: f64 = 6.626_070_15e-34; // J Hz^-1 (exact)
pub const REDUCED_PLANCK_CONSTANT: f64 = 1.054_571_817e-34; // J s

/// Returns [`REDUCED_PLANCK_CONSTANT`] (ℏ) at the target real-field precision `R`.
///
/// Companion accessor that lets real-field kernels obtain the reduced Planck
/// constant in their own precision without hand-casting the `f64` value. See
/// [`crate::real_from_f64`] for the conversion contract.
#[inline]
pub fn reduced_planck_constant<R: RealField + FromPrimitive>() -> R {
    crate::constants::real_from_f64(REDUCED_PLANCK_CONSTANT)
}
pub const VACUUM_MAGNETIC_PERMEABILITY: f64 = 1.256_637_061_27e-6; // N A^-2
pub const VACUUM_ELECTRIC_PERMITTIVITY: f64 = 8.854_187_818_8e-12; // F m^-1
pub const NEWTONIAN_CONSTANT_OF_GRAVITATION: f64 = 6.674_30e-11; // m^3 kg^-1 s^-2
