/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The specification constants both blackout examples fly: the compressed compressible carrier,
//! the reference anchors, the baseline atmosphere, the flight physics, the calibrated Park
//! two-temperature closure, the navigation budget, and the safety envelope. Per-example knobs
//! (the bank sweep, the weather conditions, the gate thresholds) live in each example's own
//! `constants.rs`.
//!
//! **Model labels**, largest first:
//!
//! * The carrier is the 2-D **compressible** marcher on tensor trains: the exact Rankine-Hugoniot
//!   post-shock state (from the truth vehicle's Mach number through the atmosphere table) is
//!   enforced on the inflow strip, and the layer behind it is *evolved*. One coupled step marches
//!   one solver pseudo-time step toward the quasi-steady layer at that flight instant.
//! * Time is compressed: each coupled step represents [`DT_FLIGHT`] seconds of flight.
//! * Ionization is the **uncalibrated finite-rate network** (RP-1232 Table II pairs, no Saha
//!   target): associative ionization `N + O -> NO⁺ + e⁻` with its dissociative-recombination
//!   reverse (the physical blackout-exit mechanism), thresholded electron impact, and a lagged
//!   atom pool whose N clock carries the Zeldovich exchange. Each rate runs at its controlling
//!   temperature — ionization at the calibrated geometric mean `√(T_tr·T_ve)`, dissociation at
//!   Park's published `T_tr^0.7·T_ve^0.3`, electron channels at `T_e = T_ve` — with the
//!   Millikan-White vibrational clock on the **evolved per-cell pressure** and the network on
//!   the **evolved per-cell density**. The sheath is renewed each step: the stagnation-line A/B
//!   under recombination measured renewal at +0.48 and carried at -0.33 decades of the flight
//!   anchor and kept renewal, whose fixed-point clock is the network's true Riccati timescale
//!   (the old forward-only surrogate *needed* renewal against runaway, measured 268x; the
//!   network self-limits either way).
//! * The trajectory is point-mass 3-DOF: drag along the velocity, lift `L = (L/D)·D` rotated by
//!   the clamped bank command (one-step actuation lag). 6-DOF is out of scope (no flight anchor).
//! * The descent is steep and compressed, so the peak deceleration is ballistic-probe class; the
//!   envelope ceiling sits above it so the cybernetic gate bounds *bank*, not existence.
//! * GNSS fixes carry deterministic receiver noise whose variance equals the filter's `R`.
//!
//! **Precision.** The constants are exact `f64` specification literals; `support::ft` lifts each
//! one losslessly into `FloatType`, and every derived number is computed in `FloatType`.

// ── Carrier (the timing study's fast GO configuration: 32^2, bond cap 16)

/// Grid mode count; the layer is `2^L x 2^L`.
pub const L: usize = 5;
/// Bond cap of the tensor-train round policy, i.e. the compression ceiling.
pub const CAP: usize = 16;
/// Solver pseudo-time step (nondimensional; one per coupled step).
pub const DT_SOLVER: f64 = 0.002;
/// Reference wave speed of the implicit acoustic envelope. Deliberately snug: the peak-station
/// inflow outgrows it once, so the rebuild-on-drift mechanism fires where the descent steepens.
pub const S_REF: f64 = 1.8;
/// Effective ratio of specific heats through the shock (reacting air, the calibrated recipe).
pub const GAMMA_EFF: f64 = 1.1;
/// The compressed-time constant: seconds of flight each coupled step represents.
pub const DT_FLIGHT: f64 = 0.1;

/// Uniform nondimensional seed near the descent-start post-shock state (`~90 km`, Mach 24):
/// density, streamwise speed, and pressure in reference units. The inflow strip takes over
/// within a few steps.
pub const SEED_RHO_HAT: f64 = 0.054;
pub const SEED_U_HAT: f64 = 1.0;
pub const SEED_P_HAT: f64 = 0.04;

// ── Reference anchors (the peak-station post-shock values; fixed for the whole descent)

/// Temperature anchor: the exact RH `T₂` at the 61 km Mach-25 condition with `γ_eff = 1.1`, K.
pub const T_REF: f64 = 8044.0;
/// Density anchor: the post-shock `n₂ = n_∞·(ρ₂/ρ₁)` at the same condition, m⁻³.
pub const N_REF: f64 = 2.645e22;
/// Speed anchor: the post-shock speed `u₂` at the same condition, m·s⁻¹.
pub const U_REF: f64 = 376.0;

/// The RAM-C II ~61 km peak electron density anchor, m⁻³.
pub const RAMC_NE_REFERENCE: f64 = 1.0e19;

// ── Baseline atmosphere: `(altitude m, n_tot m⁻³, T K, a m/s)` rows, ascending altitude.
// US-1976 shape pinned to the RAM-C II 61 km freestream (`n_∞ = 1.3e21`), so the calibrated
// peak-station recipe is reproduced exactly as the descent sweeps that altitude.
pub const ATMOSPHERE: [(f64, f64, f64, f64); 11] = [
    // ── Powered-descent extension to the ground (`plasma-retropulsion-cfd-contracts`, capability
    //    `full-descent-atmosphere`): US Standard Atmosphere 1976 rows below 30 km. Sound speed
    //    a = √(γ·R·T) at γ = 1.4, R = 287 J/(kg·K); number density decreases monotonically into
    //    the 30 km row. `DescentSchedule::sample` clamps to the table ends, so appending here
    //    relocates the low clamp from 30 km to 0 km by data alone. ──
    (0.0, 2.5e25, 288.0, 340.2), // US-1976 sea level: T 288.15 K, ρ 1.225 kg/m³ (n = ρ/m̄)
    (5_000.0, 1.5e25, 255.7, 320.5), // US-1976 5 km: T 255.68 K, p 54.02 kPa
    (10_000.0, 8.6e24, 223.3, 299.5), // US-1976 10 km: T 223.25 K, p 26.44 kPa
    (15_000.0, 4.0e24, 216.7, 295.1), // US-1976 15 km: lower-stratosphere isotherm 216.65 K
    (20_000.0, 1.8e24, 216.7, 295.1), // US-1976 20 km: isotherm 216.65 K, p 5.475 kPa
    (25_000.0, 8.2e23, 221.6, 298.4), // US-1976 25 km: T 221.65 K, p 2.511 kPa
    // ── Original rows (byte-identical): US-1976 shape pinned to the RAM-C II 61 km freestream. ──
    (30_000.0, 3.0e23, 226.0, 302.0),
    (45_000.0, 2.4e22, 264.0, 326.0),
    (61_000.0, 1.3e21, 250.0, 317.0),
    (75_000.0, 3.0e20, 208.0, 289.0),
    (90_000.0, 7.0e19, 187.0, 274.0),
];

// ── Flight physics

/// GPS L1 as an angular frequency: `ω = 2π · 1.57542 GHz`.
pub const COMMS_BAND_RAD_S: f64 = 9.899e9;
/// Vehicle characteristic length for the Knudsen number (RAM-C-like forebody scale, m).
pub const L_CHAR: f64 = 0.3;
/// Effective air molecule diameter for the freestream mean free path, m.
pub const AIR_MOLECULE_DIAMETER_M: f64 = 3.7e-10;
/// Mean molecular mass of air, kg (28.97 amu).
pub const AIR_MEAN_MOLECULAR_MASS_KG: f64 = 4.81e-26;
/// The ballistic bundle `C_d·A/m`, m²·kg⁻¹. A light probe (`β ≈ 170 kg/m²`), so the compressed
/// descent decelerates below the ionization threshold before the table floor: the flow-resolved
/// exit mechanism.
pub const CDA_OVER_M: f64 = 5.8e-3;
/// Point-mass lift-to-drag ratio (lifting-capsule class).
pub const L_OVER_D: f64 = 0.3;
/// The equivalent-airspeed reference density, kg·m⁻³: the feeds stage publishes
/// `EAS = V·√(ρ_∞/ρ_ref)`, so the lift stage's `q = ½·ρ_ref·EAS²` is the *true* freestream
/// dynamic pressure at every altitude.
pub const RHO_REF: f64 = 1.0;
/// Standard gravity, m·s⁻².
pub const G0: f64 = 9.80665;
/// Sutton-Graves stagnation-heating constant for air, kg^0.5·m⁻¹ (q = k·√(ρ/R_n)·V³).
pub const SUTTON_GRAVES_K: f64 = 1.7415e-4;
/// Nose radius, m (the RAM-C 6-inch hemisphere).
pub const NOSE_RADIUS_M: f64 = 0.1524;

// ── Park two-temperature closure (the calibrated RAM-C recipe)

/// Reduced mass of the dominant relaxing collision pair, amu (N₂-N₂).
pub const REDUCED_MASS_AMU: f64 = 7.0;
/// Characteristic vibrational temperature of N₂, K.
pub const THETA_VIB: f64 = 3393.0;
/// Sheath residence time `t_res = standoff/u₂` at the peak station, s. Held constant over the
/// descent (the standoff-to-speed ratio varies less than the chemistry it clocks).
pub const RESIDENCE_TIME_S: f64 = 2.0e-5;
/// The sheath exposure at the transit-age profile's observable peak, s. On the stagnation line
/// the flow decelerates linearly to zero at the body, so a parcel's age at fractional depth ξ is
/// `age(ξ) = t_res·ln(1/(1−ξ))` (geometry and the Rankine-Hugoniot state only, no free
/// parameter), and the `qtt_ramc_stagline` gate reads the oldest sampled parcel (ξ = 64/65):
/// the reflectometer-visible near-body gas has aged `ln(65) ≈ 4.174` residence times. Both
/// sheath clocks (the vibrational bath and the network renewal) run at this exposure, mirroring
/// the stagnation-line measurement that pinned the corridor's anchor band.
pub const SHEATH_PEAK_AGE_S: f64 = RESIDENCE_TIME_S * 4.174;
/// Freestream vibrational temperature the bath relaxes up from, K.
pub const T_VE_INITIAL: f64 = 250.0;
/// Fallback Millikan-White pressure (atm) when the evolved field is absent (first-step guard).
pub const FALLBACK_PRESSURE_ATM: f64 = 2.9e-2;
/// Fallback heavy-particle density (m⁻³) when the evolved field is absent (first-step guard).
pub const FALLBACK_N_TOT: f64 = 2.645e22;

// ── Navigation

/// Initial true position: the descent starts at 90 km on the +x radial, m.
pub const TRUTH_ALTITUDE_0: f64 = 90_000.0;
/// Initial true velocity: a steep compressed entry, sized so the drag-decelerated speed at the
/// 61 km passage is the calibrated **Mach-25 station**.
pub const TRUTH_V0: [f64; 3] = [-1_300.0, 7_860.0, 0.0];
/// Initial INS error before the first fix folds (m per axis).
pub const NAV_INIT_ERR: [f64; 3] = [50.0, -30.0, 20.0];
/// Initial ESKF covariance diagonal, one entry per error-state block: position (50 m)²,
/// velocity (10 m/s)², attitude (10 mrad)², accelerometer bias (0.1 m/s²)², gyro bias, clock.
/// A flat prior across all 17 states would let the filter explain fix residuals with physically
/// absurd bias and attitude excursions, whose injected velocity corrections corrupt the nominal.
pub const P0_DIAG: [f64; 17] = [
    2500.0, 2500.0, 2500.0, // position
    100.0, 100.0, 100.0, // velocity
    1.0e-4, 1.0e-4, 1.0e-4, // attitude
    1.0e-2, 1.0e-2, 1.0e-2, // accelerometer bias
    1.0e-8, 1.0e-8, 1.0e-8, // gyro bias
    1.0e-6, 1.0e-8, // clock bias, drift
];
/// ESKF process-noise diagonal per coupled step, per block: modest position/velocity random
/// walk (integrator and gravity-gradient mismatch), near-constant biases.
pub const Q_DIAG: [f64; 17] = [
    1.0e-4, 1.0e-4, 1.0e-4, // position
    1.0e-4, 1.0e-4, 1.0e-4, // velocity
    1.0e-12, 1.0e-12, 1.0e-12, // attitude
    1.0e-12, 1.0e-12, 1.0e-12, // accelerometer bias
    1.0e-14, 1.0e-14, 1.0e-14, // gyro bias
    1.0e-12, 1.0e-14, // clock bias, drift
];
/// GNSS fix variance, m²: a precise code-phase receiver at 1 m 1σ. The published fixes carry
/// deterministic receiver noise with exactly this variance, so the filter's `R` matches the
/// sensor.
pub const GNSS_VAR: f64 = 1.0;
/// Through-plasma optical fix variance (50 m 1σ), m². No optical fix is published; the budget
/// is carried but unused.
pub const OPTICAL_VAR: f64 = 2500.0;
/// Accelerometer bias at the calibration point, m·s⁻² per axis: tactical-grade (~2 mg). Over a
/// multi-second dwell the t² drift clears the 1 m receiver noise without exaggeration.
pub const IMU_ACCEL_BIAS: [f64; 3] = [2.0e-2, -1.4e-2, 1.0e-2];
/// Gyro bias (unused by the translational corridor; carried for the IMU's grade).
pub const IMU_GYRO_BIAS: [f64; 3] = [0.0, 0.0, 0.0];

// ── Safety envelope

/// Heat-flux certification ceiling, W·m⁻².
pub const MAX_HEAT_FLUX: f64 = 2.0e7;
/// G-load ceiling: above the steep compressed profile's ballistic peak, so the gate bounds bank
/// rather than aborting the descent.
pub const MAX_G_LOAD: f64 = 100.0;
/// Bank-angle magnitude cap, rad (~28.6 deg).
pub const MAX_BANK_RAD: f64 = 0.5;
