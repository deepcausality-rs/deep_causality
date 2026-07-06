/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! GM recovery pipeline expressed as a `CausalFlow` chain.
//!
//! Five stages, each a plain `Value -> Result<U, CausalityError>`, composed with `.try_step`:
//!
//! ```text
//! load ──► align ──► pair ──► invert ──► aggregate
//! ```
//!
//! 1. **load**      Load `.clk` and `.sp3` for one satellite via [`DataManager`].
//! 2. **align**     10th-order Lagrange interpolation aligns clock and orbit
//!    data into a [`SpaceTimeCoordinate`] vector with the
//!    IGS-removed periodic relativistic correction restored.
//! 3. **pair**      Form coordinate pairs with sufficient radial separation.
//! 4. **invert**    Apply [`solve_gm_analytical`] to each pair.
//! 5. **aggregate** MAD-filter outliers, then reduce to mean / median / σ
//!    and compare to the JGM-3 reference value.
//!
//! Generic over the precision type `R` (`f64`, `Float106`, …) so the same
//! pipeline composes at any precision the framework supports. Because each
//! stage returns a `Result`, `CausalFlow::try_step` unwraps the value and
//! short-circuits the error channel; no stage touches `CausalEffect`.

use chronometric_examples::{ClockData, OrbitData};
use core::fmt::Debug;
use deep_causality_core::{CausalityError, CausalityErrorEnum};
use deep_causality_num::{FromPrimitive, RealField};
use deep_causality_physics::{
    CentralBody, EARTH_GM, EARTH_J2, EARTH_MASS_KG, EARTH_RADIUS_EQUATORIAL,
    NEWTONIAN_CONSTANT_OF_GRAVITATION, SpaceTimeCoordinate, solve_gm_analytical,
};

use chronometric_examples::data_manager::DataManager;
use chronometric_examples::proces_utils::{apply_mad_filter, interpolate_space_time};

/// Minimum radial separation between paired coordinates (meters).
/// Below this, the denominator $1/r_a - 1/r_b$ is numerically unstable.
const MIN_RADIAL_SEPARATION_M: f64 = 1_000.0;

/// Sliding-window pair construction: index offset between paired coordinates.
/// At 30 s clock cadence this corresponds to ~50 minutes — enough orbital
/// phase variation on E14 (eccentricity ~0.16) to give meaningful radial
/// separation while keeping the kinematic state self-consistent.
const PAIR_WINDOW_SIZE: usize = 100;

/// Sliding-window step between successive pairs.
const PAIR_STEP_SIZE: usize = 1;

/// Cap on the number of pairs evaluated, to bound example runtime.
const MAX_PAIRS: usize = 131_072;

/// MAD outlier rejection threshold (in MAD-derived sigma).
const MAD_OUTLIER_SIGMA: f64 = 3.0;

#[derive(Debug, Clone, Default)]
pub struct DatasetInputs {
    /// Directory holding the `<dataset>.clk` and `<dataset>.sp3` pairs.
    pub data_dir: String,
    /// Dataset stems to load and concatenate (e.g. `["gbm18770", "gbm18771"]`).
    /// Loaded clocks/orbits are sorted by timestamp before alignment, so
    /// passing them in any order is safe.
    pub datasets: Vec<String>,
    /// Satellite identifier (e.g. `"E14"`).
    pub sat_id: String,
}

#[derive(Debug, Clone, Default)]
pub struct LoadedDataset<R: RealField> {
    pub clocks: Vec<ClockData<R>>,
    pub orbits: Vec<OrbitData<R>>,
}

#[derive(Debug, Clone, Default)]
pub struct CoordinateSet<R: RealField> {
    pub coords: Vec<SpaceTimeCoordinate<R>>,
}

#[derive(Debug, Clone, Default)]
pub struct PairSet<R: RealField> {
    pub pairs: Vec<(SpaceTimeCoordinate<R>, SpaceTimeCoordinate<R>)>,
}

#[derive(Debug, Clone, Default)]
pub struct GmEstimates<R: RealField> {
    pub estimates: Vec<R>,
}

#[derive(Debug, Clone, Default)]
pub struct GmReport<R: RealField> {
    pub n_pairs: usize,
    pub n_after_mad: usize,

    // Recovered (mean) and reference values for the geocentric gravitational
    // parameter $GM$ (m³/s²).
    pub mean_gm: R,
    pub median_gm: R,
    pub std_gm: R,
    pub reference_gm: R,
    pub gm_relative_error: R,

    // Derived planetary mass $M = GM / G$ (kg). Recovered from the same
    // pipeline by dividing through Newton's constant; published reference is
    // the IERS 2010 derived Earth mass.
    pub recovered_mass_kg: R,
    pub reference_mass_kg: R,
    pub mass_relative_error: R,
}

// ---------------------------------------------------------------------------
// Stages — each `Value -> Result<U, CausalityError>`
// ---------------------------------------------------------------------------

pub fn stage_load<R>(inputs: DatasetInputs) -> Result<LoadedDataset<R>, CausalityError>
where
    R: RealField + From<f64> + Default + Debug,
{
    if inputs.datasets.is_empty() {
        return Err(err("stage_load: no datasets specified"));
    }

    let dm = DataManager;
    let mut all_clocks = Vec::new();
    let mut all_orbits = Vec::new();

    for dataset in &inputs.datasets {
        let clk_path = format!("{}/{}.clk", inputs.data_dir, dataset);
        let sp3_path = format!("{}/{}.sp3", inputs.data_dir, dataset);
        match dm.load_gnss_single_satellite::<R, _>(&clk_path, &sp3_path, &inputs.sat_id) {
            Ok((clocks, orbits)) => {
                all_clocks.extend(clocks);
                all_orbits.extend(orbits);
            }
            Err(e) => {
                return Err(err(&format!(
                    "stage_load: failed to load dataset '{}' for {}: {}",
                    dataset, inputs.sat_id, e
                )));
            }
        }
    }

    if all_clocks.is_empty() || all_orbits.is_empty() {
        return Err(err(&format!(
            "stage_load: empty data for satellite {} across {} dataset(s)",
            inputs.sat_id,
            inputs.datasets.len()
        )));
    }

    // Defensive sort by timestamp — the Lagrange interpolation in stage_align
    // requires monotonically increasing time. If the caller passed datasets
    // in chronological order this sort is a no-op; otherwise it normalizes.
    all_clocks.sort_by_key(|c| c.timestamp());
    all_orbits.sort_by_key(|o| o.timestamp());

    Ok(LoadedDataset {
        clocks: all_clocks,
        orbits: all_orbits,
    })
}

pub fn stage_align<R>(dataset: LoadedDataset<R>) -> Result<CoordinateSet<R>, CausalityError>
where
    R: RealField + From<f64> + Into<f64> + FromPrimitive + Default + Debug,
{
    let coords = interpolate_space_time(&dataset.clocks, &dataset.orbits);
    if coords.len() < 2 {
        return Err(err(&format!(
            "stage_align: only {} aligned coordinate(s); need at least 2",
            coords.len()
        )));
    }
    Ok(CoordinateSet { coords })
}

pub fn stage_pair<R>(set: CoordinateSet<R>) -> Result<PairSet<R>, CausalityError>
where
    R: RealField + From<f64> + Default + Debug,
{
    let coords = &set.coords;
    if coords.len() <= PAIR_WINDOW_SIZE {
        return Err(err(&format!(
            "stage_pair: only {} coordinate(s); need more than the window size {}",
            coords.len(),
            PAIR_WINDOW_SIZE
        )));
    }

    // Sliding-window pairing: pair coord[i] with coord[i + WINDOW] sliding by
    // STEP. Matches the chronometric-geodesy convention and produces diverse
    // pair geometries spanning the full dataset rather than anchoring every
    // pair to the first few coordinates (the all-pairs failure mode at
    // multi-day scales).
    let min_sep = R::from(MIN_RADIAL_SEPARATION_M);
    let mut pairs = Vec::with_capacity(coords.len() / PAIR_STEP_SIZE);
    let mut i = 0usize;
    while i + PAIR_WINDOW_SIZE < coords.len() && pairs.len() < MAX_PAIRS {
        let j = i + PAIR_WINDOW_SIZE;
        let dr = (coords[i].r_m - coords[j].r_m).abs();
        if dr >= min_sep {
            pairs.push((coords[i], coords[j]));
        }
        i += PAIR_STEP_SIZE;
    }

    if pairs.is_empty() {
        return Err(err(
            "stage_pair: no coordinate pairs with sufficient radial separation",
        ));
    }
    Ok(PairSet { pairs })
}

pub fn stage_solve_gm<R>(set: PairSet<R>) -> Result<GmEstimates<R>, CausalityError>
where
    R: RealField + From<f64> + Default + Debug,
{
    let body = CentralBody::<R>::new(
        R::from(EARTH_GM),
        R::from(EARTH_RADIUS_EQUATORIAL),
        R::from(EARTH_J2),
    );
    let upper_bound = R::from(1.0e16);

    let mut estimates = Vec::with_capacity(set.pairs.len());
    for (a, b) in &set.pairs {
        let effect = solve_gm_analytical(a, b, &body);
        if let Some(gm) = effect.into_value() {
            // Drop NaNs and major outliers that come from poorly conditioned
            // input pairs (near-coincident orbital states, IGS sentinel artifacts),
            // not from the inversion itself. MAD filter in stage_aggregate
            // handles the statistical outliers.
            if gm.abs() < upper_bound {
                estimates.push(gm);
            }
        }
    }

    if estimates.is_empty() {
        return Err(err("stage_invert: no successful inversions"));
    }
    Ok(GmEstimates { estimates })
}

pub fn stage_aggregate<R>(est: GmEstimates<R>) -> Result<GmReport<R>, CausalityError>
where
    R: RealField + From<f64> + Default + Debug,
{
    let outlier_sigma = R::from(MAD_OUTLIER_SIGMA);
    let filtered = apply_mad_filter(&est.estimates, outlier_sigma);
    let n = filtered.len();
    if n == 0 {
        return Err(err("stage_aggregate: MAD filter rejected all estimates"));
    }

    let n_r = R::from(n as f64);
    let mut sum = R::zero();
    for v in &filtered {
        sum += *v;
    }
    let mean = sum / n_r;

    let mut sq = R::zero();
    for v in &filtered {
        let d = *v - mean;
        sq += d * d;
    }
    let std_dev = (sq / n_r).sqrt();

    let mut sorted = filtered.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
    let median = if n.is_multiple_of(2) {
        let two = R::from(2.0);
        (sorted[n / 2 - 1] + sorted[n / 2]) / two
    } else {
        sorted[n / 2]
    };

    let reference_gm = R::from(EARTH_GM);
    let gm_rel_err = ((mean - reference_gm) / reference_gm).abs();

    // Derive planetary mass: M = GM / G. The framework recovers GM from clock
    // time-dilation measurements; dividing through Newton's gravitational
    // constant yields Earth's mass in kg — "weighing the planet by clock."
    let g = R::from(NEWTONIAN_CONSTANT_OF_GRAVITATION);
    let recovered_mass = mean / g;
    let reference_mass = R::from(EARTH_MASS_KG);
    let mass_rel_err = ((recovered_mass - reference_mass) / reference_mass).abs();

    Ok(GmReport {
        n_pairs: est.estimates.len(),
        n_after_mad: n,
        mean_gm: mean,
        median_gm: median,
        std_gm: std_dev,
        reference_gm,
        gm_relative_error: gm_rel_err,
        recovered_mass_kg: recovered_mass,
        reference_mass_kg: reference_mass,
        mass_relative_error: mass_rel_err,
    })
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn err(msg: &str) -> CausalityError {
    CausalityError::new(CausalityErrorEnum::Custom(msg.into()))
}
