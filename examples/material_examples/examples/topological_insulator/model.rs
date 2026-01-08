/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::{Complex64, DivisionAlgebra};

// ----------------------------------------------------------------
// Physics Model: Qi-Wu-Zhang (QWZ) Chern Insulator
// ----------------------------------------------------------------
// H(k) = sin(kx)*sx + sin(ky)*sy + (u + cos(kx) + cos(ky))*sz
// This is a 2-band Hamiltonian. The lower band has non-trivial topology depending on 'u'.
// Topological Phase: |u| < 2 (Chern = +/- 1)
// Trivial Phase:     |u| > 2 (Chern = 0)

/// QWZ Model Hamiltonian
pub struct QWZModel {
    u: f64, // Mass parameter
}

impl QWZModel {
    pub fn new(u: f64) -> Self {
        Self { u }
    }

    /// Returns the d-vector for the Hamiltonian H = d . sigma
    pub(crate) fn d_vector(&self, kx: f64, ky: f64) -> (f64, f64, f64) {
        let dx = kx.sin();
        let dy = ky.sin();
        let dz = self.u + kx.cos() + ky.cos();
        (dx, dy, dz)
    }

    /// Returns the normalized spinor for the LOWER band at momentum k
    /// Using the standard formula for 2-level systems
    pub(crate) fn lower_band_spinor(&self, kx: f64, ky: f64) -> (Complex64, Complex64) {
        let (dx, dy, dz) = self.d_vector(kx, ky);
        let d = (dx * dx + dy * dy + dz * dz).sqrt();

        if d < 1e-10 {
            return (Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0));
        }

        // Spherical angles of d-vector
        let cos_theta = dz / d;
        let _sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
        let phi = dy.atan2(dx);

        // Lower band spinor: |-> = [sin(theta/2), -cos(theta/2) * e^(i*phi)]
        // Using half-angle formulas
        let half_theta = (cos_theta).acos() / 2.0;

        let up = Complex64::new(half_theta.sin(), 0.0);
        let down = Complex64::new(-half_theta.cos() * phi.cos(), -half_theta.cos() * phi.sin());

        // Already normalized by construction
        (up, down)
    }
}

/// Compute overlap <u1|u2> between two spinors
pub fn overlap(s1: (Complex64, Complex64), s2: (Complex64, Complex64)) -> Complex64 {
    s1.0.conjugate() * s2.0 + s1.1.conjugate() * s2.1
}
