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
//!   two-temperature model, and the high-temperature vibrational limiting
//!   correction that this module carries constants for but does not apply (see
//!   the `PARK_LIMITING_*` block below).
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
// with A_sr = MILLIKAN_WHITE_A_COEFFICIENT · μ_sr^(1/2) · θ_v^(4/3), μ_sr in
// amu, θ_v in K.
//
// The three constants below are stated in the natural-log form the exponential
// consumes. Only C is a clean rescaling of a base-10 original: 8.00 · ln 10 =
// 18.4207, which rounds to the shipped 18.42. A is not. 5.0e-4 · ln 10 =
// 1.1513e-3, while the shipped A is 1.16e-3; the base-10 equivalent of the
// shipped value is 1.16e-3 / ln 10 = 5.038e-4. Read A and C as independently
// rounded natural-log constants, not as one ×ln 10 conversion of a
// (5.0e-4, 8.00) pair.
// Source: Millikan & White (1963); Park (1990) rearrangement, both listed in
// the module header. Neither is present in `deep_causality_physics/papers/`,
// so no equation number is cited here.
// ─────────────────────────────────────────────────────────────────────────

/// Millikan–White `A_sr` prefactor coefficient (natural-log form). Combined with
/// `μ_sr^(1/2) · θ_v^(4/3)` to give `A_sr`. Unit: amu⁻¹ᐟ² · K⁻⁴ᐟ³ (dimensional bookkeeping).
pub const MILLIKAN_WHITE_A_COEFFICIENT: f64 = 1.16e-3;

/// Millikan–White reduced-mass offset `B` in `(T^(−1/3) − B·μ^(1/4))`. Dimensionless-ish (amu⁻¹ᐟ⁴).
pub const MILLIKAN_WHITE_MU_OFFSET: f64 = 0.015;

/// Millikan–White additive log constant `C` (natural-log form, = 8.00·ln 10).
pub const MILLIKAN_WHITE_LOG_OFFSET: f64 = 18.42;

// ─────────────────────────────────────────────────────────────────────────
// Park (1990) high-temperature limiting vibrational relaxation,
//   τ_park = 1 / (σ_v · c̄ · N),   σ_v = σ_ref · (T_ref / T)²,
// intended as the additive term τ_vt = τ_MW + τ_park that lifts the
// Millikan–White under-prediction of the relaxation time above ~8000 K.
//
// NOT APPLIED. `vibrational_relaxation_kernel`
// (`kernels/hypersonic/thermochemistry.rs`) computes `tau = exponent.exp() /
// pressure_atm` and nothing further; neither constant below has a call site
// anywhere in the workspace, and the mean thermal speed
// c̄ = sqrt(8·k_B·T / (π·m)) the term requires is never formed. The shipped
// τ_vt is the uncorrected Millikan–White value. Bias direction: τ_park is
// additive and positive, so omitting it makes τ_vt too short above ~8000 K,
// T_ve chases T_tr faster than the Park two-temperature model intends, and the
// downstream T_a = sqrt(T_tr · T_ve) and n_e come out high. Magnitude at the
// RAM-C post-shock state (T = 8044 K, n ≈ 2.6e22 m⁻³): σ_v ≈ 3.9e-20 m²,
// c̄ ≈ 2.7e3 m/s, τ_park ≈ 3.6e-7 s against τ_MW ≈ 1.9e-5 s, about two
// percent. The ratio grows with temperature, since τ_MW falls exponentially in
// T^(−1/3) while τ_park rises with T at fixed number density.
//
// The two constants are retained as the reference values for the correction
// when it is implemented; they are dead today.
// ─────────────────────────────────────────────────────────────────────────

/// Park limiting vibrational cross-section reference `σ_ref`. Unit: m².
/// Unused: see the block comment above, the correction is not applied.
pub const PARK_LIMITING_CROSS_SECTION: f64 = 1.0e-21;

/// Park limiting-cross-section reference temperature `T_ref`. Unit: K.
/// Unused: see the block comment above, the correction is not applied.
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

// ─────────────────────────────────────────────────────────────────────────
// Finite-rate ionization network (RP-1232 Table II, page 46). Table II pairs
// each forward rate with its backward rate; the source's eq. (5a) is the
// detailed-balance relation k_b = k_f / K_eq, so K_eq = k_f / k_b from one
// table row. The source states the pairs are valid for flight velocities up
// to about 8 km/s. All rates are Arrhenius forms k = Cf · T^η · exp(−θ/T) in
// cm³·mol⁻¹·s⁻¹ (two-body) or cm⁶·mol⁻²·s⁻¹ (three-body); the third-body
// concentration multiplies the three-body forms at the call site.
// Source: Gupta, Yos, Thompson & Lee, NASA RP-1232 (1990), Table II,
// verified from `papers/gupta_1990_nasa_rp1232.pdf` (rendered page 46).
// ─────────────────────────────────────────────────────────────────────────

// Reaction 7 backward: NO⁺ + e⁻ → N + O (dissociative recombination), the
// two-body reverse of the shipped associative-ionization channel. Rated at
// the electron temperature in the two-temperature model.

/// Pre-exponential factor for NO⁺ + e⁻ → N + O. Unit: cm³·mol⁻¹·s⁻¹.
pub const RP1232_NO_DR_PREFACTOR: f64 = 1.80e19;

/// Temperature exponent for NO⁺ + e⁻ → N + O (dimensionless).
pub const RP1232_NO_DR_EXPONENT: f64 = -1.0;

/// Activation temperature for NO⁺ + e⁻ → N + O (barrier-free). Unit: K.
pub const RP1232_NO_DR_ACTIVATION_TEMP: f64 = 0.0;

// Reaction 8 forward: O + e⁻ → O⁺ + e⁻ + e⁻ (electron-impact ionization).
// Table II states the central value with a ±33 percent spread; the source
// notes (page 10) these rates come from expansion-flow data and tend to be
// lower than compressive-flow data. Both are absorbed by the validation band.

/// Pre-exponential factor for O + e⁻ → O⁺ + 2e⁻. Unit: cm³·mol⁻¹·s⁻¹.
pub const RP1232_EI_O_PREFACTOR: f64 = 3.6e31;

/// Temperature exponent for O + e⁻ → O⁺ + 2e⁻ (dimensionless).
pub const RP1232_EI_O_EXPONENT: f64 = -2.91;

/// Activation temperature for O + e⁻ → O⁺ + 2e⁻. Unit: K.
pub const RP1232_EI_O_ACTIVATION_TEMP: f64 = 1.58e5;

// Reaction 9 forward: N + e⁻ → N⁺ + e⁻ + e⁻ (electron-impact ionization).

/// Pre-exponential factor for N + e⁻ → N⁺ + 2e⁻. Unit: cm³·mol⁻¹·s⁻¹.
pub const RP1232_EI_N_PREFACTOR: f64 = 1.1e32;

/// Temperature exponent for N + e⁻ → N⁺ + 2e⁻ (dimensionless). Table II
/// states −3.14; written as a quotient because the raw literal trips
/// `clippy::approx_constant` (it is a temperature exponent, not π).
pub const RP1232_EI_N_EXPONENT: f64 = -314.0 / 100.0;

/// Activation temperature for N + e⁻ → N⁺ + 2e⁻. Unit: K.
pub const RP1232_EI_N_ACTIVATION_TEMP: f64 = 1.69e5;

// Reaction 1: O₂ + M ⇌ 2O + M (dissociation forward, three-body
// recombination backward). Forward in cm³·mol⁻¹·s⁻¹ (after the third-body
// concentration multiplies once), backward in cm⁶·mol⁻²·s⁻¹.

/// Pre-exponential factor for O₂ + M → 2O + M. Unit: cm³·mol⁻¹·s⁻¹.
pub const RP1232_O2_DISS_PREFACTOR: f64 = 3.61e18;

/// Temperature exponent for O₂ + M → 2O + M (dimensionless).
pub const RP1232_O2_DISS_EXPONENT: f64 = -1.0;

/// Activation temperature for O₂ + M → 2O + M. Unit: K.
pub const RP1232_O2_DISS_ACTIVATION_TEMP: f64 = 5.94e4;

/// Pre-exponential factor for 2O + M → O₂ + M. Unit: cm⁶·mol⁻²·s⁻¹.
pub const RP1232_O2_RECOMB_PREFACTOR: f64 = 3.01e15;

/// Temperature exponent for 2O + M → O₂ + M (dimensionless).
pub const RP1232_O2_RECOMB_EXPONENT: f64 = -0.5;

// Reaction 2: N₂ + M ⇌ 2N + M.

/// Pre-exponential factor for N₂ + M → 2N + M. Unit: cm³·mol⁻¹·s⁻¹.
pub const RP1232_N2_DISS_PREFACTOR: f64 = 1.92e17;

/// Temperature exponent for N₂ + M → 2N + M (dimensionless).
pub const RP1232_N2_DISS_EXPONENT: f64 = -0.5;

/// Activation temperature for N₂ + M → 2N + M. Unit: K.
pub const RP1232_N2_DISS_ACTIVATION_TEMP: f64 = 1.131e5;

/// Pre-exponential factor for 2N + M → N₂ + M. Unit: cm⁶·mol⁻²·s⁻¹.
pub const RP1232_N2_RECOMB_PREFACTOR: f64 = 1.09e16;

/// Temperature exponent for 2N + M → N₂ + M (dimensionless).
pub const RP1232_N2_RECOMB_EXPONENT: f64 = -0.5;

// ─────────────────────────────────────────────────────────────────────────
// Standard-air elemental composition for the atom-pool closure (mole
// fractions of the undissociated diatomics; trace species folded into N₂).
// Source: U.S. Standard Atmosphere 1976 (N₂ 0.78084, O₂ 0.20946; the ~1
// percent Ar and trace gases are folded into the inert N₂ share here).
// ─────────────────────────────────────────────────────────────────────────

/// Mole fraction of N₂ in undissociated standard air (traces folded in).
pub const AIR_N2_MOLE_FRACTION: f64 = 0.79;

/// Mole fraction of O₂ in undissociated standard air.
pub const AIR_O2_MOLE_FRACTION: f64 = 0.21;

// ─────────────────────────────────────────────────────────────────────────
// Real-field accessors for the finite-rate network coefficients, following
// the house mechanism (see `constants/condensed.rs`): each `f64` constant
// has a companion function returning it at the target precision `R`.
// ─────────────────────────────────────────────────────────────────────────

use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

/// Returns [`RP1232_NO_DR_PREFACTOR`] at the target real-field precision `R`.
#[inline]
pub fn rp1232_no_dr_prefactor<R: RealField + FromPrimitive>() -> R {
    crate::constants::real_from_f64(RP1232_NO_DR_PREFACTOR)
}

/// Returns [`RP1232_NO_DR_EXPONENT`] at the target real-field precision `R`.
#[inline]
pub fn rp1232_no_dr_exponent<R: RealField + FromPrimitive>() -> R {
    crate::constants::real_from_f64(RP1232_NO_DR_EXPONENT)
}

/// Returns [`RP1232_NO_DR_ACTIVATION_TEMP`] at the target real-field precision `R`.
#[inline]
pub fn rp1232_no_dr_activation_temp<R: RealField + FromPrimitive>() -> R {
    crate::constants::real_from_f64(RP1232_NO_DR_ACTIVATION_TEMP)
}

/// Returns [`RP1232_EI_O_PREFACTOR`] at the target real-field precision `R`.
#[inline]
pub fn rp1232_ei_o_prefactor<R: RealField + FromPrimitive>() -> R {
    crate::constants::real_from_f64(RP1232_EI_O_PREFACTOR)
}

/// Returns [`RP1232_EI_O_EXPONENT`] at the target real-field precision `R`.
#[inline]
pub fn rp1232_ei_o_exponent<R: RealField + FromPrimitive>() -> R {
    crate::constants::real_from_f64(RP1232_EI_O_EXPONENT)
}

/// Returns [`RP1232_EI_O_ACTIVATION_TEMP`] at the target real-field precision `R`.
#[inline]
pub fn rp1232_ei_o_activation_temp<R: RealField + FromPrimitive>() -> R {
    crate::constants::real_from_f64(RP1232_EI_O_ACTIVATION_TEMP)
}

/// Returns [`RP1232_EI_N_PREFACTOR`] at the target real-field precision `R`.
#[inline]
pub fn rp1232_ei_n_prefactor<R: RealField + FromPrimitive>() -> R {
    crate::constants::real_from_f64(RP1232_EI_N_PREFACTOR)
}

/// Returns [`RP1232_EI_N_EXPONENT`] at the target real-field precision `R`.
#[inline]
pub fn rp1232_ei_n_exponent<R: RealField + FromPrimitive>() -> R {
    crate::constants::real_from_f64(RP1232_EI_N_EXPONENT)
}

/// Returns [`RP1232_EI_N_ACTIVATION_TEMP`] at the target real-field precision `R`.
#[inline]
pub fn rp1232_ei_n_activation_temp<R: RealField + FromPrimitive>() -> R {
    crate::constants::real_from_f64(RP1232_EI_N_ACTIVATION_TEMP)
}

/// Returns [`RP1232_O2_DISS_PREFACTOR`] at the target real-field precision `R`.
#[inline]
pub fn rp1232_o2_diss_prefactor<R: RealField + FromPrimitive>() -> R {
    crate::constants::real_from_f64(RP1232_O2_DISS_PREFACTOR)
}

/// Returns [`RP1232_O2_DISS_EXPONENT`] at the target real-field precision `R`.
#[inline]
pub fn rp1232_o2_diss_exponent<R: RealField + FromPrimitive>() -> R {
    crate::constants::real_from_f64(RP1232_O2_DISS_EXPONENT)
}

/// Returns [`RP1232_O2_DISS_ACTIVATION_TEMP`] at the target real-field precision `R`.
#[inline]
pub fn rp1232_o2_diss_activation_temp<R: RealField + FromPrimitive>() -> R {
    crate::constants::real_from_f64(RP1232_O2_DISS_ACTIVATION_TEMP)
}

/// Returns [`RP1232_O2_RECOMB_PREFACTOR`] at the target real-field precision `R`.
#[inline]
pub fn rp1232_o2_recomb_prefactor<R: RealField + FromPrimitive>() -> R {
    crate::constants::real_from_f64(RP1232_O2_RECOMB_PREFACTOR)
}

/// Returns [`RP1232_O2_RECOMB_EXPONENT`] at the target real-field precision `R`.
#[inline]
pub fn rp1232_o2_recomb_exponent<R: RealField + FromPrimitive>() -> R {
    crate::constants::real_from_f64(RP1232_O2_RECOMB_EXPONENT)
}

/// Returns [`RP1232_N2_DISS_PREFACTOR`] at the target real-field precision `R`.
#[inline]
pub fn rp1232_n2_diss_prefactor<R: RealField + FromPrimitive>() -> R {
    crate::constants::real_from_f64(RP1232_N2_DISS_PREFACTOR)
}

/// Returns [`RP1232_N2_DISS_EXPONENT`] at the target real-field precision `R`.
#[inline]
pub fn rp1232_n2_diss_exponent<R: RealField + FromPrimitive>() -> R {
    crate::constants::real_from_f64(RP1232_N2_DISS_EXPONENT)
}

/// Returns [`RP1232_N2_DISS_ACTIVATION_TEMP`] at the target real-field precision `R`.
#[inline]
pub fn rp1232_n2_diss_activation_temp<R: RealField + FromPrimitive>() -> R {
    crate::constants::real_from_f64(RP1232_N2_DISS_ACTIVATION_TEMP)
}

/// Returns [`RP1232_N2_RECOMB_PREFACTOR`] at the target real-field precision `R`.
#[inline]
pub fn rp1232_n2_recomb_prefactor<R: RealField + FromPrimitive>() -> R {
    crate::constants::real_from_f64(RP1232_N2_RECOMB_PREFACTOR)
}

/// Returns [`RP1232_N2_RECOMB_EXPONENT`] at the target real-field precision `R`.
#[inline]
pub fn rp1232_n2_recomb_exponent<R: RealField + FromPrimitive>() -> R {
    crate::constants::real_from_f64(RP1232_N2_RECOMB_EXPONENT)
}

/// Returns [`AIR_N2_MOLE_FRACTION`] at the target real-field precision `R`.
#[inline]
pub fn air_n2_mole_fraction<R: RealField + FromPrimitive>() -> R {
    crate::constants::real_from_f64(AIR_N2_MOLE_FRACTION)
}

/// Returns [`AIR_O2_MOLE_FRACTION`] at the target real-field precision `R`.
#[inline]
pub fn air_o2_mole_fraction<R: RealField + FromPrimitive>() -> R {
    crate::constants::real_from_f64(AIR_O2_MOLE_FRACTION)
}

/// Returns [`PARK_NO_IONIZATION_PREFACTOR`] at the target real-field precision `R`.
#[inline]
pub fn park_no_ionization_prefactor<R: RealField + FromPrimitive>() -> R {
    crate::constants::real_from_f64(PARK_NO_IONIZATION_PREFACTOR)
}

/// Returns [`PARK_NO_IONIZATION_EXPONENT`] at the target real-field precision `R`.
#[inline]
pub fn park_no_ionization_exponent<R: RealField + FromPrimitive>() -> R {
    crate::constants::real_from_f64(PARK_NO_IONIZATION_EXPONENT)
}

/// Returns [`PARK_NO_IONIZATION_ACTIVATION_TEMP`] at the target real-field precision `R`.
#[inline]
pub fn park_no_ionization_activation_temp<R: RealField + FromPrimitive>() -> R {
    crate::constants::real_from_f64(PARK_NO_IONIZATION_ACTIVATION_TEMP)
}

// Reaction 6: N₂ + O ⇌ NO + N (Zeldovich exchange), the low-activation
// N-atom production path that feeds associative ionization before direct
// N₂ dissociation wakes up. Source: RP-1232 Table II, reaction 6 (verified
// from the rendered page 46).

/// Pre-exponential factor for N₂ + O → NO + N. Unit: cm³·mol⁻¹·s⁻¹.
pub const RP1232_ZELDOVICH_PREFACTOR: f64 = 6.75e13;

/// Temperature exponent for N₂ + O → NO + N (dimensionless).
pub const RP1232_ZELDOVICH_EXPONENT: f64 = 0.0;

/// Activation temperature for N₂ + O → NO + N. Unit: K.
pub const RP1232_ZELDOVICH_ACTIVATION_TEMP: f64 = 3.75e4;

/// Park's classic controlling-temperature exponent for **dissociation**,
/// `T_q = T^q · T_v^(1−q)` with `q = 0.7` (Park 1990; the geometric mean
/// `q = 0.5` is the alternative). The controlling-temperature choice is the
/// largest closure divergence among production codes (DPLR/LAURA/US3D); this
/// model adopts the Park lineage's own published exponent for the Park rate
/// set. The *ionization* controller keeps the calibrated geometric mean.
pub const PARK_DISSOCIATION_Q: f64 = 0.7;

/// Returns [`RP1232_ZELDOVICH_PREFACTOR`] at the target real-field precision `R`.
#[inline]
pub fn rp1232_zeldovich_prefactor<R: RealField + FromPrimitive>() -> R {
    crate::constants::real_from_f64(RP1232_ZELDOVICH_PREFACTOR)
}

/// Returns [`RP1232_ZELDOVICH_EXPONENT`] at the target real-field precision `R`.
#[inline]
pub fn rp1232_zeldovich_exponent<R: RealField + FromPrimitive>() -> R {
    crate::constants::real_from_f64(RP1232_ZELDOVICH_EXPONENT)
}

/// Returns [`RP1232_ZELDOVICH_ACTIVATION_TEMP`] at the target real-field precision `R`.
#[inline]
pub fn rp1232_zeldovich_activation_temp<R: RealField + FromPrimitive>() -> R {
    crate::constants::real_from_f64(RP1232_ZELDOVICH_ACTIVATION_TEMP)
}

/// Returns [`PARK_DISSOCIATION_Q`] at the target real-field precision `R`.
#[inline]
pub fn park_dissociation_q<R: RealField + FromPrimitive>() -> R {
    crate::constants::real_from_f64(PARK_DISSOCIATION_Q)
}
