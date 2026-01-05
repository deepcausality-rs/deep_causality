/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Advanced GR Metrics
//!
//! Provides constructors for:
//! - **Minkowski**: Flat spacetime
//! - **Schwarzschild**: Static spherical black hole
//! - **Kerr Metric**: Rotating black hole
//! - **FLRW Metric**: Cosmology

use crate::PhysicsError;
use deep_causality_metric::{EastCoastMetric, LorentzianMetric};
use deep_causality_tensor::CausalTensor;
use std::f64::consts::PI;

/// Creates a Minkowski (flat) spacetime metric.
///
/// # Mathematical Definition
/// ```text
/// η_μν = diag(-1, +1, +1, +1)  (East Coast convention)
/// ```
pub fn minkowski_metric() -> CausalTensor<f64> {
    let metric = EastCoastMetric::minkowski_4d().into_metric();
    // Get diagonal form from metric convention
    // East Coast is (-1, 1, 1, 1)
    let sign_0 = metric.sign_of_sq(0) as f64;
    let sign_1 = metric.sign_of_sq(1) as f64;
    let sign_2 = metric.sign_of_sq(2) as f64;
    let sign_3 = metric.sign_of_sq(3) as f64;

    let data = vec![
        sign_0, 0.0, 0.0, 0.0, 0.0, sign_1, 0.0, 0.0, 0.0, 0.0, sign_2, 0.0, 0.0, 0.0, 0.0, sign_3,
    ];
    CausalTensor::from_vec(data, &[4, 4])
}

/// Computes the Kerr metric components (rotating black hole) in Boyer-Lindquist coordinates.
///
/// # Mathematical Definition
/// ```text
/// ds² = -(1 - 2Mr/Σ)dt² - (4aMr sin²θ/Σ)dtdφ + (Σ/Δ)dr² + Σdθ² + (r² + a² + 2Ma²r sin²θ/Σ)sin²θ dφ²
/// ```
/// where:
/// - Σ = r² + a² cos²θ
/// - Δ = r² - 2Mr + a²
///
/// # Arguments
/// * `mass` - Mass M
/// * `a` - Spin parameter J/M
/// * `r`, `theta` - Coordinates
///
/// # Returns
/// 4x4 Metric Tensor [t, r, θ, φ]
pub fn kerr_metric_at(
    mass: f64,
    a: f64,
    r: f64,
    theta: f64,
) -> Result<CausalTensor<f64>, PhysicsError> {
    if r < 0.0 {
        return Err(PhysicsError::DimensionMismatch(
            "Radius must be non-negative".into(),
        ));
    }

    let r2 = r * r;
    let a2 = a * a;
    let cos_theta = theta.cos();
    let sin_theta = theta.sin();
    let sin2_theta = sin_theta * sin_theta;
    let cos2_theta = cos_theta * cos_theta;

    let sigma = r2 + a2 * cos2_theta;
    let delta = r2 - 2.0 * mass * r + a2;

    if delta.abs() < 1e-10 {
        return Err(PhysicsError::NumericalInstability(
            "Coordinate singularity at horizon".into(),
        ));
    }
    if sigma.abs() < 1e-10 {
        return Err(PhysicsError::NumericalInstability(
            "Ring singularity".into(),
        ));
    }

    // Components (East Coast Signature -+++)
    // g_tt = -(1 - 2Mr/Σ)
    let g_tt = -(1.0 - 2.0 * mass * r / sigma);

    // g_rr = Σ/Δ
    let g_rr = sigma / delta;

    // g_thth = Σ
    let g_theta = sigma;

    // g_phph = (r² + a² + 2Ma²r sin²θ/Σ) sin²θ
    let g_phi = (r2 + a2 + 2.0 * mass * a2 * r * sin2_theta / sigma) * sin2_theta;

    // g_tph = -2Mra sin²θ / Σ
    let g_tphi = -2.0 * mass * r * a * sin2_theta / sigma;

    let mut data = vec![0.0; 16];
    // Diagonal
    data[0] = g_tt;
    data[5] = g_rr;
    data[10] = g_theta;
    data[15] = g_phi;

    // Off-Diagonal (symmetric)
    // t=0, phi=3. Index 3 and 12.
    data[3] = g_tphi;
    data[12] = g_tphi;

    Ok(CausalTensor::from_vec(data, &[4, 4]))
}

/// Computes the FLRW metric components (cosmology).
///
/// # Mathematical Definition
/// ```text
/// ds² = -dt² + a(t)² [ dr²/(1-kr²) + r²dθ² + r²sin²θ dφ² ]
/// ```
///
/// # Arguments
/// * `scale_factor` - a(t)
/// * `curvature_k` - k (-1, 0, +1)
/// * `r`, `theta` - Coordinates
pub fn flrw_metric_at(
    scale_factor: f64,
    curvature_k: f64,
    r: f64,
    theta: f64,
) -> Result<CausalTensor<f64>, PhysicsError> {
    if scale_factor <= 0.0 {
        return Err(PhysicsError::DimensionMismatch(
            "Scale factor must be positive".into(),
        ));
    }

    let a2 = scale_factor * scale_factor;
    let kr2 = curvature_k * r * r;

    if (1.0 - kr2).abs() < 1e-10 {
        return Err(PhysicsError::NumericalInstability(
            "Singularity at 1-kr^2=0".into(),
        ));
    }

    // East Coast Signature (-+++)
    let g_tt = -1.0;
    let g_rr = a2 / (1.0 - kr2);
    let g_theta = a2 * r * r;
    let g_phi = a2 * r * r * theta.sin().powi(2);

    let data = vec![
        g_tt, 0.0, 0.0, 0.0, 0.0, g_rr, 0.0, 0.0, 0.0, 0.0, g_theta, 0.0, 0.0, 0.0, 0.0, g_phi,
    ];

    Ok(CausalTensor::from_vec(data, &[4, 4]))
}

/// Creates Schwarzschild metric components at radius r.
///
/// # Mathematical Definition
/// ```text
/// ds² = -(1 - r_s/r)dt² + (1 - r_s/r)⁻¹dr² + r²(dθ² + sin²θ dφ²)
/// ```
///
/// # Arguments
/// * `mass` - Black hole mass (geometric units, M = GM/c²)
/// * `r` - Radial coordinate
pub fn schwarzschild_metric_at(mass: f64, r: f64) -> Result<CausalTensor<f64>, PhysicsError> {
    if r <= 0.0 {
        return Err(PhysicsError::DimensionMismatch(
            "Radius must be positive".into(),
        ));
    }

    let r_s = 2.0 * mass; // Schwarzschild radius in geometric units
    if r <= r_s {
        return Err(PhysicsError::NumericalInstability(format!(
            "Radius {} is at or inside horizon r_s = {}",
            r, r_s
        )));
    }

    let f = 1.0 - r_s / r;
    let theta = PI / 2.0; // Equatorial plane

    // Diagonal metric: diag(g_tt, g_rr, g_θθ, g_φφ)
    let g_tt = -f;
    let g_rr = 1.0 / f;
    let g_theta = r * r;
    let g_phi = r * r * theta.sin().powi(2);

    let data = vec![
        g_tt, 0.0, 0.0, 0.0, 0.0, g_rr, 0.0, 0.0, 0.0, 0.0, g_theta, 0.0, 0.0, 0.0, 0.0, g_phi,
    ];

    Ok(CausalTensor::from_vec(data, &[4, 4]))
}

/// Computes Christoffel symbols for Schwarzschild spacetime at radius r.
///
/// # Mathematical Definition
/// ```text
/// Γ^r_tt = f f' / 2,  Γ^t_tr = f' / (2f),  etc.
/// where f = 1 - r_s/r and f' = df/dr = r_s/r²
/// ```
#[allow(clippy::identity_op, clippy::erasing_op)]
pub fn schwarzschild_christoffel_at(mass: f64, r: f64) -> Result<CausalTensor<f64>, PhysicsError> {
    if r <= 0.0 {
        return Err(PhysicsError::DimensionMismatch(
            "Radius must be positive".into(),
        ));
    }

    let r_s = 2.0 * mass;
    if r <= r_s {
        return Err(PhysicsError::NumericalInstability(format!(
            "Radius {} is at or inside horizon r_s = {}",
            r, r_s
        )));
    }

    let f = 1.0 - r_s / r;
    let f_prime = r_s / (r * r);
    let theta = PI / 2.0;

    // Christoffel symbols Γ^ρ_μν
    // Shape: [4, 4, 4] = 64 elements
    let dim = 4;
    let mut gamma = vec![0.0; dim * dim * dim];

    // Non-zero components (Schwarzschild, equatorial)
    // Γ^t_tr = Γ^t_rt = f'/(2f)
    let g_t_tr = f_prime / (2.0 * f);
    gamma[0 * 16 + 1 * 4 + 0] = g_t_tr; // Γ^t_rt
    gamma[0 * 16 + 0 * 4 + 1] = g_t_tr; // Γ^t_tr

    // Γ^r_tt = f * f' / 2
    gamma[1 * 16 + 0 * 4 + 0] = f * f_prime / 2.0;

    // Γ^r_rr = -f' / (2f)
    gamma[1 * 16 + 1 * 4 + 1] = -f_prime / (2.0 * f);

    // Γ^r_θθ = -(r - r_s)
    gamma[1 * 16 + 2 * 4 + 2] = -(r - r_s);

    // Γ^r_φφ = -(r - r_s) sin²θ
    gamma[1 * 16 + 3 * 4 + 3] = -(r - r_s) * theta.sin().powi(2);

    // Γ^θ_rθ = Γ^θ_θr = 1/r
    gamma[2 * 16 + 1 * 4 + 2] = 1.0 / r;
    gamma[2 * 16 + 2 * 4 + 1] = 1.0 / r;

    // Γ^θ_φφ = -sinθ cosθ
    gamma[2 * 16 + 3 * 4 + 3] = -theta.sin() * theta.cos();

    // Γ^φ_rφ = Γ^φ_φr = 1/r
    gamma[3 * 16 + 1 * 4 + 3] = 1.0 / r;
    gamma[3 * 16 + 3 * 4 + 1] = 1.0 / r;

    // Γ^φ_θφ = Γ^φ_φθ = cotθ
    gamma[3 * 16 + 2 * 4 + 3] = theta.cos() / theta.sin();
    gamma[3 * 16 + 3 * 4 + 2] = theta.cos() / theta.sin();

    Ok(CausalTensor::from_vec(gamma, &[dim, dim, dim]))
}

/// Computes Kretschmann scalar for Schwarzschild spacetime.
///
/// # Mathematical Definition
/// ```text
/// K = 48 M² / r⁶
/// ```
/// This is an exact analytical result for the Schwarzschild solution.
pub fn schwarzschild_kretschmann(mass: f64, r: f64) -> f64 {
    48.0 * mass * mass / r.powi(6)
}
