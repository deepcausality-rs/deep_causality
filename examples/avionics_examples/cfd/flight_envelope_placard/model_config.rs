/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Configuration construction (the "what"): where the test matrix lives, the placard world the
//! per-point physics reads, and the fitted shock model built from it. Execution stays in `model`;
//! the gate placards in `constants`.

use crate::FloatType;
use deep_causality_cfd::FittedNormalShock;
use deep_causality_context::{
    Context, Contextoid, ContextoidType, ContextuableGraph, Data, Datable, NoSpace, NoSpaceTime,
    NoTime,
};
use deep_causality_num::lift;
use deep_causality_physics::PhysicsError;
use std::path::PathBuf;

/// The Mach-altitude matrix: the recorded corridor by default, or a caller-supplied path (the
/// out-of-envelope demonstration passes its own file).
pub fn matrix_path() -> PathBuf {
    std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            avionics_examples::paths::manifest_dir()
                .join("cfd/flight_envelope_placard/mach_alt_matrix.csv")
        })
}

/// Where the placard table is written.
pub fn table_path() -> PathBuf {
    avionics_examples::paths::manifest_dir().join("cfd/flight_envelope_placard/placard_table.csv")
}

// ── The placard world

/// One quantity of the placard world: an atmosphere profile tabulated at ascending altitudes, or a
/// scalar.
#[derive(Debug, Clone, PartialEq)]
pub enum PlacardDatum {
    /// One value per atmosphere row, in ascending-altitude order.
    Profile(Vec<FloatType>),
    /// A single value.
    Scalar(FloatType),
}

impl Default for PlacardDatum {
    fn default() -> Self {
        PlacardDatum::Scalar(FloatType::default())
    }
}

/// The placard world, one `Data` contextoid per quantity. The atmosphere is keyed by altitude alone
/// (no latitude or longitude) and the study runs no clock, so the spatial, temporal and spacetime
/// slots are empty.
pub type PlacardContext =
    Context<Data<PlacardDatum>, NoSpace<FloatType>, NoTime, NoSpaceTime<FloatType>>;

/// The `.prepare` rig the sweep shares across grid points: the shock model and the world it was
/// built from.
pub type PlacardRig = (FittedNormalShock<FloatType>, PlacardContext);

// Atmosphere profiles: U.S. Standard Atmosphere, 1976 (NOAA-S/T 76-1562): temperature, speed of
// sound, and density at the tabulated geometric altitudes, with density converted to number density
// through the 28.97 amu mean molecular mass ([`AIR_MEAN_MOLECULAR_MASS_KG`]). Same source and shape
// as the blackout examples' table, extended down to sea level for this envelope. The study
// interpolates linearly between rows; the README states what that costs.

/// Node index (profile): geometric altitude, m, ascending.
pub const ALTITUDE_M: usize = 0;
/// Node index (profile): total number density, m⁻³.
pub const NUMBER_DENSITY: usize = 1;
/// Node index (profile): static temperature, K.
pub const TEMPERATURE_K: usize = 2;
/// Node index (profile): speed of sound, m/s.
pub const SOUND_SPEED_MS: usize = 3;

/// Node index (scalar): ratio of specific heats for calorically perfect air. Exact for a diatomic
/// ideal gas with frozen vibration; the standard value below the dissociation regime (Anderson,
/// *Modern Compressible Flow*, 3rd ed., ch. 1). The hottest grid point here reaches a stagnation
/// temperature near 1500 K, where vibrational excitation has begun but dissociation has not, so the
/// perfect-gas value is the honest effective gamma for this envelope. The README states where the
/// approximation is crude.
pub const GAMMA: usize = 4;
/// Node index (scalar): mean molecular mass of air, kg (28.97 amu). Converts the atmosphere's
/// number density into mass density; the same value the blackout examples carry.
pub const AIR_MEAN_MOLECULAR_MASS_KG: usize = 5;
/// Node index (scalar): Sutton-Graves stagnation-heating constant for air, kg^0.5·m⁻¹, in
/// `q̇ = k·√(ρ_∞/R_n)·V³` (Sutton, K. and Graves, R. A., "A General Stagnation-Point
/// Convective-Heating Equation for Arbitrary Gas Mixtures", NASA TR R-376, 1971). The same constant
/// the blackout examples' load stage carries. The correlation is calibrated for blunt-body entry
/// speeds; at the low-supersonic end of this grid its numbers are small and serve as a trend
/// column, not a thermal-protection sizing input.
pub const SUTTON_GRAVES_K: usize = 6;
/// Node index (scalar): nose radius, m: a stated example constant for a blunt demonstrator
/// forebody. Sets the `√(1/R_n)` scale of the heating column; half a meter is a round, plausible
/// leading-body radius for a supersonic testbed and is not tied to any specific vehicle.
pub const NOSE_RADIUS_M: usize = 7;

/// Build the placard world, added in node-index order: node `i` holds contextoid id `i + 1`. The
/// specification values are exact `f64` literals, lifted losslessly into the working `FloatType`.
pub fn placard_world() -> Result<PlacardContext, PhysicsError> {
    // (altitude m, n_tot m⁻³, T K, a m/s), ascending altitude.
    let atmosphere: [(f64, f64, f64, f64); 9] = [
        (0.0, 2.547e25, 288.15, 340.3),
        (5_000.0, 1.531e25, 255.68, 320.5),
        (10_000.0, 8.597e24, 223.25, 299.5),
        (15_000.0, 4.050e24, 216.65, 295.1),
        (20_000.0, 1.848e24, 216.65, 295.1),
        (25_000.0, 8.333e23, 221.55, 298.4),
        (30_000.0, 3.827e23, 226.51, 301.7),
        (40_000.0, 8.308e22, 250.35, 317.2),
        (47_000.0, 2.967e22, 270.65, 329.8),
    ];
    let tabulated = |column: fn(&(f64, f64, f64, f64)) -> f64| {
        PlacardDatum::Profile(atmosphere.iter().map(|row| lift(column(row))).collect())
    };
    let data = [
        tabulated(|row| row.0),                // ALTITUDE_M
        tabulated(|row| row.1),                // NUMBER_DENSITY
        tabulated(|row| row.2),                // TEMPERATURE_K
        tabulated(|row| row.3),                // SOUND_SPEED_MS
        PlacardDatum::Scalar(lift(1.4)),       // GAMMA
        PlacardDatum::Scalar(lift(4.81e-26)),  // AIR_MEAN_MOLECULAR_MASS_KG
        PlacardDatum::Scalar(lift(1.7415e-4)), // SUTTON_GRAVES_K
        PlacardDatum::Scalar(lift(0.5)),       // NOSE_RADIUS_M
    ];
    let mut context = Context::with_capacity(1, "placard world", data.len());
    for (id, datum) in (1..).zip(data) {
        context
            .add_node(Contextoid::new(
                id,
                ContextoidType::Datoid(Data::new(id, datum)),
            ))
            .map_err(|e| {
                PhysicsError::CalculationError(format!("placard world rejected node {id}: {e}"))
            })?;
    }
    Ok(context)
}

/// Read one `Data` contextoid's payload out of the placard world.
fn datum(world: &PlacardContext, index: usize) -> Result<PlacardDatum, PhysicsError> {
    world
        .get_node(index)
        .and_then(|node| node.vertex_type().dataoid())
        .map(Datable::get_data)
        .ok_or_else(|| {
            PhysicsError::CalculationError(format!(
                "placard world node {index} is absent or not a Datoid"
            ))
        })
}

/// Read one atmosphere profile out of the placard world.
pub fn profile(world: &PlacardContext, index: usize) -> Result<Vec<FloatType>, PhysicsError> {
    match datum(world, index)? {
        PlacardDatum::Profile(values) => Ok(values),
        PlacardDatum::Scalar(_) => Err(PhysicsError::CalculationError(format!(
            "placard world node {index} is a scalar, not a profile"
        ))),
    }
}

/// Read one scalar out of the placard world.
pub fn scalar(world: &PlacardContext, index: usize) -> Result<FloatType, PhysicsError> {
    match datum(world, index)? {
        PlacardDatum::Scalar(value) => Ok(value),
        PlacardDatum::Profile(_) => Err(PhysicsError::CalculationError(format!(
            "placard world node {index} is a profile, not a scalar"
        ))),
    }
}

/// The `.prepare` rig: the placard world, and the exact-Rankine-Hugoniot shock model at the world's
/// effective gamma.
pub fn placard_rig() -> Result<PlacardRig, PhysicsError> {
    let world = placard_world()?;
    let shock = FittedNormalShock::<FloatType>::new(scalar(&world, GAMMA)?)?;
    Ok((shock, world))
}
