/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_calculus::{DifferentiableField, DifferentiateFieldExt, Scalar, quadrature};
use deep_causality_num::{Complex64, DivisionAlgebra};
use std::f64::consts::PI;

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

// =============================================================================
// Tangent-functor Berry curvature + nested-quadrature Chern number
// =============================================================================

/// One component of the QWZ d-vector `d(k) = (sin kx, sin ky, u + cos kx + cos ky)`, written as a
/// differentiable field of `(kx, ky)` so the tangent functor supplies `∂d/∂k` with no finite
/// differences.
struct DComponent {
    comp: usize,
    u: f64,
}

impl DifferentiableField<2> for DComponent {
    fn run<S: Scalar>(&self, k: &[S; 2]) -> S {
        let (kx, ky) = (k[0], k[1]);
        match self.comp {
            0 => kx.sin(),
            1 => ky.sin(),
            _ => {
                S::from_f64(self.u).expect("u lifts into the working scalar") + kx.cos() + ky.cos()
            }
        }
    }
}

/// Berry curvature `Ω(kx, ky)` of the lower band of `H = d·σ`, in closed form
/// `Ω = −(1/2|d|³) · d·(∂_kx d × ∂_ky d)`. The first derivatives come from the tangent functor.
pub(crate) fn berry_curvature<S: Scalar>(u: f64, kx: S, ky: S) -> S {
    let comps = [
        DComponent { comp: 0, u },
        DComponent { comp: 1, u },
        DComponent { comp: 2, u },
    ];
    let k = [kx, ky];
    let d = [comps[0].run(&k), comps[1].run(&k), comps[2].run(&k)];
    let g0 = comps[0].gradient(&k);
    let g1 = comps[1].gradient(&k);
    let g2 = comps[2].gradient(&k);
    let dkx = [g0[0], g1[0], g2[0]]; // ∂_kx d
    let dky = [g0[1], g1[1], g2[1]]; // ∂_ky d
    let cross = [
        dkx[1] * dky[2] - dkx[2] * dky[1],
        dkx[2] * dky[0] - dkx[0] * dky[2],
        dkx[0] * dky[1] - dkx[1] * dky[0],
    ];
    let triple = d[0] * cross[0] + d[1] * cross[1] + d[2] * cross[2];
    let norm2 = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
    let norm3 = norm2 * norm2.sqrt();
    let half = S::from_f64(0.5).expect("½ lifts into the working scalar");
    -(half * triple / norm3)
}

/// Chern number `C = (1/2π) ∫∫_BZ Ω dkx dky`, by nested composite-Simpson `quadrature` of the
/// tangent-functor Berry curvature. Precision-generic: instantiate at `f64` or `Float106`.
pub(crate) fn chern_quadrature<S: Scalar>(u: f64, n: usize) -> S {
    let pi = S::from_f64(PI).expect("π lifts into the working scalar");
    let neg_pi = S::from_f64(-PI).expect("−π lifts into the working scalar");
    let two_pi = S::from_f64(2.0 * PI).expect("2π lifts into the working scalar");
    let integral = quadrature(
        |kx: S| quadrature(|ky: S| berry_curvature(u, kx, ky), neg_pi, pi, n),
        neg_pi,
        pi,
        n,
    );
    integral / two_pi
}

/// Chern number by the Fukui–Hatsugai–Suzuki lattice method: the gauge-invariant sum of Berry
/// fluxes `Im ln(Wilson loop)` over a Brillouin-zone grid. This is the prior accumulation the
/// quadrature result is checked against; it runs at `f64` (the spinors are `Complex64`).
pub(crate) fn chern_wilson(model: &QWZModel, n: usize) -> f64 {
    let dk = 2.0 * PI / (n as f64);
    let mut total_flux = 0.0;
    for i in 0..n {
        for j in 0..n {
            let kx = -PI + (i as f64) * dk;
            let ky = -PI + (j as f64) * dk;
            let psi_00 = model.lower_band_spinor(kx, ky);
            let psi_10 = model.lower_band_spinor(kx + dk, ky);
            let psi_11 = model.lower_band_spinor(kx + dk, ky + dk);
            let psi_01 = model.lower_band_spinor(kx, ky + dk);
            let wilson = overlap(psi_00, psi_10)
                * overlap(psi_10, psi_11)
                * overlap(psi_11, psi_01)
                * overlap(psi_01, psi_00);
            total_flux += wilson.im.atan2(wilson.re);
        }
    }
    total_flux / (2.0 * PI)
}

/// Finiteness at the working precision, kept generic via the `Scalar` bound.
pub(crate) fn finite<S: Scalar>(x: S) -> bool {
    x.is_finite()
}

/// Nearest integer in the small Chern range `{−3 … 3}`, without needing `Float::round` on the
/// concrete scalar (so it stays precision-generic).
pub(crate) fn nearest_int<S: Scalar>(x: S) -> i32 {
    let half = S::from_f64(0.5).expect("½ lifts");
    for c in -3..=3 {
        let cc = S::from_f64(c as f64).expect("candidate lifts");
        if (x - cc).abs() < half {
            return c;
        }
    }
    0
}
