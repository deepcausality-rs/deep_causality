/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Published supersonic-retropropulsion coefficients: the digitized
//! Jarvinen–Adams central-nozzle dataset and the flow-regime transition
//! bounds, plus Sibulkin's source-flow scaling used by the Cordell plume
//! model. Every block carries its source pinpoint; digitized values are
//! figure reads from the scanned report (stated uncertainty per block).

// ─────────────────────────────────────────────────────────────────────────
// Jarvinen & Adams, "The Aerodynamic Characteristics of Large Angled Cones
// with Retrorockets," Final Report MC 70-3001-R2 (BNY), NASA Contract
// NAS 7-576, February 1970 (NTRS 19720005324;
// `papers/jarvinen_adams_1970_ntrs_19720005324.pdf`).
// Thrust coefficient definition: C_T = T / (q_inf * A_m), A_m = model base
// area (report Nomenclature, p. v). Note: the Korzun–Cruz–Braun survey uses
// a DIFFERENT, nozzle-referenced definition C_T = T/(p0 * A*) — do not mix.
// ─────────────────────────────────────────────────────────────────────────

/// Preserved-drag fraction `C_A_F / C_A0` vs `C_T` for the 60° single
/// (central) engine aeroshell, Air Nozzle 2, at M∞ = 2.0. Digitized from
/// Fig. 32 (report p. 54), cross-checked against Fig. 56 (p. 81);
/// uncertainty ±0.02 in the fraction, ±0.05 in C_T (±0.03 above C_T = 4).
/// The sharp drop between C_T = 1.03 and 1.05 is the report's measured
/// jet-penetration → blunt-flow transition ("occurs sharply at a thrusting
/// coefficient near unity", report p. 32); the piecewise-linear bracket
/// across it deliberately encodes that jump. Negative preserved fractions
/// past C_T ≈ 2 are the report's slightly negative forebody axial force
/// (wake-type flow). Domain: C_T ∈ [0.0, 8.8].
pub const JARVINEN_ADAMS_PRESERVED_DRAG_M2: [(f64, f64); 12] = [
    (0.00, 1.00),
    (0.46, 0.22),
    (0.72, 0.17),
    (0.75, 0.16),
    (1.03, 0.12),
    (1.05, 0.02),
    (1.98, -0.03),
    (3.98, -0.09),
    (4.05, -0.10),
    (7.00, -0.12),
    (7.05, -0.13),
    (8.80, -0.12),
];

/// Unpowered (C_T = 0) axial-force coefficient `C_A0` vs freestream Mach for
/// the 60° single-engine aeroshell. Digitized from the C_T = 0 intercepts of
/// Figs. 32–33 (report pp. 54–55), corroborated by Fig. 11 (p. 27);
/// uncertainty ±0.03. The M = 0.4 point exists only for the three-nozzle
/// model (~0.52 with the 60° fairing) and is excluded; domain M ∈ [0.6, 2.0].
/// Caveat (report p. 25): the corner radius r_c/r_m = 0.06 reduces C_A about
/// 5% below sharp-corner cone data.
pub const JARVINEN_ADAMS_BASELINE_CA0: [(f64, f64); 5] = [
    (0.60, 0.60),
    (0.80, 0.68),
    (1.05, 0.89),
    (1.50, 1.12),
    (2.00, 1.25),
];

/// Unpowered axial-force coefficient at the M∞ = 2.0 reference station of
/// the preserved-drag curve (Fig. 32 intercept; Fig. 56).
pub const JARVINEN_ADAMS_CA0_M2: f64 = 1.25;

/// Jet-penetration → blunt-flow transition thrust coefficient at M∞ = 2.0
/// for the central nozzle: "The transition from jet penetration to blunt
/// flow regimes occurs sharply at a thrusting coefficient near unity (1)"
/// (report p. 32, §3.1.3). Below it the flow is unsteady (the jet penetrates
/// far upstream, bow shock up to six body diameters forward); above it the
/// blunt-flow regime is steady.
pub const JARVINEN_ADAMS_TRANSITION_CT_M2: f64 = 1.0;

/// The transition is fixed in jet-exit-to-freestream pressure ratio across
/// all supersonic conditions tested: P_ej/P∞ ≈ 7.0–7.2 (report Fig. 18,
/// p. 35; Conclusion 3, p. 145), with corresponding C_T in 0.5–3.0.
pub const JARVINEN_ADAMS_TRANSITION_PRESSURE_RATIO_LO: f64 = 7.0;
pub const JARVINEN_ADAMS_TRANSITION_PRESSURE_RATIO_HI: f64 = 7.2;
pub const JARVINEN_ADAMS_TRANSITION_CT_LO: f64 = 0.5;
pub const JARVINEN_ADAMS_TRANSITION_CT_HI: f64 = 3.0;

/// PERIPHERAL-configuration bow-shock rippling onset: "Both Jarvinen and
/// Adams and Keyes and Hefner observed local instabilities affecting the
/// slope of the bow shock as the total thrust coefficient increased beyond
/// approximately 3.0" — Korzun, Cruz & Braun, "A Survey of Supersonic
/// Retropropulsion Technology for Mars Entry, Descent, and Landing," 2008
/// IEEE Aerospace Conf., IEEEAC #1246, p. 6
/// (`papers/korzun_braun_cruz_srp_survey.pdf`). This bound concerns the
/// peripheral nozzle configuration, not the central one the tables above
/// describe; recorded for the future peripheral work.
pub const KEYES_HEFNER_PERIPHERAL_RIPPLE_CT: f64 = 3.0;

// ─────────────────────────────────────────────────────────────────────────
// Cordell analytic SRP plume model — Sibulkin source-flow scaling.
// Cordell, C. E., Jr., Ph.D. dissertation, Georgia Institute of Technology,
// December 2013 (`papers/cordell_2013_srp_analytic.pdf`), Ch. III; journal
// version: Cordell & Braun, JSR 50(4):763–770, 2013.
// ─────────────────────────────────────────────────────────────────────────

/// Sibulkin's axial density-distribution scaling: `B = 0.4·π / ψ∞`
/// (dissertation Eq. (12), p. 79). The 0.4 is Sibulkin's source-flow
/// coefficient.
pub const SIBULKIN_SCALING_COEFFICIENT: f64 = 0.4;

/// Validated freestream Mach envelope of the Cordell plume model
/// (dissertation Tables 7 & 11, pp. 119, 129: M∞ = 2–4).
pub const CORDELL_MACH_ENVELOPE_LO: f64 = 2.0;
pub const CORDELL_MACH_ENVELOPE_HI: f64 = 4.0;

/// Validated jet gamma envelope of the Cordell plume model (dissertation
/// Table 12, p. 130: γ = 1.2–1.4).
pub const CORDELL_GAMMA_ENVELOPE_LO: f64 = 1.2;
pub const CORDELL_GAMMA_ENVELOPE_HI: f64 = 1.4;

// ─────────────────────────────────────────────────────────────────────────
// Real-field accessors, following the house mechanism (constants/mod.rs).
// ─────────────────────────────────────────────────────────────────────────

use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

/// Returns [`JARVINEN_ADAMS_TRANSITION_CT_M2`] at the target precision `R`.
#[inline]
pub fn jarvinen_adams_transition_ct_m2<R: RealField + FromPrimitive>() -> R {
    crate::constants::real_from_f64(JARVINEN_ADAMS_TRANSITION_CT_M2)
}

/// Returns [`JARVINEN_ADAMS_TRANSITION_PRESSURE_RATIO_LO`] at the target precision `R`.
#[inline]
pub fn jarvinen_adams_transition_pressure_ratio_lo<R: RealField + FromPrimitive>() -> R {
    crate::constants::real_from_f64(JARVINEN_ADAMS_TRANSITION_PRESSURE_RATIO_LO)
}

/// Returns [`SIBULKIN_SCALING_COEFFICIENT`] at the target precision `R`.
#[inline]
pub fn sibulkin_scaling_coefficient<R: RealField + FromPrimitive>() -> R {
    crate::constants::real_from_f64(SIBULKIN_SCALING_COEFFICIENT)
}
