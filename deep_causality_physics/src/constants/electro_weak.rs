/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// =============================================================================
// Electroweak Constants (PDG 2024 values)
// =============================================================================

/// Fine structure constant α = e²/(4π) ≈ 1/137 (low-energy limit)
///
/// ```text
/// α = e² / (4π ℏc) ≈ 7.297 × 10⁻³ ≈ 1/137
/// ```
pub const ALPHA_EM: f64 = 1.0 / 137.035999177;

/// Running fine structure constant at Z pole: α(M_Z) ≈ 1/128
///
/// Due to vacuum polarization, α runs with energy scale Q:
/// ```text
/// α(Q) = α(0) / (1 - Δα(Q))
/// α(M_Z) ≈ 1/127.95
/// ```
/// This is the correct value for precision electroweak calculations.
pub const ALPHA_EM_MZ: f64 = 1.0 / 127.95;

/// Electromagnetic coupling constant e = √(4πα) ≈ 0.303 (low-energy)
///
/// ```text
/// e = g sin θ_W = g' cos θ_W ≈ 0.303
/// ```
pub const EM_COUPLING: f64 = 0.3028221;

/// Running EM coupling at Z pole e(M_Z) = √(4π α(M_Z)) ≈ 0.313
///
/// This coupling should be used for precision electroweak calculations
/// to obtain correct W and Z boson masses from M_W = g v / 2.
pub const EM_COUPLING_MZ: f64 = 0.31343;

/// Higgs boson mass M_H in GeV
///
/// ```text
/// M_H = √(2λ) v ≈ 125 GeV
/// ```
pub const HIGGS_MASS: f64 = 125.10;

/// Top quark mass m_t in GeV (heaviest Standard Model fermion)
///
/// ```text
/// m_t = y_t v / √2 ≈ 173 GeV
/// ```
pub const TOP_MASS: f64 = 172.52;

// =============================================================================
// Z Resonance & Conversion Factors
// =============================================================================

/// Z boson total width Γ_Z in GeV (PDG 2024)
pub const Z_WIDTH: f64 = 2.4952;

/// Z boson partial width to electrons Γ_ee in GeV (PDG 2024)
pub const Z_PARTIAL_WIDTH_EE: f64 = 0.08391;

/// Z boson hadronic partial width Γ_had in GeV (PDG 2024)
pub const Z_PARTIAL_WIDTH_HAD: f64 = 1.7444;

/// Unit conversion factor (ℏc)² to convert GeV⁻² to nanobarns (nb)
///
/// ```text
/// 1 GeV⁻² ≈ 0.389379 mb = 389379 nb
/// ```
pub const GEV2_TO_NB: f64 = 389379.366;

/// Unit conversion factor (ℏc)² to convert GeV⁻² to picobarns (pb)
///
/// ```text
/// 1 GeV⁻² ≈ 389,379,366 pb
/// ```
pub const GEV2_TO_PB: f64 = 389379366.0;
