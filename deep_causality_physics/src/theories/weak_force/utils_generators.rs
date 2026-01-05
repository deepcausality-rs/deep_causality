/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// =============================================================================
// Pauli Matrices (SU(2) Generators)
// =============================================================================

/// Returns the three Pauli matrices σ₁, σ₂, σ₃ as 2×2 complex matrices.
///
/// # Mathematical Definition
/// ```text
/// σ₁ = ⎛0 1⎞   σ₂ = ⎛0 -i⎞   σ₃ = ⎛1  0⎞
///      ⎝1 0⎠        ⎝i  0⎠        ⎝0 -1⎠
/// ```
/// Satisfies [σ_a, σ_b] = 2i ε_{abc} σ_c
pub fn pauli_matrices() -> [[(f64, f64); 4]; 3] {
    let sigma1 = [(0.0, 0.0), (1.0, 0.0), (1.0, 0.0), (0.0, 0.0)];
    let sigma2 = [(0.0, 0.0), (0.0, -1.0), (0.0, 1.0), (0.0, 0.0)];
    let sigma3 = [(1.0, 0.0), (0.0, 0.0), (0.0, 0.0), (-1.0, 0.0)];
    [sigma1, sigma2, sigma3]
}

/// Returns the SU(2) generators T_a = σ_a / 2.
///
/// # Mathematical Definition
/// ```text
/// T_a = σ_a / 2
/// ```
/// Satisfies [T_a, T_b] = i ε_{abc} T_c
pub fn su2_generators() -> [[(f64, f64); 4]; 3] {
    let pauli = pauli_matrices();
    let mut generators = [[(0.0, 0.0); 4]; 3];
    for (i, matrix) in pauli.iter().enumerate() {
        for (j, (re, im)) in matrix.iter().enumerate() {
            generators[i][j] = (re / 2.0, im / 2.0);
        }
    }
    generators
}
