/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Model layer for the hyperlens: the optics constants, the permittivity a metric implies, and the
//! dispersion relation the two are fed into.
//!
//! Every constant is declared at the working type through `const_scalar_from_int!` and
//! `const_scalar_from_float!`, so the compiler resolves them against the alias in `main`.

use crate::FloatType;
use deep_causality_algebra::Real;
use deep_causality_metric::Metric;
use deep_causality_num::{const_scalar_from_float, const_scalar_from_int};

// =============================================================================
// The small numbers the optics is written with
// =============================================================================

pub const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);
pub const TWO: FloatType = const_scalar_from_int!(FloatType, 2);

// =============================================================================
// Optics
// =============================================================================

/// The illuminating wavelength in nanometres. Five hundred is green light.
pub const WAVELENGTH_NM: FloatType = const_scalar_from_int!(FloatType, 500);

/// The magnitude every principal permittivity takes in this model.
///
/// A real metamaterial has magnitudes that differ per axis and vary with frequency. Holding them
/// equal isolates the one thing this example is about: the **sign** pattern, which is what the
/// metric carries.
pub const EPSILON_MAGNITUDE: FloatType = const_scalar_from_float!(FloatType, 1.0);

/// The principal axes, by generator index in the metric.
pub const AXIS_X: usize = 0;
pub const AXIS_Y: usize = 1;
pub const AXIS_Z: usize = 2;

/// Object periods to probe, in nanometres, from coarse to fine.
pub const SEPARATIONS_NM: [FloatType; 8] = [
    const_scalar_from_int!(FloatType, 1000),
    const_scalar_from_int!(FloatType, 750),
    const_scalar_from_int!(FloatType, 500),
    const_scalar_from_int!(FloatType, 400),
    const_scalar_from_int!(FloatType, 300),
    const_scalar_from_int!(FloatType, 200),
    const_scalar_from_int!(FloatType, 100),
    const_scalar_from_int!(FloatType, 50),
];

// =============================================================================
// The two materials, as metrics
// =============================================================================

/// Vacuum: every axis squares to `+1`, so every principal permittivity is positive.
pub fn vacuum() -> Metric {
    Metric::Euclidean(3)
}

/// A Type I hyperbolic metamaterial: `ε_x = ε_y > 0` and `ε_z < 0`.
///
/// That sign pattern is the signature `(+, +, −)`, which is `Cl(2, 1)`. The optical axis is the
/// one negative generator, and it is what opens the dispersion surface from a sphere into a
/// hyperboloid.
pub fn hyperbolic_metamaterial() -> Metric {
    Metric::Generic { p: 2, q: 1, r: 0 }
}

/// The principal permittivity along one axis, its sign read from the metric.
///
/// This is the whole of the example's claim: the material is described by a metric signature, and
/// the optics below reads the signature rather than a hand-written sign. Swapping the metric
/// swaps the physics.
pub fn permittivity(metric: &Metric, axis: usize) -> FloatType {
    match metric.sign_of_sq(axis) {
        sign if sign > 0 => EPSILON_MAGNITUDE,
        sign if sign < 0 => -EPSILON_MAGNITUDE,
        _ => ZERO,
    }
}

// =============================================================================
// Dispersion
// =============================================================================

/// The free-space wavenumber `k₀ = 2π/λ`, in radians per nanometre.
pub fn free_space_wavenumber() -> FloatType {
    TWO * FloatType::pi() / WAVELENGTH_NM
}

/// The in-plane spatial frequency a periodic object of period `d` carries, in radians per
/// nanometre: `k_x = 2π/d`. A finer object needs a higher `k_x`.
pub fn spatial_frequency(separation_nm: FloatType) -> FloatType {
    TWO * FloatType::pi() / separation_nm
}

/// The squared out-of-plane wavenumber `k_z²` for a TM wave in a uniaxial medium.
///
/// The dispersion relation is
///
/// ```text
/// k_x²/ε_z + k_z²/ε_x = k₀²      so      k_z² = ε_x·(k₀² − k_x²/ε_z)
/// ```
///
/// A positive `k_z²` is a propagating wave and a negative one is evanescent, decaying instead of
/// carrying its detail to the far field. With every permittivity positive the relation is a
/// sphere and large `k_x` drives `k_z²` negative. With `ε_z` negative the second term changes
/// sign, the surface opens into a hyperboloid, and `k_z²` stays positive at every `k_x`.
pub fn squared_out_of_plane_wavenumber(metric: &Metric, separation_nm: FloatType) -> FloatType {
    let epsilon_x = permittivity(metric, AXIS_X);
    let epsilon_z = permittivity(metric, AXIS_Z);

    let k_0 = free_space_wavenumber();
    let k_x = spatial_frequency(separation_nm);

    epsilon_x * (k_0 * k_0 - k_x * k_x / epsilon_z)
}
