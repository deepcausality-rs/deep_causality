/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// Universal Constants (CODATA 2022)

pub const G: f64 = 9.80665; // m s^-2 (exact)

pub const SPEED_OF_LIGHT: f64 = 299_792_458.0; // m s^-1 (exact)

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
pub const VACUUM_MAGNETIC_PERMEABILITY: f64 = 1.256_637_061_27e-6; // N A^-2
pub const VACUUM_ELECTRIC_PERMITTIVITY: f64 = 8.854_187_818_8e-12; // F m^-1
pub const NEWTONIAN_CONSTANT_OF_GRAVITATION: f64 = 6.674_30e-11; // m^3 kg^-1 s^-2
