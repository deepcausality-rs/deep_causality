/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Hypersonic reacting-air / Park two-temperature model coefficients for the
//! Gap-2 (Tier-A) plasma-blackout slice.
//!
//! Primary sources (PDFs in `deep_causality_physics/papers/`):
//! * Gupta, Yos, Thompson & Lee, "A Review of Reaction Rates and Thermodynamic
//!   and Transport Properties for an 11-Species Air Model for Chemical and
//!   Thermal Nonequilibrium Calculations to 30000 K," NASA RP-1232 (1990).
//!   `papers/gupta_1990_nasa_rp1232.pdf` — Table II (reaction rates), eq. 3a/5b.
//! * Millikan & White, "Systematics of Vibrational Relaxation," J. Chem. Phys.
//!   39, 3209 (1963) — the τ_vt correlation.
//! * Park, "Nonequilibrium Hypersonic Aerothermodynamics," Wiley (1990) — the
//!   two-temperature model and the high-temperature vibrational limiting
//!   correction.
//! * Park, "Review of Chemical-Kinetic Problems of Future NASA Missions, I:
//!   Earth Entries," J. Thermophys. Heat Transfer 7(3):385 (1993).
//!
//! Note on rate-set sensitivity: published air-chemistry rate sets disagree on
//! the associative-ionization pre-exponential/exponent (Gupta RP-1232:
//! Cf = 9.03e9, η = 0.5, θd = 32,400 K; Park lineage: ~8.8e8, η ≈ 0.5,
//! θd = 31,900 K; Dunn–Kang: 5.3e12, η = 0, θd = 17,778 K). The Tier-A slice
//! uses the **RP-1232 primary-table values** (verified from the downloaded PDF)
//! and the verification tolerance is set wide enough to absorb this rate-set
//! sensitivity (see `add-park2t-blackout-tier-a`).

// ─────────────────────────────────────────────────────────────────────────
// Associative ionization N + O ⇌ NO⁺ + e⁻ (the dominant low-velocity channel).
// Arrhenius forward rate k_f = Cf · T^η · exp(−θd / T), controlled by the
// heavy-particle translational temperature T.
// Source: NASA RP-1232 (Gupta et al. 1990), Table II, reaction 7.
// ─────────────────────────────────────────────────────────────────────────

/// Pre-exponential factor `Cf` for N + O ⇌ NO⁺ + e⁻. Unit: cm³·mol⁻¹·s⁻¹.
pub const PARK_NO_IONIZATION_PREFACTOR: f64 = 9.03e9;

/// Temperature exponent `η` for N + O ⇌ NO⁺ + e⁻ (dimensionless).
pub const PARK_NO_IONIZATION_EXPONENT: f64 = 0.5;

/// Characteristic (activation) temperature `θd` for N + O ⇌ NO⁺ + e⁻. Unit: K.
pub const PARK_NO_IONIZATION_ACTIVATION_TEMP: f64 = 32_400.0;

/// First ionization energy of NO (the dominant air ionization channel), used by
/// the Saha-equilibrium target. Unit: eV. (NO → NO⁺ + e⁻; ≈ 9.26 eV.)
pub const NO_IONIZATION_ENERGY_EV: f64 = 9.26;

// ─────────────────────────────────────────────────────────────────────────
// Millikan–White vibrational relaxation correlation:
//   τ_sr · P = exp[ A_sr · (T^(−1/3) − B · μ_sr^(1/4)) − C ]   (P in atm, τ in s)
// with A_sr = MW_A_COEFFICIENT · μ_sr^(1/2) · θ_v^(4/3), μ_sr in amu, θ_v in K.
// The natural-log constants below are the base-10 originals (5.0e-4, 0.015, 8.00)
// converted via ×ln(10): 5.0e-4·ln10 = 1.16e-3 and 8.00·ln10 = 18.42.
// Source: Millikan & White (1963); Park (1990) rearrangement.
// ─────────────────────────────────────────────────────────────────────────

/// Millikan–White `A_sr` prefactor coefficient (natural-log form). Combined with
/// `μ_sr^(1/2) · θ_v^(4/3)` to give `A_sr`. Unit: amu⁻¹ᐟ² · K⁻⁴ᐟ³ (dimensional bookkeeping).
pub const MILLIKAN_WHITE_A_COEFFICIENT: f64 = 1.16e-3;

/// Millikan–White reduced-mass offset `B` in `(T^(−1/3) − B·μ^(1/4))`. Dimensionless-ish (amu⁻¹ᐟ⁴).
pub const MILLIKAN_WHITE_MU_OFFSET: f64 = 0.015;

/// Millikan–White additive log constant `C` (natural-log form, = 8.00·ln 10).
pub const MILLIKAN_WHITE_LOG_OFFSET: f64 = 18.42;

// ─────────────────────────────────────────────────────────────────────────
// Park (1990) high-temperature limiting vibrational relaxation, applied as
//   τ_park = 1 / (σ_v · c̄ · N),   σ_v = σ_ref · (T_ref / T)²
// to correct the Millikan–White under-prediction above ~8000 K.
// ─────────────────────────────────────────────────────────────────────────

/// Park limiting vibrational cross-section reference `σ_ref`. Unit: m².
pub const PARK_LIMITING_CROSS_SECTION: f64 = 1.0e-21;

/// Park limiting-cross-section reference temperature `T_ref`. Unit: K.
pub const PARK_LIMITING_REFERENCE_TEMP: f64 = 50_000.0;

// ─────────────────────────────────────────────────────────────────────────
// Characteristic vibrational temperatures θ_v of the principal air species,
// used by the Millikan–White A_sr term. θ_v = h·c·ω_e / k_B.
// Source: Park (1990); standard spectroscopic constants (Vincenti & Kruger).
// ─────────────────────────────────────────────────────────────────────────

/// Characteristic vibrational temperature of N₂. Unit: K.
pub const THETA_VIB_N2: f64 = 3_393.0;

/// Characteristic vibrational temperature of O₂. Unit: K.
pub const THETA_VIB_O2: f64 = 2_273.0;

/// Characteristic vibrational temperature of NO. Unit: K.
pub const THETA_VIB_NO: f64 = 2_739.0;
