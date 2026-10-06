/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Model layer for the hyperlens: the optical world the sweep reads, the permittivity a metric
//! implies, and the dispersion relation the two are fed into.
//!
//! Every constant is declared at the working type through `const_scalar_from_int!` and
//! `const_scalar_from_float!`, so the compiler resolves them against the alias in `main`.

use crate::FloatType;
use deep_causality_algebra::Real;
use deep_causality_context::{
    Context, ContextIndexError, Contextoid, ContextoidId, ContextoidType, ContextuableGraph, Data,
    NoSpace, NoSpaceTime, NoTime,
};
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

/// The principal axes, by generator index in the metric.
pub const AXIS_X: usize = 0;
pub const AXIS_Y: usize = 1;
pub const AXIS_Z: usize = 2;

/// One fact of the optical world: a single magnitude, or the sweep of object periods.
#[derive(Debug, Clone, PartialEq)]
pub enum OpticsDatum {
    /// A single magnitude: the wavelength in nanometres, or the permittivity magnitude.
    Magnitude(FloatType),
    /// Object periods in nanometres, from coarse to fine.
    Periods(Vec<FloatType>),
}

impl Default for OpticsDatum {
    fn default() -> Self {
        Self::Magnitude(ZERO)
    }
}

/// The optical world the sweep reads: the illumination, the permittivity magnitude and the object
/// periods to probe.
///
/// The model works in wavenumbers and holds no position, instant or event, so the spatial, temporal
/// and spacetime slots are the absent ones.
pub type OpticsContext =
    Context<Data<OpticsDatum>, NoSpace<FloatType>, NoTime, NoSpaceTime<FloatType>>;

/// Contextoid id: the illuminating wavelength, in nanometres.
const WAVELENGTH_NM: ContextoidId = 1;
/// Contextoid id: the magnitude every principal permittivity takes, dimensionless.
const EPSILON_MAGNITUDE: ContextoidId = 2;
/// Contextoid id: the object periods to probe, in nanometres.
const SEPARATIONS_NM: ContextoidId = 3;

/// The optical world, one `Data` node per quantity, keyed by its contextoid id.
///
/// - The wavelength is 500 nm, green light.
/// - Every principal permittivity has magnitude 1. A real metamaterial has magnitudes that differ
///   per axis and vary with frequency. Holding them equal isolates the one thing this example is
///   about: the **sign** pattern, which is what the metric carries.
/// - The object periods run from 1000 nm down to 50 nm.
pub fn optics_world() -> Result<OpticsContext, ContextIndexError> {
    let facts = [
        (
            WAVELENGTH_NM,
            OpticsDatum::Magnitude(const_scalar_from_int!(FloatType, 500)),
        ),
        (
            EPSILON_MAGNITUDE,
            OpticsDatum::Magnitude(const_scalar_from_float!(FloatType, 1.0)),
        ),
        (
            SEPARATIONS_NM,
            OpticsDatum::Periods(vec![
                const_scalar_from_int!(FloatType, 1000),
                const_scalar_from_int!(FloatType, 750),
                const_scalar_from_int!(FloatType, 500),
                const_scalar_from_int!(FloatType, 400),
                const_scalar_from_int!(FloatType, 300),
                const_scalar_from_int!(FloatType, 200),
                const_scalar_from_int!(FloatType, 100),
                const_scalar_from_int!(FloatType, 50),
            ]),
        ),
    ];
    let mut world = Context::with_capacity(1, "hyperlens optics", facts.len());
    for (id, fact) in facts {
        world.add_node(Contextoid::new(
            id,
            ContextoidType::Datoid(Data::new(id, fact)),
        ))?;
    }
    Ok(world)
}

/// The illuminating wavelength, in nanometres.
pub fn wavelength_nm(world: &OpticsContext) -> Result<FloatType, ContextIndexError> {
    magnitude(world, WAVELENGTH_NM)
}

/// The magnitude every principal permittivity takes.
pub fn epsilon_magnitude(world: &OpticsContext) -> Result<FloatType, ContextIndexError> {
    magnitude(world, EPSILON_MAGNITUDE)
}

/// The object periods to probe, in nanometres, from coarse to fine.
pub fn separations_nm(world: &OpticsContext) -> Result<Vec<FloatType>, ContextIndexError> {
    match datum(world, SEPARATIONS_NM)? {
        OpticsDatum::Periods(periods) => Ok(periods),
        OpticsDatum::Magnitude(_) => Err(ContextIndexError::new(format!(
            "contextoid id {SEPARATIONS_NM} holds a magnitude, not periods"
        ))),
    }
}

/// A single magnitude out of the optical world.
fn magnitude(world: &OpticsContext, id: ContextoidId) -> Result<FloatType, ContextIndexError> {
    match datum(world, id)? {
        OpticsDatum::Magnitude(value) => Ok(value),
        OpticsDatum::Periods(_) => Err(ContextIndexError::new(format!(
            "contextoid id {id} holds periods, not a magnitude"
        ))),
    }
}

/// One fact out of the optical world.
fn datum(world: &OpticsContext, id: ContextoidId) -> Result<OpticsDatum, ContextIndexError> {
    world
        .get_data_by_id(id)
        .ok_or_else(|| ContextIndexError::new(format!("no optics datum with contextoid id {id}")))
}

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

/// The principal permittivity along one axis: `epsilon_magnitude`, its sign read from the metric.
///
/// This is the whole of the example's claim: the material is described by a metric signature, and
/// the optics below reads the signature rather than a hand-written sign. Swapping the metric
/// swaps the physics.
pub fn permittivity(metric: &Metric, axis: usize, epsilon_magnitude: FloatType) -> FloatType {
    match metric.sign_of_sq(axis) {
        sign if sign > 0 => epsilon_magnitude,
        sign if sign < 0 => -epsilon_magnitude,
        _ => ZERO,
    }
}

// =============================================================================
// Dispersion
// =============================================================================

/// The free-space wavenumber `k₀ = 2π/λ`, in radians per nanometre.
pub fn free_space_wavenumber(wavelength_nm: FloatType) -> FloatType {
    TWO * FloatType::pi() / wavelength_nm
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
pub fn squared_out_of_plane_wavenumber(
    metric: &Metric,
    wavelength_nm: FloatType,
    epsilon_magnitude: FloatType,
    separation_nm: FloatType,
) -> FloatType {
    let epsilon_x = permittivity(metric, AXIS_X, epsilon_magnitude);
    let epsilon_z = permittivity(metric, AXIS_Z, epsilon_magnitude);

    let k_0 = free_space_wavenumber(wavelength_nm);
    let k_x = spatial_frequency(separation_nm);

    epsilon_x * (k_0 * k_0 - k_x * k_x / epsilon_z)
}
