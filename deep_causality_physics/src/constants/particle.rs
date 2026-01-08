/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// =============================================================================
// Weak Force / Particle Physics Constants (PDG 2024 values)
// =============================================================================

/// Fermi coupling constant G_F in GeV⁻²
///
/// ```text
/// G_F / √2 = g² / (8 M_W²) ≈ 1.166 × 10⁻⁵ GeV⁻²
/// ```
pub const FERMI_CONSTANT: f64 = 1.1663787e-5;

/// W boson mass in GeV
///
/// ```text
/// M_W = g v / 2 ≈ 80.4 GeV
/// ```
pub const W_MASS: f64 = 80.377;

/// Z boson mass in GeV
///
/// ```text
/// M_Z = M_W / cos θ_W ≈ 91.2 GeV
/// ```
pub const Z_MASS: f64 = 91.1876;

/// Weak mixing angle (sin²θ_W)
///
/// ```text
/// sin² θ_W = 1 - (M_W / M_Z)² ≈ 0.231
/// ```
pub const SIN2_THETA_W: f64 = 0.23121;

/// Higgs vacuum expectation value v = (√2 G_F)^(-1/2) ≈ 246 GeV
///
/// ```text
/// v = (√2 G_F)^{-1/2} = 2 M_W / g ≈ 246 GeV
/// ```
pub const HIGGS_VEV: f64 = 246.22;
