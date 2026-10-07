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
//!   under recombination measured renewal at +0.22 and carried at -1.18 decades of the flight
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
/// Reference wave speed of the implicit acoustic envelope.
///
/// The rebuild-on-drift mechanism does **not** fire on the corridor's current profile. The carrier
/// re-pins its envelope when the marched state needs more than it has, `û + √(γ·T̂) > s_ref·(1 +
/// tol)`; at 1.8 the descent never crosses that, so the corridor's gate (5a) reports 0 rebuilds and
/// its provenance log carries no `carrier rebuilt at step` entry. The weather example flies the same
/// value and also records none. Nothing here is out of tolerance; the envelope is simply wide enough
/// for the profiles these examples fly.
///
/// The mechanism is exercised by the retropropulsion example's terminal leg, which flies
/// [`S_REF_TERMINAL`] (1.4) into sea-level air and logs `carrier rebuilt at step 1: s_ref 1.4 ->
/// 2.7683613571273638 (rebuild 1)`. Read that example, not this constant, for the rebuild path.
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

// ── Reference scales (nondimensionalization; fixed for the whole descent)

/// Temperature scale, K. The marched layer is nondimensionalized by this value and the evolved
/// projections are re-dimensionalized by it, so it sets conditioning, not physics; it is of the
/// order of the exact Rankine-Hugoniot `T₂` at the 71.93 km RAM-C II anchor station with
/// `γ_eff = 1.1`. [`SEED_RHO_HAT`], [`SEED_P_HAT`], [`S_REF`] and [`DT_SOLVER`] are in these units.
pub const T_REF: f64 = 8044.0;
/// Number-density scale, m⁻³, of the order of the post-shock `n₂ = n_∞·(ρ₂/ρ₁)` at the anchor
/// station. A scale, as [`T_REF`].
pub const N_REF: f64 = 2.645e22;
/// Speed scale, m·s⁻¹, of the order of the post-shock speed `u₂` at the anchor station. A scale,
/// as [`T_REF`].
pub const U_REF: f64 = 376.0;

// ── The RAM-C II flight anchor (Grantham 1970, NASA TN D-6062)

/// The RAM-C II station-1 (x/D = 0.15) peak electron density at its Ka-band critical-density
/// crossing, m⁻³: `N_e,pk = 0.63 · 1.287e-8 · f²` cm⁻³ at `f = 35 000 MHz` = 9.93e12 cm⁻³.
/// Grantham p. 18 places the crossing at 236 000 ft ([`RAMC_ANCHOR_ALTITUDE_M`]); p. 11 gives the
/// critical-density relation and the 0.63 slope-technique ratio; Table I the Ka-band frequency.
pub const RAMC_NE_REFERENCE: f64 = 0.63 * 1.287e-8 * 3.5e10 * 3.5e10 * 1.0e6;
/// Altitude of the station-1 Ka-band crossing, m (236 000 ft).
pub const RAMC_ANCHOR_ALTITUDE_M: f64 = 71_930.0;
/// RAM-C II flight speed at the anchor station, m/s (Grantham 1970, Table VII: 7.66 km/s at 71 km).
/// The corridor's entry velocity is sized to cross [`RAMC_ANCHOR_ALTITUDE_M`] at this speed.
pub const RAMC_ANCHOR_SPEED_MS: f64 = 7_660.0;
/// Lowest altitude of the RAM-C II primary data period, m (185 000 ft, beryllium-cap ejection).
/// Between this altitude and [`RAMC_ANCHOR_ALTITUDE_M`] station 1 is overdense at Ka-band, the
/// highest frequency flown, so [`RAMC_NE_REFERENCE`] is a lower bound on the flight's peak
/// electron density there.
pub const RAMC_LOWER_BOUND_FLOOR_M: f64 = 56_390.0;

// ── Baseline atmosphere: `(altitude m, n_tot m⁻³, T K, a m/s)` rows, ascending altitude.
//
// U.S. Standard Atmosphere 1976 (NOAA/NASA/USAF, NASA-TM-X-74335), at 1 km spacing from 0 to
// 90 km, generated from the standard's defining constants: Table 2 (k = 1.380622e-23 N·m/K,
// R* = 8.31432e3 N·m/(kmol·K), g₀ = 9.80665 m/s², P₀ = 101 325 Pa, r₀ = 6356.766 km,
// T₀ = 288.15 K, γ = 1.40) and Table 4 (layer base geopotential heights 0, 11, 20, 32, 47, 51,
// 71, 84.852 km' with gradients -6.5, 0.0, +1.0, +2.8, 0.0, -2.8, -2.0 K/km'), with
// M₀ = 28.9644 kg/kmol. `n_tot = P/(kT)`, `a = √(γ·R*·T/M₀)`. The generator reproduces Table I
// to five significant figures at 61, 71 and 72 km (61 km: 244.274 K, 19.157 Pa,
// 2.7321e-4 kg/m³). Above 86 km the rows extend isothermally at the 86 km temperature,
// hydrostatic in geometric altitude; at 90 km that agrees with Table I's density to 0.2 %.
// Number density decreases monotonically through the table.
pub const ATMOSPHERE: [(f64, f64, f64, f64); 91] = [
    (0.0, 2.5470e25, 288.150, 340.29),
    (1000.0, 2.3113e25, 281.651, 336.43),
    (2000.0, 2.0928e25, 275.154, 332.53),
    (3000.0, 1.8905e25, 268.659, 328.58),
    (4000.0, 1.7036e25, 262.166, 324.59),
    (5000.0, 1.5312e25, 255.676, 320.55),
    (6000.0, 1.3725e25, 249.187, 316.45),
    (7000.0, 1.2267e25, 242.700, 312.31),
    (8000.0, 1.0932e25, 236.215, 308.11),
    (9000.0, 9.7110e24, 229.733, 303.85),
    (10000.0, 8.5975e24, 223.252, 299.53),
    (11000.0, 7.5848e24, 216.774, 295.15),
    (12000.0, 6.4857e24, 216.650, 295.07),
    (13000.0, 5.5430e24, 216.650, 295.07),
    (14000.0, 4.7375e24, 216.650, 295.07),
    (15000.0, 4.0493e24, 216.650, 295.07),
    (16000.0, 3.4612e24, 216.650, 295.07),
    (17000.0, 2.9587e24, 216.650, 295.07),
    (18000.0, 2.5292e24, 216.650, 295.07),
    (19000.0, 2.1622e24, 216.650, 295.07),
    (20000.0, 1.8486e24, 216.650, 295.07),
    (21000.0, 1.5742e24, 217.581, 295.70),
    (22000.0, 1.3413e24, 218.574, 296.38),
    (23000.0, 1.1437e24, 219.567, 297.05),
    (24000.0, 9.7591e23, 220.560, 297.72),
    (25000.0, 8.3341e23, 221.552, 298.39),
    (26000.0, 7.1225e23, 222.544, 299.06),
    (27000.0, 6.0916e23, 223.536, 299.72),
    (28000.0, 5.2138e23, 224.527, 300.39),
    (29000.0, 4.4657e23, 225.518, 301.05),
    (30000.0, 3.8278e23, 226.509, 301.71),
    (31000.0, 3.2833e23, 227.500, 302.37),
    (32000.0, 2.8183e23, 228.490, 303.02),
    (33000.0, 2.4062e23, 230.973, 304.67),
    (34000.0, 2.0557e23, 233.744, 306.49),
    (35000.0, 1.7597e23, 236.513, 308.30),
    (36000.0, 1.5090e23, 239.282, 310.10),
    (37000.0, 1.2965e23, 242.050, 311.89),
    (38000.0, 1.1158e23, 244.818, 313.67),
    (39000.0, 9.6197e22, 247.584, 315.43),
    (40000.0, 8.3076e22, 250.350, 317.19),
    (41000.0, 7.1864e22, 253.114, 318.94),
    (42000.0, 6.2266e22, 255.878, 320.67),
    (43000.0, 5.4035e22, 258.641, 322.40),
    (44000.0, 4.6965e22, 261.403, 324.12),
    (45000.0, 4.0882e22, 264.164, 325.82),
    (46000.0, 3.5640e22, 266.925, 327.52),
    (47000.0, 3.1115e22, 269.684, 329.21),
    (48000.0, 2.7376e22, 270.650, 329.80),
    (49000.0, 2.4176e22, 270.650, 329.80),
    (50000.0, 2.1350e22, 270.650, 329.80),
    (51000.0, 1.8856e22, 270.650, 329.80),
    (52000.0, 1.6750e22, 269.031, 328.81),
    (53000.0, 1.4926e22, 266.277, 327.12),
    (54000.0, 1.3286e22, 263.524, 325.43),
    (55000.0, 1.1812e22, 260.771, 323.72),
    (56000.0, 1.0488e22, 258.019, 322.01),
    (57000.0, 9.3017e21, 255.268, 320.29),
    (58000.0, 8.2390e21, 252.518, 318.56),
    (59000.0, 7.2882e21, 249.769, 316.82),
    (60000.0, 6.4387e21, 247.021, 315.07),
    (61000.0, 5.6805e21, 244.273, 313.32),
    (62000.0, 5.0047e21, 241.527, 311.55),
    (63000.0, 4.4031e21, 238.781, 309.77),
    (64000.0, 3.8683e21, 236.036, 307.99),
    (65000.0, 3.3934e21, 233.292, 306.19),
    (66000.0, 2.9723e21, 230.549, 304.39),
    (67000.0, 2.5995e21, 227.807, 302.57),
    (68000.0, 2.2698e21, 225.065, 300.75),
    (69000.0, 1.9787e21, 222.325, 298.91),
    (70000.0, 1.7221e21, 219.585, 297.06),
    (71000.0, 1.4963e21, 216.846, 295.20),
    (72000.0, 1.2968e21, 214.263, 293.44),
    (73000.0, 1.1191e21, 212.308, 292.10),
    (74000.0, 9.6442e20, 210.353, 290.75),
    (75000.0, 8.3002e20, 208.399, 289.40),
    (76000.0, 7.1338e20, 206.446, 288.04),
    (77000.0, 6.1227e20, 204.493, 286.67),
    (78000.0, 5.2475e20, 202.541, 285.30),
    (79000.0, 4.4909e20, 200.589, 283.92),
    (80000.0, 3.8377e20, 198.639, 282.54),
    (81000.0, 3.2746e20, 196.688, 281.15),
    (82000.0, 2.7899e20, 194.739, 279.75),
    (83000.0, 2.3732e20, 192.790, 278.35),
    (84000.0, 2.0155e20, 190.841, 276.94),
    (85000.0, 1.7090e20, 188.893, 275.52),
    (86000.0, 1.4466e20, 186.946, 274.10),
    (87000.0, 1.2109e20, 186.946, 274.10),
    (88000.0, 1.0137e20, 186.946, 274.10),
    (89000.0, 8.4857e19, 186.946, 274.10),
    (90000.0, 7.1042e19, 186.946, 274.10),
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

/// Reduced mass of the dominant relaxing collision pair, amu. Defined once in `deep_causality_cfd`
/// (next to the `Park2tClosure` it feeds) and re-exported here, so all three plasma-blackout examples
/// and the `qtt_ramc_stagline` harness see the same value from a single source.
pub use deep_causality_cfd::REDUCED_MASS_AMU;
/// Characteristic vibrational temperature of N₂, K.
pub const THETA_VIB: f64 = 3393.0;
/// Sheath residence time `t_res = standoff/u₂` at the anchor station (the stagnation-line
/// harness's 7.6 mm standoff over its post-shock `u₂`), s. Held constant over the descent (the
/// standoff-to-speed ratio varies less than the chemistry it clocks).
pub const RESIDENCE_TIME_S: f64 = 2.0e-5;
/// The sheath exposure at the transit-age profile's observable peak, s. On the stagnation line
/// the flow decelerates linearly to zero at the body, so a parcel's age at fractional depth ξ is
/// `age(ξ) = t_res·ln(1/(1−ξ))` (geometry and the Rankine-Hugoniot state only, no free
/// parameter), and the `qtt_ramc_stagline` gate reads the oldest sampled parcel (ξ = 64/65):
/// the reflectometer-visible near-body gas has aged `ln(65) ≈ 4.174` residence times. Both
/// sheath clocks (the vibrational bath and the network renewal) run at this exposure, mirroring
/// the stagnation-line measurement that pinned the corridor's anchor band.
pub const SHEATH_PEAK_AGE_S: f64 = RESIDENCE_TIME_S * 4.174;
/// Freestream vibrational temperature the bath relaxes up from, K, held for the whole descent. The
/// US-1976 freestream spans 187-288 K; the bath relaxes from this value toward `T_tr` (thousands of
/// kelvin), so the spread moves the relaxed `T_ve` by under 1 % of its post-shock rise.
pub const T_VE_INITIAL: f64 = 250.0;
/// Fallback Millikan-White pressure (atm) when the evolved field is absent (first-step guard).
pub const FALLBACK_PRESSURE_ATM: f64 = 2.9e-2;
/// Fallback heavy-particle density (m⁻³) when the evolved field is absent (first-step guard).
pub const FALLBACK_N_TOT: f64 = 2.645e22;

// ── Navigation

/// Initial true position: the descent starts at 90 km on the +x radial, m.
pub const TRUTH_ALTITUDE_0: f64 = 90_000.0;
/// Initial true velocity: a steep compressed entry, scaled at fixed flight-path angle so the
/// drag-decelerated vehicle crosses the RAM-C II anchor altitude ([`RAMC_ANCHOR_ALTITUDE_M`]) at the
/// flight's 7.66 km/s (Grantham 1970, Table VII), which matches the stagnation-line harness's
/// freestream at the anchor comparison.
pub const TRUTH_V0: [f64; 3] = [-1_255.1, 7_588.6, 0.0];
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
/// ESKF process-noise **continuous-time spectral density** (units `state²/s`), per block: modest
/// position/velocity random walk (integrator and gravity-gradient mismatch), near-constant biases.
/// [`NavFilter::predict`](deep_causality_cfd::NavFilter::predict) discretises it onto each step as
/// `Q_d = Q_c·dt`.
///
/// These are the previous per-step values re-expressed as a spectral density by the
/// `per-step → per-second` conversion `Q_c = Q_step / DT_FLIGHT` (i.e. ×10 at `DT_FLIGHT = 0.1 s`), so
/// that `Q_c·dt` at the configured step reproduces the calibrated per-step process noise while making the
/// tuning independent of the step size. The magnitudes therefore read 10× the pre-2026-07-24 per-step
/// diagonal. The round-trip `Q_step × 10 × 0.1` is bit-exact for the position, velocity, attitude,
/// accel-bias and clock-bias blocks; for the gyro-bias and clock-drift blocks (`1e-14`) it differs by
/// ≤1 ULP (`1e-13 × 0.1 ≠ 1e-14` in `f64`), far below any observable effect, so the filter's behaviour at
/// `DT_FLIGHT` is unchanged **at every displayed figure** (see the discretisation note on `NavFilter::predict`).
pub const Q_DIAG: [f64; 17] = [
    1.0e-3, 1.0e-3, 1.0e-3, // position
    1.0e-3, 1.0e-3, 1.0e-3, // velocity
    1.0e-11, 1.0e-11, 1.0e-11, // attitude
    1.0e-11, 1.0e-11, 1.0e-11, // accelerometer bias
    1.0e-13, 1.0e-13, 1.0e-13, // gyro bias
    1.0e-11, 1.0e-13, // clock bias, drift
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

// ── Retropropulsion (the powered-descent example's vehicle; M4/M5)

/// Full-throttle thrust of the retro engine, N. A single central nozzle — the configuration the
/// Jarvinen–Adams drag-preservation dataset measures, and the one whose drag collapse the example
/// exists to demonstrate.
pub const RETRO_THRUST_N: f64 = 70_000.0;
/// Specific impulse, s (kerolox-class sea-level engine).
pub const RETRO_ISP_S: f64 = 282.0;
/// Vehicle wet mass at the ignition corridor, kg. Pre-ignition this is the mass implied by the
/// corridor's `CDA_OVER_M` bundle, so Act-1 force normalization is unchanged by carrying mass as
/// state.
pub const VEHICLE_MASS_KG: f64 = 3_400.0;
/// Usable propellant at the ignition corridor, kg.
///
/// Sized for the burn the example actually flies. The ignition corridor sits in the Jarvinen–Adams
/// Mach band (0.4–2.0), which is where the SRP physics is anchored and therefore where this example
/// must burn — but it is *early* for a landing burn, so the vehicle spends propellant fighting drag
/// the atmosphere would otherwise supply free. A tank sized for a late suicide burn runs dry around
/// 13 km.
pub const PROPELLANT_KG: f64 = 2_200.0;
/// Forebody drag coefficient of the aeroshell.
///
/// Blunt-capsule class in hypersonic continuum flow (Apollo-command-module lineage, ~1.35-1.4). It
/// exists so the reference area below is *derived* rather than stated: the ballistic bundle
/// `CDA_OVER_M` and an aerodynamic reference area describe the same vehicle, and stating both
/// independently lets them describe two.
pub const VEHICLE_CD: f64 = 1.4;

/// Aerodynamic **reference area** for the thrust coefficient `C_T = T/(q∞·S_ref)`, m².
///
/// Derived from the flown ballistic bundle, not stated beside it:
/// `S_ref = (C_d·A/m)·m / C_d = CDA_OVER_M · VEHICLE_MASS_KG / VEHICLE_CD`.
///
/// Stated independently the two disagreed. `CDA_OVER_M · VEHICLE_MASS_KG` is 19.72 m² of `C_d·A`, so
/// a 4.6 m² reference area implied a drag coefficient of **4.29** — roughly three times any capsule.
/// `C_T` sets the preserved-drag fraction and the envelope's dynamic throttle ceiling, while
/// `CDA_OVER_M` sets the drag that fraction multiplies, so the two fed one expression chain while
/// describing different vehicles. Derived, they cannot drift: this yields 14.09 m² (a 4.23 m
/// aeroshell) at a ballistic coefficient of 172 kg/m², which is the `β ≈ 170` the bundle is
/// documented as.
///
/// Named apart from the shared `S_REF` on purpose: that constant is the reference **wave speed** of
/// the implicit acoustic envelope. The two are the same scalar type and a plausible-looking name
/// apart, so passing the wrong one compiles and runs.
pub const PLUME_S_REF_M2: f64 = CDA_OVER_M * VEHICLE_MASS_KG / VEHICLE_CD;

// ── Powered-descent safety envelope (the burn axes)

/// Throttle floor: below it the central-nozzle jet-penetration flow is unsteady, so a *running*
/// engine may not be throttled under it. A commanded shutdown is not bounded by it.
pub const THROTTLE_FLOOR: f64 = 0.15;
/// Throttle ceiling (engine deep-throttle limit).
pub const THROTTLE_CEILING: f64 = 0.95;
/// Thrust-coefficient cap. Bow-shock instabilities appear past `C_T ≈ 3` (Jarvinen–Adams;
/// Keyes–Hefner), so the envelope holds the burn below it — a *dynamic* throttle ceiling, since
/// `C_T` moves with the sensed dynamic pressure.
pub const MAX_CT: f64 = 3.0;
/// Ignition dynamic-pressure window, Pa. Bounds when the burn may *start*, not how hard it may
/// push: once the burn is under way the running axes bound it instead.
pub const IGNITION_Q_MIN: f64 = 1_200.0;
/// Upper edge of the ignition dynamic-pressure window, Pa.
pub const IGNITION_Q_MAX: f64 = 26_000.0;
/// Propellant reserve floor, kg. Thrust commanded at or below it is an unrecoverable breach.
pub const PROPELLANT_FLOOR_KG: f64 = 40.0;
/// Descent-rate bound over the whole flown profile, m·s⁻¹.
///
/// The gate applies this on **every** step once a throttle is commanded, and the guidance commands
/// zero from step 0 so the burn axes stay live before ignition — so this axis must admit the entry
/// interface, where the vehicle is still falling at well over a kilometre per second because the
/// atmosphere has not yet decelerated it. It is a whole-profile limit, not a landing limit; the
/// tight touchdown figure is a gate over the terminal state, not an envelope axis.
pub const MAX_DESCENT_RATE: f64 = 1_500.0;

// ── Ignition corridor (the commit conditions)

/// Lower edge of the ignition Mach band. Jarvinen–Adams spans Mach 0.4–2.0, so the corridor commits
/// inside the dataset's validated envelope.
pub const IGNITION_MACH_MIN: f64 = 0.4;
/// Upper edge of the ignition Mach band.
pub const IGNITION_MACH_MAX: f64 = 2.0;
/// Sigma multiplier on the dispersion table's navigation drift: the commit demands
/// `drift_mean + k·drift_sd` from the interpolated row.
pub const IGNITION_MARGIN_K: f64 = 3.0;
/// Altitude below which the classifier reports touchdown, m.
pub const TOUCHDOWN_ALTITUDE_M: f64 = 15.0;

// ── Terminal-leg carrier anchors (M5)

/// Ratio of specific heats for the terminal leg. Cool low-Mach air, not the reacting-shock recipe:
/// the carrier keeps the schedule's shock gamma and the marcher's gamma apart, and the terminal leg
/// sets both here.
pub const GAMMA_TERMINAL: f64 = 1.4;
/// Reference **wave speed** of the terminal leg's implicit acoustic envelope. At low Mach the sound
/// speed dominates, so this is retuned rather than inherited.
pub const S_REF_TERMINAL: f64 = 1.4;
/// Terminal-leg reference temperature, K (near sea-level standard).
pub const T_REF_TERMINAL: f64 = 288.0;
/// Terminal-leg reference number density, m⁻³ (near sea level). The corridor's anchor is a 90 km
/// post-shock value; carrying it down here would leave the nondimensional density orders of
/// magnitude from unity, and the rebuild trigger — keyed on wave speed alone — never corrects that.
pub const N_REF_TERMINAL: f64 = 2.5e25;
/// Terminal-leg reference speed, m·s⁻¹.
pub const U_REF_TERMINAL: f64 = 120.0;
/// Terminal-leg seed density (nondimensional).
pub const SEED_RHO_HAT_TERMINAL: f64 = 1.0;
/// Terminal-leg seed axial momentum (nondimensional).
pub const SEED_U_HAT_TERMINAL: f64 = 0.4;
/// Terminal-leg seed energy (nondimensional).
pub const SEED_P_HAT_TERMINAL: f64 = 1.0;
/// The terminal leg's rebuild budget: a leg needing more re-pins than this is not converging on an
/// acoustic envelope, and the carrier refuses rather than marching on an undersized one.
pub const TERMINAL_REBUILD_BUDGET: usize = 4;

// ── The plume as an imprint on the marched layer (M3's opt-in state-realism seam)

/// Chamber (stagnation) pressure at full throttle, Pa. Scales linearly with commanded throttle.
pub const CHAMBER_PRESSURE_MAX: f64 = 1.2e7;
/// Chamber (stagnation) temperature, K (kerolox).
pub const CHAMBER_TEMPERATURE: f64 = 3_400.0;
/// Jet specific gas constant, J/(kg·K).
pub const JET_R_SPECIFIC: f64 = 355.0;
/// Jet ratio of specific heats. Inside the Cordell envelope [1.2, 1.4].
pub const JET_GAMMA: f64 = 1.22;
/// Nozzle exit Mach number.
pub const NOZZLE_EXIT_MACH: f64 = 2.8;
/// Conical nozzle half-angle, rad (15°).
pub const NOZZLE_HALF_ANGLE_RAD: f64 = 0.2618;
/// Throat diameter, m.
pub const NOZZLE_THROAT_D: f64 = 0.19;
/// Exit radius, m.
pub const NOZZLE_EXIT_R: f64 = 0.42;
/// Cone length, m.
pub const NOZZLE_CONE_L: f64 = 0.86;
/// Freestream ratio of specific heats.
pub const PLUME_GAMMA_INF: f64 = 1.4;

// ── The two SRP models' validity envelopes, and the fact that they barely overlap
//
// The drag correlation and the plume-boundary model come from different work and are validated over
// different flight regimes:
//
//   * Jarvinen-Adams measured drag preservation over **Mach 0.4-2.0**, which is why the ignition
//     corridor commits inside that band.
//   * Cordell-Braun validated the analytic plume boundary over **Mach 2-4**.
//
// They meet at a single point. A descent flying the correlation's band is therefore outside the
// geometry model's for essentially all of the burn, and the marched-layer imprint the geometry drives
// can only be live in the instant around Mach 2. This was previously invisible: the geometry model
// was handed a frozen freestream Mach of 2.2 while the vehicle flew the Jarvinen-Adams band, so its
// own envelope check tested the constant and always passed. Both bands are now declared at the call
// site and both stand down where they do not apply.

/// Lower edge of the Cordell-Braun plume-boundary model's validated freestream Mach range.
pub const CORDELL_MACH_MIN: f64 = 2.0;
/// Upper edge of the Cordell-Braun plume-boundary model's validated freestream Mach range.
pub const CORDELL_MACH_MAX: f64 = 4.0;

/// Throttle move required before the carrier re-imprints the plume mask.
pub const IMPRINT_THROTTLE_TOL: f64 = 0.05;
/// Cap on plume re-imprints per leg, so a noisy throttle cannot rebuild the mask every step.
pub const IMPRINT_MAX_REFRESHES: usize = 24;
/// Body face on the unit square (the plume hugs it and extends upstream).
pub const IMPRINT_FACE_X: f64 = 0.55;
/// Plume axis on the unit square.
pub const IMPRINT_AXIS_Y: f64 = 0.5;
/// Mask skirt, in cell widths.
pub const IMPRINT_SMOOTHING_CELLS: f64 = 1.5;
/// Physical domain width the published plume geometry is nondimensionalized against, m.
pub const IMPRINT_DOMAIN_M: f64 = 20.0;
/// The pinned jet conserved state `[ρ̂, m̂x, m̂y, Ê]` the mask interior relaxes toward: dense,
/// directed **upstream** (−x, against the oncoming flow), and hot.
pub const IMPRINT_TARGET: [f64; 4] = [1.0, -1.0, 0.0, 2.0];
/// Penalization strength, solver time units. `η ≤ Δt` is a hard pin.
pub const IMPRINT_ETA: f64 = 0.002;

// ── Terminal (subsonic) burn envelope
//
// The supersonic-retropropulsion axes do NOT carry into the terminal leg, and applying them there
// is a physics error rather than conservatism. Both are bow-shock-interaction constraints:
//
//   * the `C_T <= 3` cap is the Jarvinen-Adams bow-shock instability bound. Subsonically there is
//     no bow shock for the plume to displace, and the dataset it comes from spans Mach 0.4-2.0.
//   * the throttle floor exists because below it the central-nozzle jet penetrates and the
//     interaction goes unsteady — again a supersonic-interaction statement.
//
// Below the SRP regime the vehicle is an ordinary rocket on a landing burn: Falcon 9 throttles to
// ~40%, the Apollo LM to ~10%, neither bounded by a thrust coefficient. Carrying the SRP cap down
// makes the admissible throttle collapse with dynamic pressure and forbids the engine entirely
// around 3.7 km — not because the vehicle cannot brake, but because it is not allowed to try.

/// Terminal-leg throttle floor: a deep-throttle limit, not a jet-penetration bound.
pub const TERMINAL_THROTTLE_FLOOR: f64 = 0.05;

/// Commanded contact speed at the touchdown plane, m·s⁻¹ — the vehicle is flown to *arrive*
/// descending at this rate rather than to stop above the ground. Set to the published Falcon 9
/// first-stage figure (about 2 m/s vertical at touchdown), which is a design point rather than a
/// residual: that booster's landing engine at minimum throttle out-thrusts the nearly empty stage,
/// so it cannot hover and must be flown into the deck.
///
/// This vehicle is a different class — at the floor above it produces 3.5 kN against roughly 25 kN
/// of weight, so it *can* hover, and a zero contact speed would be admissible. It is commanded
/// nonzero anyway, for the reason every real lander does: to make firm gear contact instead of
/// station-keeping over a site it is not standing on. It also keeps gate (6) honest — a guidance
/// nulled at exactly the sampled altitude reports its own setpoint back as a measurement.
pub const CONTACT_SPEED_MS: f64 = 2.0;
/// Terminal-leg thrust-coefficient cap.
///
/// Set beyond any reachable value on purpose: `C_T` is not a meaningful stability constraint
/// subsonically, and `BurnEnvelope` requires the axis to be present. Documented here rather than
/// silently tuned, so the reason it never binds is legible.
pub const TERMINAL_MAX_CT: f64 = 1.0e9;
/// Terminal-leg descent-rate bound, m·s⁻¹.
///
/// Must admit the leg's own **entry** condition — the vehicle hands over from the supersonic burn
/// still falling at ~146 m/s — because the gate applies this on every step from the first. It is a
/// whole-leg envelope axis, not a landing criterion; how softly the vehicle actually arrives is a
/// gate on the terminal state.
pub const TERMINAL_MAX_DESCENT_RATE: f64 = 200.0;
