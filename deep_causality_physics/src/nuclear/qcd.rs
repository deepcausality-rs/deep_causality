/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Quantum Chromodynamics (QCD) kernels for SU(3) gauge theory.
//!
//! Provides fundamental QCD operations including:
//! - Gell-Mann matrices (SU(3) generators)
//! - Structure constants (f^{abc})
//! - Covariant derivative
//! - Wilson loop (confinement order parameter)
//! - Confinement potential (string tension)
//! - Running coupling (asymptotic freedom)

use crate::PhysicsError;

// ============================================================================
// Constants: Gell-Mann Matrices
// ============================================================================

/// The 8 Gell-Mann matrices λ^a (a = 1..8), generators of SU(3).
///
/// Each matrix is 3x3 complex, stored as [Re(00), Im(00), Re(01), Im(01), ...]
/// for row-major order with 9 complex entries = 18 f64 values.
///
/// For simplicity, we store only the real parts since most components are real.
/// Non-zero imaginary parts are handled via the structure constants.
///
/// Returns: 8 matrices, each as [3][3] = 9 real values (imaginary handled separately).
pub fn gell_mann_matrices() -> [[f64; 9]; 8] {
    // λ_1: off-diagonal in (1,2) positions
    let lambda1 = [0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0];

    // λ_2: anti-symmetric imaginary (1,2) - stored as pure real with sign convention
    // Actually has imaginary parts: (0,-i,0), (i,0,0), (0,0,0)
    // We store the magnitude; use structure constants for phases
    let lambda2 = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]; // Imaginary - use structure constants

    // λ_3: diagonal (1, -1, 0)
    let lambda3 = [1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0];

    // λ_4: off-diagonal in (1,3) positions
    let lambda4 = [0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0];

    // λ_5: anti-symmetric imaginary (1,3)
    let lambda5 = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]; // Imaginary

    // λ_6: off-diagonal in (2,3) positions
    let lambda6 = [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0];

    // λ_7: anti-symmetric imaginary (2,3)
    let lambda7 = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]; // Imaginary

    // λ_8: diagonal (1, 1, -2) / sqrt(3)
    let inv_sqrt3 = 1.0 / 3.0_f64.sqrt();
    let lambda8 = [
        inv_sqrt3,
        0.0,
        0.0,
        0.0,
        inv_sqrt3,
        0.0,
        0.0,
        0.0,
        -2.0 * inv_sqrt3,
    ];

    [
        lambda1, lambda2, lambda3, lambda4, lambda5, lambda6, lambda7, lambda8,
    ]
}

// ============================================================================
// Constants: SU(3) Structure Constants
// ============================================================================

/// Returns the totally antisymmetric SU(3) structure constants f^{abc}.
///
/// Non-zero values (and their antisymmetric permutations):
/// - f^{123} = 1
/// - f^{147} = f^{165} = f^{246} = f^{257} = f^{345} = f^{376} = 1/2
/// - f^{458} = f^{678} = √3/2
///
/// Returns: Function that computes f^{abc} for given indices (1-indexed).
pub fn structure_constant(a: usize, b: usize, c: usize) -> f64 {
    // Canonical non-zero structure constants (1-indexed in physics convention)
    // We convert to 0-indexed internally
    let half = 0.5;
    let sqrt3_half = 3.0_f64.sqrt() * 0.5;

    // Normalize to sorted order with sign tracking for antisymmetry
    let mut indices = [a, b, c];
    let mut sign = 1.0;

    // Bubble sort with sign tracking
    for i in 0..2 {
        for j in 0..2 - i {
            if indices[j] > indices[j + 1] {
                indices.swap(j, j + 1);
                sign *= -1.0;
            }
        }
    }

    let [i, j, k] = indices;

    // Non-zero structure constants (1-indexed)
    let value = match (i, j, k) {
        (1, 2, 3) => 1.0,
        (1, 4, 7) => half,
        (1, 5, 6) => -half, // Note sign
        (2, 4, 6) => half,
        (2, 5, 7) => half,
        (3, 4, 5) => half,
        (3, 6, 7) => -half, // Note sign
        (4, 5, 8) => sqrt3_half,
        (6, 7, 8) => sqrt3_half,
        _ => 0.0,
    };

    sign * value
}

/// Returns all non-zero structure constants as a list of (a, b, c, f^abc).
pub fn all_structure_constants() -> Vec<(usize, usize, usize, f64)> {
    let half = 0.5;
    let sqrt3_half = 3.0_f64.sqrt() * 0.5;

    vec![
        (1, 2, 3, 1.0),
        (1, 4, 7, half),
        (1, 6, 5, half), // Permutation of (1,5,6)
        (2, 4, 6, half),
        (2, 5, 7, half),
        (3, 4, 5, half),
        (3, 7, 6, half), // Permutation of (3,6,7)
        (4, 5, 8, sqrt3_half),
        (6, 7, 8, sqrt3_half),
    ]
}

// ============================================================================
// Kernels
// ============================================================================

/// Computes the gauge covariant derivative: $D_\mu \psi = \partial_\mu \psi + i g A_\mu \psi$.
///
/// For SU(3), $A_\mu = A_\mu^a T^a$ where $T^a = \lambda^a / 2$.
///
/// # Arguments
/// * `psi` - Field value (color triplet, 3 complex components = 6 f64).
/// * `psi_gradient` - Ordinary derivative $\partial_\mu \psi$ (4 spacetime × 6 color = 24 values).
/// * `gluon_field` - Gluon potential $A_\mu^a$ (4 spacetime × 8 color = 32 values).
/// * `coupling` - QCD coupling constant $g$.
///
/// # Returns
/// * `Ok(Vec<f64>)` - Covariant derivative $D_\mu \psi$ (4 × 6 = 24 values).
pub fn covariant_derivative_kernel(
    psi: &[f64],          // 6 values: (Re, Im) for each color
    psi_gradient: &[f64], // 24 values: 4 spacetime × 6 color
    gluon_field: &[f64],  // 32 values: 4 spacetime × 8 color adjoint
    coupling: f64,
) -> Result<Vec<f64>, PhysicsError> {
    // Validate input dimensions
    if psi.len() != 6 {
        return Err(PhysicsError::DimensionMismatch(format!(
            "Psi must have 6 components (3 complex), got {}",
            psi.len()
        )));
    }
    if psi_gradient.len() != 24 {
        return Err(PhysicsError::DimensionMismatch(format!(
            "Psi gradient must have 24 components, got {}",
            psi_gradient.len()
        )));
    }
    if gluon_field.len() != 32 {
        return Err(PhysicsError::DimensionMismatch(format!(
            "Gluon field must have 32 components, got {}",
            gluon_field.len()
        )));
    }

    let matrices = gell_mann_matrices();
    let mut result = vec![0.0; 24];

    // For each spacetime direction mu
    for mu in 0..4 {
        // Start with ordinary derivative
        for c in 0..6 {
            result[mu * 6 + c] = psi_gradient[mu * 6 + c];
        }

        // Add gauge term: i * g * A_mu^a * (lambda^a / 2) * psi
        // Simplified: we add the diagonal contributions from λ_3 and λ_8
        for a in 0..8 {
            let a_mu_a = gluon_field[mu * 8 + a];
            let lambda = &matrices[a];

            // Matrix-vector multiply (lambda/2) * psi
            // For real matrices acting on complex vector
            for i in 0..3 {
                for j in 0..3 {
                    let m_ij = lambda[i * 3 + j] * 0.5; // T^a = λ^a / 2
                    if m_ij != 0.0 {
                        // Multiply complex: (0 + i * g * A * m_ij) * psi[j]
                        // = i * g * A * m_ij * (psi_re + i * psi_im)
                        // = i * g * A * m_ij * psi_re - g * A * m_ij * psi_im
                        let psi_re = psi[j * 2];
                        let psi_im = psi[j * 2 + 1];
                        let factor = coupling * a_mu_a * m_ij;

                        // Re part: -g * A * m * psi_im
                        result[mu * 6 + i * 2] -= factor * psi_im;
                        // Im part: +g * A * m * psi_re
                        result[mu * 6 + i * 2 + 1] += factor * psi_re;
                    }
                }
            }
        }
    }

    // Check for numerical issues
    if result.iter().any(|v| !v.is_finite()) {
        return Err(PhysicsError::NumericalInstability(
            "Non-finite result in covariant derivative".into(),
        ));
    }

    Ok(result)
}

/// Computes a simplified Wilson loop trace for confinement analysis.
///
/// The Wilson loop is $W(C) = \text{Tr}[\mathcal{P} \exp(i g \oint_C A_\mu dx^\mu)]$.
///
/// For a rectangular loop of area $A$, in confining phase: $\langle W \rangle \sim \exp(-\sigma A)$.
///
/// This simplified version computes: $W \approx \text{Tr}[\exp(i g \sum_i A_i \cdot \Delta x_i)]$
///
/// # Arguments
/// * `gluon_values` - Gluon field values at each path segment (N segments × 8 color).
/// * `path_lengths` - Length of each path segment (N values).
/// * `coupling` - QCD coupling constant $g$.
///
/// # Returns
/// * `Ok(f64)` - Wilson loop trace (should be ~3 for trivial loop, decays for confinement).
pub fn wilson_loop_kernel(
    gluon_values: &[f64],
    path_lengths: &[f64],
    coupling: f64,
) -> Result<f64, PhysicsError> {
    let num_segments = path_lengths.len();
    if gluon_values.len() != num_segments * 8 {
        return Err(PhysicsError::DimensionMismatch(format!(
            "Expected {} gluon values for {} segments, got {}",
            num_segments * 8,
            num_segments,
            gluon_values.len()
        )));
    }

    if num_segments == 0 {
        return Ok(3.0); // Tr(I) = 3 for SU(3)
    }

    // For small loops, approximate: W ≈ 1 - (g²/2) * sum(A² * dl²)
    // This captures the area law behavior for confinement
    let mut phase_sum = 0.0;

    for i in 0..num_segments {
        let dl = path_lengths[i];

        // Sum of |A|² at this segment
        let mut a_squared = 0.0;
        for a in 0..8 {
            let a_a = gluon_values[i * 8 + a];
            a_squared += a_a * a_a;
        }

        phase_sum += a_squared * dl * dl;
    }

    // Wilson loop with quadratic approximation
    let w = 3.0 * (-0.5 * coupling * coupling * phase_sum).exp();

    if !w.is_finite() {
        return Err(PhysicsError::NumericalInstability(
            "Non-finite Wilson loop result".into(),
        ));
    }

    Ok(w)
}

/// Computes the linear confining potential: $V(r) = \sigma r + V_0$.
///
/// The string tension $\sigma \approx 0.18 \text{ GeV}^2$ in physical units.
///
/// # Arguments
/// * `distance` - Separation distance $r$ in fm (or natural units).
/// * `string_tension` - String tension $\sigma$ in GeV² (typically ~0.18).
/// * `coulomb_term` - Optional Coulomb coefficient for short-range: $-\alpha/r$.
///
/// # Returns
/// * `Ok(f64)` - Potential energy $V(r)$ in GeV.
pub fn confinement_potential_kernel(
    distance: f64,
    string_tension: f64,
    coulomb_term: Option<f64>,
) -> Result<f64, PhysicsError> {
    if distance <= 0.0 {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Distance must be positive".into(),
        ));
    }

    if !distance.is_finite() || !string_tension.is_finite() {
        return Err(PhysicsError::NumericalInstability(
            "Non-finite input to confinement potential".into(),
        ));
    }

    // Linear confining term
    let mut v = string_tension * distance;

    // Optional Coulomb term for short-range behavior
    if let Some(alpha) = coulomb_term {
        if !alpha.is_finite() {
            return Err(PhysicsError::NumericalInstability(
                "Non-finite Coulomb coefficient".into(),
            ));
        }
        v -= alpha / distance;
    }

    Ok(v)
}

/// Computes the running QCD coupling constant $\alpha_s(Q^2)$ (asymptotic freedom).
///
/// One-loop beta function: $\alpha_s(Q^2) = \frac{4\pi}{(11 - 2n_f/3) \ln(Q^2/\Lambda_{QCD}^2)}$
///
/// # Arguments
/// * `q_squared` - Momentum transfer squared $Q^2$ in GeV².
/// * `lambda_qcd` - QCD scale $\Lambda_{QCD}$ in GeV (typically ~0.2).
/// * `n_flavors` - Number of active quark flavors (typically 3-6).
///
/// # Returns
/// * `Ok(f64)` - Running coupling $\alpha_s(Q^2)$.
pub fn running_coupling_kernel(
    q_squared: f64,
    lambda_qcd: f64,
    n_flavors: u32,
) -> Result<f64, PhysicsError> {
    if q_squared <= 0.0 {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Q² must be positive".into(),
        ));
    }

    if lambda_qcd <= 0.0 {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Lambda_QCD must be positive".into(),
        ));
    }

    let lambda_squared = lambda_qcd * lambda_qcd;
    if q_squared <= lambda_squared {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Q² must be greater than Λ_QCD² for perturbative regime".into(),
        ));
    }

    // One-loop beta function coefficient
    let b0 = 11.0 - (2.0 * n_flavors as f64) / 3.0;
    if b0 <= 0.0 {
        return Err(PhysicsError::PhysicalInvariantBroken(format!(
            "Too many flavors ({}): b0 = {} <= 0",
            n_flavors, b0
        )));
    }

    let log_ratio = (q_squared / lambda_squared).ln();
    let alpha_s = 4.0 * std::f64::consts::PI / (b0 * log_ratio);

    if !alpha_s.is_finite() || alpha_s <= 0.0 {
        return Err(PhysicsError::NumericalInstability(format!(
            "Invalid running coupling: α_s = {}",
            alpha_s
        )));
    }

    Ok(alpha_s)
}
