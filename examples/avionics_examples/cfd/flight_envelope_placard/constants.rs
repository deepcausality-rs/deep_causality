/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Every number the placard study uses, with its justification. The constants are exact `f64`
//! specification literals; `main::ft` lifts each one losslessly into the working `FloatType`,
//! and every derived number is computed in that type.

// ── Gas model

/// Ratio of specific heats for calorically perfect air. Exact for a diatomic ideal gas with
/// frozen vibration; the standard value below the dissociation regime (Anderson, *Modern
/// Compressible Flow*, 3rd ed., ch. 1). The hottest grid point here reaches a stagnation
/// temperature near 1500 K, where vibrational excitation has begun but dissociation has not,
/// so the perfect-gas value is the honest effective gamma for this envelope. The README states
/// where the approximation is crude.
pub const GAMMA: f64 = 1.4;

/// Mean molecular mass of air, kg (28.97 amu). Converts the atmosphere table's number density
/// into mass density; the same value the blackout examples carry.
pub const AIR_MEAN_MOLECULAR_MASS_KG: f64 = 4.81e-26;

// ── Stagnation-point heating

/// Sutton-Graves stagnation-heating constant for air, kg^0.5·m⁻¹, in
/// `q̇ = k·√(ρ_∞/R_n)·V³` (Sutton, K. and Graves, R. A., "A General Stagnation-Point
/// Convective-Heating Equation for Arbitrary Gas Mixtures", NASA TR R-376, 1971). The same
/// constant the blackout examples' load stage carries. The correlation is calibrated for
/// blunt-body entry speeds; at the low-supersonic end of this grid its numbers are small and
/// serve as a trend column, not a thermal-protection sizing input.
pub const SUTTON_GRAVES_K: f64 = 1.7415e-4;

/// Nose radius, m: a stated example constant for a blunt demonstrator forebody. Sets the
/// `√(1/R_n)` scale of the heating column; half a meter is a round, plausible leading-body
/// radius for a supersonic testbed and is not tied to any specific vehicle.
pub const NOSE_RADIUS_M: f64 = 0.5;

// ── Placards (the acceptance envelope)

/// Dynamic-pressure placard, kPa. A chosen demonstration placard for a transport-category-like
/// supersonic envelope, not certification data: the recorded matrix peaks near 24 kPa at
/// M 1.20 / 11 km, so 60 kPa leaves the whole corridor inside the envelope with margin, while
/// low-altitude supersonic flight (the exceeds matrix's M 1.5 / 5 km point, about 85 kPa)
/// lands outside it.
pub const Q_MAX_PLACARD_KPA: f64 = 60.0;

/// Post-shock stagnation-temperature placard, K. A chosen demonstration ceiling for an
/// uncooled hot-structure leading edge (nickel-alloy class), not certification data: the
/// recorded matrix peaks near 1502 K at M 5.0 / 40 km, so 1700 K bounds the corridor with
/// about 13 percent margin.
pub const T0_MAX_PLACARD_K: f64 = 1700.0;

// ── Atmosphere: `(altitude m, n_tot m⁻³, T K, a m/s)` rows, ascending altitude.
// U.S. Standard Atmosphere, 1976 (NOAA-S/T 76-1562): temperature, speed of sound, and density
// at the tabulated geometric altitudes, with density converted to number density through the
// 28.97 amu mean molecular mass ([`AIR_MEAN_MOLECULAR_MASS_KG`]). Same source and shape as the
// blackout examples' table, extended down to sea level for this envelope. The study
// interpolates linearly between rows; the README states what that costs.
pub const ATMOSPHERE: [(f64, f64, f64, f64); 9] = [
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
