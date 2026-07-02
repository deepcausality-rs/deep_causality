/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Every tuned number in the corridor lives here: the carrier setup, the flight physics, the
//! navigation budget, the safety envelope, the branch study, and the three flight stations.
//!
//! **Model labels.** Each simplification is stated where its number is defined; the largest ones
//! up front:
//!
//! * The carrier flow is the *incompressible* QTT rollout in nondimensional units (`U_inf = 1`,
//!   box `2π`). `T_tr` is a recovery-temperature reconstruction over a mandatory Rankine-Hugoniot
//!   jump, not a true post-shock thermodynamic path.
//! * Ionization is driven at the Park two-temperature rate-controlling temperature
//!   `Tₐ = √(T_tr·T_ve)`, with `T_ve` relaxed over one sheath residence time on the
//!   Millikan-White clock (the parcel-renewal picture). This is the calibrated closure that lands
//!   the RAM-C II peak within the production chemistry spread; the peak-station gate compares
//!   against the flight anchor directly.
//! * The sheath is renewed each step (fresh parcels, one residence time of chemistry), which is
//!   what lets the exit clear physically. The *carried wake* fraction still has no recombination
//!   channel (forward Park rate only); see [`EXIT`].
//! * Bank enters the 2-D carrier as the projected forebody radius, the dominant flow-side effect
//!   of banking that this carrier can express.
//! * Dead reckoning drifts through the real INS mechanism: the accelerometer bias corrupts the
//!   sensed specific force. The bias magnitude is exaggerated so the t² signature is visible over
//!   the compressed 0.16 s dwell; a flight-grade 10 µg bias writes the same signature over a real
//!   20 s blackout.
//! * The sensed heating is the carrier's Brinkman wall heat-flux integral, rescaled by one
//!   calibration constant to a RAM-C-like MW/m² magnitude.
//! * Leg time is carrier time (`dt = 4 ms` per step), so navigation drifts are small; the gates
//!   compare them relatively, and the mechanism scales unchanged to a flight-length dwell.

// ── Carrier (the Gap-1/Gap-2 verification setup, reused verbatim)

/// Grid mode count; the box is `2^L × 2^L`.
pub const L: usize = 5;
/// Bond cap of the tensor-train round policy, i.e. the compression ceiling.
pub const CAP: usize = 16;
/// Kinematic viscosity of the incompressible carrier.
pub const NU: f64 = 0.05;
/// Solver time step (also the coupling and navigation step).
pub const DT: f64 = 0.004;
/// Brinkman penalization parameter; small means a hard wall.
pub const ETA: f64 = 0.016;
/// Free-stream speed of the nondimensional carrier.
pub const U_INF: f64 = 1.0;
/// Body-mask smoothing width in cells.
pub const SMOOTH_CELLS: f64 = 2.0;
/// Diffusivity transporting the carried ionization fraction.
pub const SCALAR_KAPPA: f64 = 0.05;

// ── Flight physics

/// Frozen-mixture specific heat at constant pressure, J·kg⁻¹·K⁻¹.
pub const C_P: f64 = 1004.0;
/// GPS L1 as an angular frequency: `ω = 2π · 1.57542 GHz`.
pub const COMMS_BAND_RAD_S: f64 = 9.899e9;
/// Vehicle characteristic length for the Knudsen number (RAM-C-like forebody scale, m).
pub const L_CHAR: f64 = 0.3;
/// The ④ channel's ballistic bundle `C_d·A/m`, scaled so the carrier's dynamic pressure carries a
/// RAM-C-like `~30 m/s²` drag into the navigation predict.
pub const CDA_OVER_M: f64 = 30.0;
/// Reference density of the ④ dynamic-pressure rescaling.
pub const RHO_REF: f64 = 1.0;
/// Standard gravity, m·s⁻².
pub const G0: f64 = 9.80665;

// ── Park two-temperature closure (the calibrated RAM-C recipe)

/// Reduced mass of the dominant relaxing collision pair, amu (N₂-N₂).
pub const REDUCED_MASS_AMU: f64 = 7.0;
/// Characteristic vibrational temperature of N₂, K.
pub const THETA_VIB: f64 = 3393.0;
/// Sheath residence time `t_res = standoff/u₂`: the RAM-C 7.6 mm standoff over the post-shock
/// speed `u₂ ≈ 376 m/s` at the Mach-25 station (reacting `γ_eff = 1.1`).
pub const RESIDENCE_TIME_S: f64 = 2.0e-5;

// ── Sensed loads

/// Calibration from the carrier's (nondimensional) Brinkman wall heat-flux magnitude to W·m⁻², at
/// a RAM-C-like MW/m² scale.
pub const Q_CALIBRATION: f64 = 2.0;

// ── Reference anchors

/// The RAM-C II ~61 km peak electron density anchor, m⁻³. With the two-temperature controller the
/// peak-station gate compares against this flight anchor directly (a 5x band).
pub const RAMC_NE_REFERENCE: f64 = 1.0e19;

// ── Navigation

/// Initial true position (a bound LEO-class state, m).
pub const TRUTH_R0: [f64; 3] = [7.0e6, 1.0e6, 2.0e6];
/// Initial true velocity (m/s).
pub const TRUTH_V0: [f64; 3] = [-1.0e3, 6.5e3, 3.0e3];
/// Initial INS error before the first fix folds (m per axis).
pub const NAV_INIT_ERR: [f64; 3] = [50.0, -30.0, 20.0];
/// Initial position uncertainty (50 m 1σ), m².
pub const P0_VAR: f64 = 2500.0;
/// ESKF process-noise diagonal.
pub const PROCESS_NOISE: f64 = 1.0e-4;
/// GNSS fix variance (5 m 1σ), m².
pub const GNSS_VAR: f64 = 25.0;
/// Through-plasma optical fix variance (50 m 1σ), m². No optical fix is published in this
/// corridor, so the budget is carried but unused.
pub const OPTICAL_VAR: f64 = 2500.0;
/// Accelerometer bias, m·s⁻² per axis: the dead-reckoning drift driver the IMU folds into the
/// sensed specific force. Exaggerated for the compressed dwell (see the module labels).
pub const IMU_ACCEL_BIAS: [f64; 3] = [0.6, -0.4, 0.3];
/// Gyro bias (unused by the translational corridor; carried for the IMU's grade).
pub const IMU_GYRO_BIAS: [f64; 3] = [0.0, 0.0, 0.0];

// ── Safety envelope + guidance

/// Heat-flux certification ceiling, W·m⁻² (margin over the ~4 MW/m² peak).
pub const MAX_HEAT_FLUX: f64 = 2.0e7;
/// G-load ceiling.
pub const MAX_G_LOAD: f64 = 15.0;
/// Bank-angle magnitude cap, rad.
pub const MAX_BANK_RAD: f64 = 0.5;
/// Guidance gain: bank command proportional to sensed heating. Deliberately aggressive, so the
/// cybernetic gate visibly bounds the command at the peak station.
pub const GUIDANCE_GAIN: f64 = 2.3e-7;

// ── Branch study

/// Steps each counterfactual branch continues past the shared blackout onset.
pub const BRANCH_STEPS: usize = 20;
/// Candidate bank angles for the branch study (degrees).
pub const BANK_ANGLES_DEG: [f64; 3] = [0.0, 30.0, 60.0];

// ── Flight stations

/// One flight station of the corridor: the reacting and rarefaction constants a leg marches under.
pub struct FlightCondition {
    pub name: &'static str,
    pub mach: f64,
    pub t_inf: f64,
    /// Effective ratio of specific heats through the shock. Reacting air absorbs energy into
    /// dissociation, so the peak station uses `γ_eff = 1.1` (the calibrated recipe); the weaker
    /// stations use the perfect-gas 1.4.
    pub gamma_eff: f64,
    /// Post-shock sheath heavy-particle number density (m⁻³); the plasma scales with it.
    pub n_tot: f64,
    /// Post-shock pressure (atm); the Millikan-White relaxation clock scales with `1/p`.
    pub pressure_atm: f64,
    /// Mean free path at the station (m); the Knudsen and regime driver.
    pub mean_free_path: f64,
    pub steps: usize,
    /// Projected forebody radius as a fraction of the box. Bank enters here.
    pub radius_frac: f64,
}

/// Approach (~90 km). Thin air: the chemistry is frozen (`τ_ion ≫` dwell), the Knudsen number
/// sits in the slip band, the electron density stays below the L1 cutoff, and GNSS aids the INS.
pub const APPROACH: FlightCondition = FlightCondition {
    name: "approach_90km",
    mach: 8.0,
    t_inf: 250.0,
    gamma_eff: 1.4,
    n_tot: 1.0e19,
    pressure_atm: 4.6e-6,
    mean_free_path: 0.025,
    steps: 40,
    radius_frac: 0.18,
};

/// Peak heating (61 km, the RAM-C II Mach-25 station), on the calibrated recipe: reacting
/// `γ_eff = 1.1` lands `T₂ ≈ 8,044 K`, and the sheath density is the post-shock
/// `n₂ = n_∞·(ρ₂/ρ₁) ≈ 1.3e21 × 20.3`. Ionization, driven at the lagged controller `Tₐ`, crosses
/// the L1 cutoff within a few steps: blackout onset, dead reckoning, the branch study.
pub const PEAK: FlightCondition = FlightCondition {
    name: "peak_61km",
    mach: 25.0,
    t_inf: 250.0,
    gamma_eff: 1.1,
    n_tot: 2.645e22,
    pressure_atm: 2.9e-2,
    mean_free_path: 2.0e-4,
    steps: 40,
    radius_frac: 0.18,
};

/// Exit (~30 km, decelerated to Mach 7). Under sheath renewal the plasma layer is refreshed each
/// step by post-shock air whose weak shock (`T₂ ≈ 3,000 K`) barely ionizes over one residence
/// time, so the electron density collapses and the first fix ends the blackout — the physical
/// exit mechanism (the hot sheath convects away and nothing replenishes it). The *carried* wake
/// fraction still has no recombination channel (the LER holds only the forward Park rate); the
/// renewal makes that a wake property, not the signal path's.
pub const EXIT: FlightCondition = FlightCondition {
    name: "exit_30km",
    mach: 7.0,
    t_inf: 226.0,
    gamma_eff: 1.4,
    n_tot: 2.0e24,
    pressure_atm: 6.5e-1,
    mean_free_path: 3.0e-6,
    steps: 40,
    radius_frac: 0.18,
};
