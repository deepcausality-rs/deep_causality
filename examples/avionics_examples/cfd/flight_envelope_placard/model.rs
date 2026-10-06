/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Domain logic (the "how" of the physics): the atmosphere interpolation and the per-point
//! placard computation, both reading the placard world of the `.prepare` rig. Configuration
//! (matrix location, the placard world, shock model) lives in `model_config`; the gate placards in
//! `constants`.

use crate::FloatType;
use crate::model_config::{
    AIR_MEAN_MOLECULAR_MASS_KG, ALTITUDE_M, GAMMA, NOSE_RADIUS_M, NUMBER_DENSITY, PlacardContext,
    PlacardRig, SOUND_SPEED_MS, SUTTON_GRAVES_K, TEMPERATURE_K, profile, scalar,
};
use deep_causality_algebra::Real;
use deep_causality_cfd::{FromTableRow, GateSeq, PhysicsError, StudyView, TableRow};
use deep_causality_num::lift;

/// One Mach-altitude test point: the case axis, read from the matrix by column name.
#[derive(Debug, Clone)]
pub struct FlightPoint {
    pub mach: FloatType,
    pub alt_km: FloatType,
}

impl TableRow for FlightPoint {
    type Scalar = FloatType;
    const SCHEMA: &'static [(&'static str, &'static str)] = &[("mach", "-"), ("alt", "km")];
    fn cells(&self) -> Vec<FloatType> {
        vec![self.mach, self.alt_km]
    }
}

impl FromTableRow for FlightPoint {
    fn from_cells(cells: &[FloatType]) -> Option<Self> {
        Some(Self {
            mach: cells[0],
            alt_km: cells[1],
        })
    }
}

/// One placard row for one grid point: the reduced result the map records and the gates read.
#[derive(Debug, Clone)]
pub struct PlacardRow {
    pub mach: FloatType,
    pub alt_km: FloatType,
    /// Dynamic pressure, kPa.
    pub q_kpa: FloatType,
    /// Post-shock stagnation temperature, K.
    pub t0_k: FloatType,
    /// Sutton-Graves stagnation-point heating, W/cm².
    pub qdot_w_cm2: FloatType,
}

impl TableRow for PlacardRow {
    type Scalar = FloatType;
    const SCHEMA: &'static [(&'static str, &'static str)] = &[
        ("mach", "-"),
        ("alt", "km"),
        ("q", "kPa"),
        ("t0_post_shock", "K"),
        ("qdot", "W/cm2"),
    ];
    fn cells(&self) -> Vec<FloatType> {
        vec![
            self.mach,
            self.alt_km,
            self.q_kpa,
            self.t0_k,
            self.qdot_w_cm2,
        ]
    }
}

/// The atmosphere rows `(altitude m, n_tot m⁻³, T K, a m/s)` of the placard world, ascending in
/// altitude. Profiles of unequal length are an error.
fn atmosphere_rows(
    world: &PlacardContext,
) -> Result<Vec<(FloatType, FloatType, FloatType, FloatType)>, PhysicsError> {
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

/// The freestream `(n_tot m⁻³, T K, a m/s)` at `alt_m`, linearly interpolated between the
/// world's atmosphere rows. Altitudes outside the table are an error naming the valid range.
pub fn atmosphere_at(
    world: &PlacardContext,
    alt_m: FloatType,
) -> Result<(FloatType, FloatType, FloatType), String> {
    let atmosphere = atmosphere_rows(world).map_err(|e| e.to_string())?;
    let (Some(&(floor, ..)), Some(&(ceiling, ..))) = (atmosphere.first(), atmosphere.last()) else {
        return Err("the placard world's atmosphere table is empty".into());
    };
    if alt_m < floor || alt_m > ceiling {
        return Err(format!(
            "altitude {:.1} km is outside the atmosphere table ({:.0} to {:.0} km); \
             fix the matrix row or extend the atmosphere in model_config::placard_world",
            alt_m / lift::<FloatType>(1000.0),
            floor / lift::<FloatType>(1000.0),
            ceiling / lift::<FloatType>(1000.0)
        ));
    }
    for pair in atmosphere.windows(2) {
        let (a0, n0, temp0, c0) = pair[0];
        let (a1, n1, temp1, c1) = pair[1];
        if alt_m <= a1 {
            let w = (alt_m - a0) / (a1 - a0);
            return Ok((
                n0 + w * (n1 - n0),
                temp0 + w * (temp1 - temp0),
                c0 + w * (c1 - c0),
            ));
        }
    }
    Err(format!(
        "altitude {:.1} km found no bracketing atmosphere rows (table not ascending?)",
        alt_m / lift::<FloatType>(1000.0)
    ))
}

/// One placard row for one grid point: `[mach, alt km, q kPa, T₀ K, q̇ W/cm²]`.
///
/// Above Mach 1 the stagnation temperature is taken through the exact Rankine-Hugoniot jump:
/// the post-shock static state `(T₂, u₂/u₁)` from the fitted shock, the post-shock Mach number
/// `M₂ = M₁·(u₂/u₁)·√(T₁/T₂)`, then the isentropic re-stagnation
/// `T₀ = T₂·(1 + (γ−1)/2·M₂²)`. For a calorically perfect gas this equals the freestream total
/// temperature (the shock is adiabatic), so the branch is exactly continuous at Mach 1, where
/// the shock-free isentropic form takes over.
pub fn placard_point(rig: &PlacardRig, point: &FlightPoint) -> Result<PlacardRow, PhysicsError> {
    let (shock, world) = rig;
    let (mach, alt_km) = (point.mach, point.alt_km);
    let here = format!("M {mach:.2} / {alt_km:.1} km");
    let (n_inf, t_inf, a_inf) = atmosphere_at(world, alt_km * lift::<FloatType>(1000.0))
        .map_err(|e| PhysicsError::CalculationError(format!("grid point {here}: {e}")))?;
    let rho_inf = n_inf * scalar(world, AIR_MEAN_MOLECULAR_MASS_KG)?;
    let v = mach * a_inf;
    let q_pa = lift::<FloatType>(0.5) * rho_inf * v * v;

    let half_gm1 = lift::<FloatType>(0.5) * (scalar(world, GAMMA)? - lift::<FloatType>(1.0));
    let t0_k = if mach >= lift::<FloatType>(1.0) {
        let post = shock.post_shock(t_inf, n_inf, mach).map_err(|e| {
            PhysicsError::CalculationError(format!(
                "grid point {here}: post-shock state failed: {e}"
            ))
        })?;
        let m2 = mach * post.u_ratio * Real::sqrt(t_inf / post.t2);
        post.t2 * (lift::<FloatType>(1.0) + half_gm1 * m2 * m2)
    } else {
        // No shock below Mach 1: the isentropic stagnation temperature, the exact shock-free
        // limit of the branch above.
        t_inf * (lift::<FloatType>(1.0) + half_gm1 * mach * mach)
    };

    let qdot_w_m2 = scalar(world, SUTTON_GRAVES_K)?
        * Real::sqrt(rho_inf / scalar(world, NOSE_RADIUS_M)?)
        * v
        * v
        * v;

    Ok(PlacardRow {
        mach,
        alt_km,
        q_kpa: q_pa / lift::<FloatType>(1000.0), // Pa -> kPa
        t0_k,
        qdot_w_cm2: qdot_w_m2 / lift::<FloatType>(1.0e4), // W/m² -> W/cm²
    })
}

// ── The placard gating sequence ───────────────────────────────────────────────────────────────

/// The placard's gating sequence: every grid point inside the q-max and stagnation-temperature
/// envelope, offenders named rather than averaged away.
pub fn placard_gates() -> GateSeq<PlacardRow> {
    GateSeq::new("flight envelope placard")
        .gate("q-max placard", gate_q_max)
        .gate("stagnation temperature", gate_stagnation_temperature)
}

/// Every point sits inside the dynamic-pressure placard; exceeding points are named.
pub fn gate_q_max(view: &StudyView<'_, PlacardRow>) -> (bool, String) {
    use crate::constants::Q_MAX_PLACARD_KPA;
    if view.rows().is_empty() {
        return (false, "no grid points computed (empty matrix)".into());
    }
    let q_max = lift::<FloatType>(Q_MAX_PLACARD_KPA);
    let offenders: Vec<String> = view
        .rows()
        .iter()
        .filter(|r| r.q_kpa > q_max)
        .map(|r| {
            format!(
                "q = {:.1} kPa at M {:.2} / {:.1} km",
                r.q_kpa, r.mach, r.alt_km
            )
        })
        .collect();
    if offenders.is_empty() {
        let peak = peak_by(view.rows(), |r| r.q_kpa);
        (
            true,
            format!(
                "max q = {:.1} kPa at M {:.2} / {:.1} km, inside the {Q_MAX_PLACARD_KPA:.0} kPa placard",
                peak.q_kpa, peak.mach, peak.alt_km
            ),
        )
    } else {
        (
            false,
            format!(
                "{} exceeds the {Q_MAX_PLACARD_KPA:.0} kPa placard",
                offenders.join("; ")
            ),
        )
    }
}

/// Every point sits inside the stagnation-temperature placard; exceeding points are named.
pub fn gate_stagnation_temperature(view: &StudyView<'_, PlacardRow>) -> (bool, String) {
    use crate::constants::T0_MAX_PLACARD_K;
    if view.rows().is_empty() {
        return (false, "no grid points computed (empty matrix)".into());
    }
    let t0_max = lift::<FloatType>(T0_MAX_PLACARD_K);
    let offenders: Vec<String> = view
        .rows()
        .iter()
        .filter(|r| r.t0_k > t0_max)
        .map(|r| {
            format!(
                "T0 = {:.1} K at M {:.2} / {:.1} km",
                r.t0_k, r.mach, r.alt_km
            )
        })
        .collect();
    if offenders.is_empty() {
        let peak = peak_by(view.rows(), |r| r.t0_k);
        (
            true,
            format!(
                "max T0 = {:.1} K at M {:.2} / {:.1} km, inside the {T0_MAX_PLACARD_K:.0} K placard",
                peak.t0_k, peak.mach, peak.alt_km
            ),
        )
    } else {
        (
            false,
            format!(
                "{} exceeds the {T0_MAX_PLACARD_K:.0} K placard",
                offenders.join("; ")
            ),
        )
    }
}

/// The row maximizing `key`. The gates guard against an empty row set before calling this (an
/// empty study fails visibly rather than reaching here), and the `split_first` seed makes that
/// invariant explicit instead of an opaque `rows[0]` index panic.
fn peak_by(rows: &[PlacardRow], key: impl Fn(&PlacardRow) -> FloatType) -> &PlacardRow {
    let (first, rest) = rows
        .split_first()
        .expect("peak_by requires a non-empty row set; the gates guard this");
    rest.iter()
        .fold(first, |best, r| if key(r) > key(best) { r } else { best })
}
