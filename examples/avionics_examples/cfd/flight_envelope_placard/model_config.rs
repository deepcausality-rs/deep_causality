/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Configuration construction (the "what"): where the test matrix lives, the placard world the
//! per-point physics reads, and the fitted shock model and atmosphere rows built from it.
//! Execution stays in `model`; the gate placards in `constants`.

use crate::FloatType;
use deep_causality_cfd::FittedNormalShock;
use deep_causality_context::{
    Context, Contextoid, ContextoidId, ContextoidType, ContextuableGraph, Data, NoSpace,
    NoSpaceTime, NoTime,
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

/// One atmosphere row `(altitude m, n_tot m⁻³, T K, a m/s)`.
pub type AtmosphereRow = (FloatType, FloatType, FloatType, FloatType);

/// The `.prepare` rig the sweep shares across grid points: the shock model, the world it was built
/// from, and the world's atmosphere rows, read once.
pub type PlacardRig = (
    FittedNormalShock<FloatType>,
    PlacardContext,
    Vec<AtmosphereRow>,
);

// Atmosphere profiles: U.S. Standard Atmosphere, 1976 (NOAA-S/T 76-1562): temperature, speed of
// sound, and density at the tabulated geometric altitudes, with density converted to number density
// through the 28.97 amu mean molecular mass ([`AIR_MEAN_MOLECULAR_MASS_KG`]). Same source and shape
// as the blackout examples' table, extended down to sea level for this envelope. The study
// interpolates linearly between rows; the README states what that costs.

/// Contextoid id (profile): geometric altitude, m, ascending.
pub const ALTITUDE_M: ContextoidId = 1;
/// Contextoid id (profile): total number density, m⁻³.
pub const NUMBER_DENSITY: ContextoidId = 2;
/// Contextoid id (profile): static temperature, K.
pub const TEMPERATURE_K: ContextoidId = 3;
/// Contextoid id (profile): speed of sound, m/s.
pub const SOUND_SPEED_MS: ContextoidId = 4;

/// Contextoid id (scalar): ratio of specific heats for calorically perfect air. Exact for a
/// diatomic ideal gas with frozen vibration; the standard value below the dissociation regime
/// (Anderson, *Modern Compressible Flow*, 3rd ed., ch. 1). The hottest grid point here reaches a
/// stagnation temperature near 1500 K, where vibrational excitation has begun but dissociation has
/// not, so the perfect-gas value is the honest effective gamma for this envelope. The README states
/// where the approximation is crude.
pub const GAMMA: ContextoidId = 5;
/// Contextoid id (scalar): mean molecular mass of air, kg (28.97 amu). Converts the atmosphere's
/// number density into mass density; the same value the blackout examples carry.
pub const AIR_MEAN_MOLECULAR_MASS_KG: ContextoidId = 6;
/// Contextoid id (scalar): Sutton-Graves stagnation-heating constant for air, kg^0.5·m⁻¹, in
/// `q̇ = k·√(ρ_∞/R_n)·V³` (Sutton, K. and Graves, R. A., "A General Stagnation-Point
/// Convective-Heating Equation for Arbitrary Gas Mixtures", NASA TR R-376, 1971). The same constant
/// the blackout examples' load stage carries. The correlation is calibrated for blunt-body entry
/// speeds; at the low-supersonic end of this grid its numbers are small and serve as a trend
/// column, not a thermal-protection sizing input.
pub const SUTTON_GRAVES_K: ContextoidId = 7;
/// Contextoid id (scalar): nose radius, m: a stated example constant for a blunt demonstrator
/// forebody. Sets the `√(1/R_n)` scale of the heating column; half a meter is a round, plausible
/// leading-body radius for a supersonic testbed and is not tied to any specific vehicle.
pub const NOSE_RADIUS_M: ContextoidId = 8;

/// Build the placard world, each quantity keyed by its contextoid id. The specification values are
/// exact `f64` literals, lifted losslessly into the working `FloatType`.
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
    let facts = [
        (ALTITUDE_M, tabulated(|row| row.0)),
        (NUMBER_DENSITY, tabulated(|row| row.1)),
        (TEMPERATURE_K, tabulated(|row| row.2)),
        (SOUND_SPEED_MS, tabulated(|row| row.3)),
        (GAMMA, PlacardDatum::Scalar(lift(1.4))),
        (
            AIR_MEAN_MOLECULAR_MASS_KG,
            PlacardDatum::Scalar(lift(4.81e-26)),
        ),
        (SUTTON_GRAVES_K, PlacardDatum::Scalar(lift(1.7415e-4))),
        (NOSE_RADIUS_M, PlacardDatum::Scalar(lift(0.5))),
    ];
    let mut context = Context::with_capacity(1, "placard world", facts.len());
    for (id, datum) in facts {
        context
            .add_node(Contextoid::new(
                id,
                ContextoidType::Datoid(Data::new(id, datum)),
            ))
            .map_err(|e| {
                PhysicsError::CalculationError(format!(
                    "placard world rejected contextoid {id}: {e}"
                ))
            })?;
    }
    Ok(context)
}

/// Read the `Data` contextoid carrying `id` out of the placard world.
fn datum(world: &PlacardContext, id: ContextoidId) -> Result<PlacardDatum, PhysicsError> {
    world.get_data_by_id(id).ok_or_else(|| {
        PhysicsError::CalculationError(format!(
            "placard world holds no Datoid with contextoid id {id}"
        ))
    })
}

/// Read one atmosphere profile out of the placard world.
pub fn profile(world: &PlacardContext, id: ContextoidId) -> Result<Vec<FloatType>, PhysicsError> {
    match datum(world, id)? {
        PlacardDatum::Profile(values) => Ok(values),
        PlacardDatum::Scalar(_) => Err(PhysicsError::CalculationError(format!(
            "placard world contextoid {id} is a scalar, not a profile"
        ))),
    }
}

/// Read one scalar out of the placard world.
pub fn scalar(world: &PlacardContext, id: ContextoidId) -> Result<FloatType, PhysicsError> {
    match datum(world, id)? {
        PlacardDatum::Scalar(value) => Ok(value),
        PlacardDatum::Profile(_) => Err(PhysicsError::CalculationError(format!(
            "placard world contextoid {id} is a profile, not a scalar"
        ))),
    }
}

/// The atmosphere rows of the placard world, ascending in altitude. Profiles of unequal length are
/// an error.
pub fn atmosphere_rows(world: &PlacardContext) -> Result<Vec<AtmosphereRow>, PhysicsError> {
    let altitude = profile(world, ALTITUDE_M)?;
    let density = profile(world, NUMBER_DENSITY)?;
    let temperature = profile(world, TEMPERATURE_K)?;
    let sound = profile(world, SOUND_SPEED_MS)?;
    let rows = altitude.len();
    if [density.len(), temperature.len(), sound.len()]
        .iter()
        .any(|&len| len != rows)
    {
        return Err(PhysicsError::CalculationError(
            "the placard world's atmosphere profiles differ in length".into(),
        ));
    }
    Ok(altitude
        .into_iter()
        .zip(density)
        .zip(temperature)
        .zip(sound)
        .map(|(((a, n), t), c)| (a, n, t, c))
        .collect())
}

/// The `.prepare` rig: the placard world, the exact-Rankine-Hugoniot shock model at the world's
/// effective gamma, and the world's atmosphere rows, read and checked once for the whole sweep.
pub fn placard_rig() -> Result<PlacardRig, PhysicsError> {
    let world = placard_world()?;
    let shock = FittedNormalShock::<FloatType>::new(scalar(&world, GAMMA)?)?;
    let atmosphere = atmosphere_rows(&world)?;
    Ok((shock, world, atmosphere))
}
