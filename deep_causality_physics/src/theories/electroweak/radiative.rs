/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::PhysicsError;
use crate::constants::FERMI_CONSTANT;
use std::f64::consts::PI;

/// Container for computed radiative corrections
#[derive(Debug, Clone, Copy, Default)]
pub struct RadiativeCorrections {
    /// Veltman screening correction (Δρ) from top/bottom mass splitting
    pub delta_rho: f64,
    /// Standard Radiative Correction Factor (Δr) referenced to α(0)
    pub delta_r: f64,
    /// Loop-corrected W boson mass
    pub w_mass_corrected: f64,
    /// Effective Sin²θ_W (for Z-pole decay widths)
    pub sin2_theta_eff: f64,
}

/// Calculates the Veltman screening correction (Δρ)
///
/// This correction arises from the large mass splitting between the top and bottom quarks
/// in vacuum polarization loops. It is the dominant contribution to the ρ parameter deviation.
///
/// # Formula
/// ```text
/// Δρ = (3 · G_F · m_t²) / (8 · π² · √2)
/// ```
pub fn calculate_delta_rho(top_mass: f64) -> f64 {
    let numerator = 3.0 * FERMI_CONSTANT * top_mass * top_mass;
    let denominator = 8.0 * PI * PI * 2.0_f64.sqrt();
    numerator / denominator
}

/// Calculates the Weak part of the radiative correction (Δr_weak)
///
/// When using α(M_Z), the large QED running (Δα) is absorbed.
/// The remaining correction is dominated by Veltman screening.
///
/// # Formula
/// ```text
/// Δr_weak ≈ - (cos²θ_W / sin²θ_W) · Δρ
/// ```
pub fn calculate_delta_r_weak(sin2_theta_w: f64, delta_rho: f64) -> f64 {
    let cos2_theta_w = 1.0 - sin2_theta_w;
    let cot2_theta_w = cos2_theta_w / sin2_theta_w;
    -cot2_theta_w * delta_rho
}

/// Calculates the Effective Weak Mixing Angle
///
/// Used for Z-pole asymmetries and partial widths.
///
/// # Formula
/// ```text
/// sin²θ_eff ≈ sin²θ_W + cos²θ_W · Δρ
/// ```
pub fn calculate_effective_angle(sin2_theta_w: f64, delta_rho: f64) -> f64 {
    let cos2 = 1.0 - sin2_theta_w;
    sin2_theta_w + cos2 * delta_rho
}

/// Iteratively solves for the physical W boson mass using High-Energy Inputs.
///
/// USAGE NOTE: This function takes α(M_Z) as the primary coupling input to ensure
/// high precision in the mass calculation, avoiding errors from manual Δα reconstruction.
///
/// The Solver Equation (renormalized to M_Z scale):
/// ```text
/// M_W² (1 - M_W²/M_Z²) = (π · α(M_Z)) / (√2 · G_F · (1 - Δr_weak))
/// ```
///
/// Returns a `RadiativeCorrections` struct containing the *Standard* Δr (referenced to α(0))
/// for consistency with standard reporting.
pub fn solve_w_mass(
    mz: f64,
    top_mass: f64,
    alpha_mz: f64,
    alpha_0: f64, // Used only for reporting Standard Delta R
    g_f: f64,
) -> Result<RadiativeCorrections, PhysicsError> {
    // 1. Calculate Δρ (depends only on m_t, G_F)
    let delta_rho = calculate_delta_rho(top_mass);

    // 2. Initial guess
    let mut mw = 80.30;
    let target_accuracy = 1e-6;
    let max_iters = 20;

    let mut delta_r_weak = 0.0;
    let mut sin2_eff = 0.0;
    let mut sin2_on_shell;

    for _ in 0..max_iters {
        // Calculate On-Shell sin²θ_W from current M_W
        sin2_on_shell = 1.0 - (mw * mw) / (mz * mz);

        // Update Weak Correction (Δr_weak)
        delta_r_weak = calculate_delta_r_weak(sin2_on_shell, delta_rho);

        // Update Effective Angle for report
        sin2_eff = calculate_effective_angle(sin2_on_shell, delta_rho);

        // Calculate RHS constant A using ALPHA_MZ
        // A = (π · α(M_Z)) / (√2 · G_F · (1 - Δr_weak))
        let numerator = PI * alpha_mz;
        let denominator = 2.0_f64.sqrt() * g_f * (1.0 - delta_r_weak);
        let a = numerator / denominator;

        // Solve Quadratic for M_W^2
        let mz2 = mz * mz;
        let discriminant = mz2 * mz2 - 4.0 * mz2 * a;

        if discriminant < 0.0 {
            return Err(PhysicsError::NumericalInstability(
                "Radiative correction solver failed: Negative discriminant".into(),
            ));
        }

        let mw2_new = (mz2 + discriminant.sqrt()) / 2.0;
        let mw_new = mw2_new.sqrt();

        // Check convergence
        if (mw_new - mw).abs() < target_accuracy {
            mw = mw_new;
            break;
        }

        mw = mw_new;
    }

    // 3. Reconstruct Standard Delta R (referenced to alpha(0)) for UI Consistency
    // The loop condition is: 1 - Δr_std = (α(0) / α(M_Z)) * (1 - Δr_weak)
    // So: Δr_std = 1 - (α(0) / α(M_Z)) * (1 - Δr_weak)
    let term = (alpha_0 / alpha_mz) * (1.0 - delta_r_weak);
    let delta_r_std = 1.0 - term;

    Ok(RadiativeCorrections {
        delta_rho,
        delta_r: delta_r_std,
        w_mass_corrected: mw,
        sin2_theta_eff: sin2_eff,
    })
}
