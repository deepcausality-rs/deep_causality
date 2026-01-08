/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::PhysicsError;
use crate::constants::FERMI_CONSTANT;
use deep_causality_num::RealField;
use std::f64::consts::PI;

/// Container for computed radiative corrections
#[derive(Debug, Clone, Copy, Default)]
pub struct RadiativeCorrections<T> {
    /// Veltman screening correction (Δρ) from top/bottom mass splitting
    pub delta_rho: T,
    /// Standard Radiative Correction Factor (Δr) referenced to α(0)
    pub delta_r: T,
    /// Loop-corrected W boson mass
    pub w_mass_corrected: T,
    /// Effective Sin²θ_W (for Z-pole decay widths)
    pub sin2_theta_eff: T,
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
pub fn calculate_delta_rho<T>(top_mass: T) -> T
where
    T: RealField + From<f64>,
{
    let gf = <T as From<f64>>::from(FERMI_CONSTANT);
    let pi = <T as From<f64>>::from(PI);
    let sqrt2 = <T as From<f64>>::from(2.0).sqrt();

    let numerator = <T as From<f64>>::from(3.0) * gf * top_mass * top_mass;
    let denominator = <T as From<f64>>::from(8.0) * pi * pi * sqrt2;
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
pub fn calculate_delta_r_weak<T>(sin2_theta_w: T, delta_rho: T) -> T
where
    T: RealField + From<f64>,
{
    let one = <T as From<f64>>::from(1.0);
    let cos2_theta_w = one - sin2_theta_w;
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
pub fn calculate_effective_angle<T>(sin2_theta_w: T, delta_rho: T) -> T
where
    T: RealField + From<f64>,
{
    let one = <T as From<f64>>::from(1.0);
    let cos2 = one - sin2_theta_w;
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
pub fn solve_w_mass<T>(
    mz: T,
    top_mass: T,
    alpha_mz: T,
    alpha_0: T, // Used only for reporting Standard Delta R
    g_f: T,
) -> Result<RadiativeCorrections<T>, PhysicsError>
where
    T: RealField + From<f64>,
{
    // 1. Calculate Δρ (depends only on m_t, G_F)
    let delta_rho = calculate_delta_rho(top_mass);

    // 2. Initial guess
    let mut mw = <T as From<f64>>::from(80.30);
    let target_accuracy = <T as From<f64>>::from(1e-6);
    let max_iters = 20;

    let mut delta_r_weak = <T as From<f64>>::from(0.0);
    let mut sin2_eff = <T as From<f64>>::from(0.0);
    // let mut sin2_on_shell; // Removed unused variable warning if detected, but it's used in loop.

    for _ in 0..max_iters {
        // Calculate On-Shell sin²θ_W from current M_W
        let sin2_on_shell = <T as From<f64>>::from(1.0) - (mw * mw) / (mz * mz);

        // Update Weak Correction (Δr_weak)
        delta_r_weak = calculate_delta_r_weak(sin2_on_shell, delta_rho);

        // Update Effective Angle for report
        sin2_eff = calculate_effective_angle(sin2_on_shell, delta_rho);

        // Calculate RHS constant A using ALPHA_MZ
        // A = (π · α(M_Z)) / (√2 · G_F · (1 - Δr_weak))
        let pi = <T as From<f64>>::from(PI);
        let sqrt2 = <T as From<f64>>::from(2.0).sqrt();

        let numerator = pi * alpha_mz;
        let denominator = sqrt2 * g_f * (<T as From<f64>>::from(1.0) - delta_r_weak);
        let a = numerator / denominator;

        // Solve Quadratic for M_W^2
        let mz2 = mz * mz;
        let discriminant = mz2 * mz2 - <T as From<f64>>::from(4.0) * mz2 * a;

        if discriminant < <T as From<f64>>::from(0.0) {
            return Err(PhysicsError::NumericalInstability(
                "Radiative correction solver failed: Negative discriminant".into(),
            ));
        }

        let mw2_new = (mz2 + discriminant.sqrt()) / <T as From<f64>>::from(2.0);
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
    let term = (alpha_0 / alpha_mz) * (<T as From<f64>>::from(1.0) - delta_r_weak);
    let delta_r_std = <T as From<f64>>::from(1.0) - term;

    Ok(RadiativeCorrections {
        delta_rho,
        delta_r: delta_r_std,
        w_mass_corrected: mw,
        sin2_theta_eff: sin2_eff,
    })
}
