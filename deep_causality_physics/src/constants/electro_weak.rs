/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// =============================================================================
// Electroweak Constants (PDG 2024 values)
// =============================================================================

/// Fine structure constant α = e²/(4π) ≈ 1/137
///
/// ```text
/// α = e² / (4π ℏc) ≈ 7.297 × 10⁻³ ≈ 1/137
/// ```
pub const ALPHA_EM: f64 = 1.0 / 137.035999084;

/// Electromagnetic coupling constant e = √(4πα)
///
/// ```text
/// e = g sin θ_W = g' cos θ_W ≈ 0.303
/// ```
pub const EM_COUPLING: f64 = 0.3028221;

/// Higgs boson mass M_H in GeV
///
/// ```text
/// M_H = √(2λ) v ≈ 125 GeV
/// ```
pub const HIGGS_MASS: f64 = 125.25;

/// Top quark mass m_t in GeV (heaviest Standard Model fermion)
///
/// ```text
/// m_t = y_t v / √2 ≈ 173 GeV
/// ```
pub const TOP_MASS: f64 = 172.69;
