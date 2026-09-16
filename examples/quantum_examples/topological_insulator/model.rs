/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Model layer for the Chern insulator: the Qi-Wu-Zhang d-vector as a differentiable field, the
//! Berry curvature built from its exact derivatives, and the two ways of integrating it.
//!
//! Every constant is declared at the working type through `const_scalar_from_int!`, so the
//! compiler resolves them against the alias in `main`.

use crate::FloatType;
use deep_causality_algebra::{Real, RealField};
use deep_causality_calculus::{DifferentiableField, DifferentiateFieldExt, Scalar, quadrature};
use deep_causality_num::{const_scalar_from_int, lift_usize};
use deep_causality_num_complex::Complex;

// =============================================================================
// The small numbers the model is written with
// =============================================================================

pub const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);
pub const ONE: FloatType = const_scalar_from_int!(FloatType, 1);
pub const THREE: FloatType = const_scalar_from_int!(FloatType, 3);

// =============================================================================
// Resolution
// =============================================================================

/// Composite-Simpson panels per axis. Simpson's rule needs an even count.
pub const N_QUADRATURE: usize = 100;

/// Brillouin-zone grid for the lattice cross-check.
pub const N_WILSON: usize = 100;

/// Below this the d-vector is treated as having no direction, which happens only where the gap
/// closes and the Berry curvature is undefined anyway.
pub fn degeneracy_floor<S: Scalar>() -> S {
    let ten_billion = lift_usize::<S>(10_000_000_000);

    S::one() / ten_billion
}

// =============================================================================
// The field
// =============================================================================

/// Where each argument of [`DComponent`] sits.
///
/// The mass is an argument rather than a field of the struct, which is the point. A struct that
/// stored `u` at one concrete type would have to widen it inside `run`, and the model would then
/// be evaluated at whatever precision `u` was written down in no matter what `S` the caller asked
/// for. Passing it in leaves every number in the model at the caller's precision.
pub const KX: usize = 0;
pub const KY: usize = 1;
pub const MASS: usize = 2;

/// One component of the Qi-Wu-Zhang d-vector,
///
/// ```text
/// d(k) = (sin kx,  sin ky,  u + cos kx + cos ky)
/// ```
///
/// written as a differentiable field so the tangent functor supplies `∂d/∂k` exactly, with no
/// finite differences and no step size to choose.
pub struct DComponent {
    pub component: usize,
}

impl DifferentiableField<3> for DComponent {
    fn run<S: Scalar>(&self, at: &[S; 3]) -> S {
        match self.component {
            0 => Real::sin(at[KX]),
            1 => Real::sin(at[KY]),
            _ => at[MASS] + Real::cos(at[KX]) + Real::cos(at[KY]),
        }
    }
}

/// The three components, as fields.
pub fn d_components() -> [DComponent; 3] {
    [
        DComponent { component: 0 },
        DComponent { component: 1 },
        DComponent { component: 2 },
    ]
}

/// The d-vector itself, evaluated at the caller's precision.
pub fn d_vector<S: Scalar>(u: S, kx: S, ky: S) -> [S; 3] {
    let at = [kx, ky, u];
    let components = d_components();

    [
        components[0].run(&at),
        components[1].run(&at),
        components[2].run(&at),
    ]
}

// =============================================================================
// Berry curvature
// =============================================================================

/// The Berry curvature of the lower band of `H = d·σ`, in closed form:
///
/// ```text
/// Ω = −(1 / 2|d|³) · d · (∂_kx d × ∂_ky d)
/// ```
///
/// Both first derivatives come from the tangent functor, so they are exact. The mass rides in the
/// third slot of the point and is differentiated with respect to as well; only the two momentum
/// columns of the gradient are read, so the extra slot costs a derivative nobody looks at and buys
/// a model with no concrete type written into it.
pub fn berry_curvature<S: Scalar>(u: S, kx: S, ky: S) -> S {
    let at = [kx, ky, u];
    let components = d_components();

    let d = [
        components[0].run(&at),
        components[1].run(&at),
        components[2].run(&at),
    ];

    let g0 = components[0].gradient(&at);
    let g1 = components[1].gradient(&at);
    let g2 = components[2].gradient(&at);

    let d_kx = [g0[KX], g1[KX], g2[KX]];
    let d_ky = [g0[KY], g1[KY], g2[KY]];

    let cross = [
        d_kx[1] * d_ky[2] - d_kx[2] * d_ky[1],
        d_kx[2] * d_ky[0] - d_kx[0] * d_ky[2],
        d_kx[0] * d_ky[1] - d_kx[1] * d_ky[0],
    ];

    let triple = d[0] * cross[0] + d[1] * cross[1] + d[2] * cross[2];
    let norm_squared = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
    let norm_cubed = norm_squared * Real::sqrt(norm_squared);

    let two = S::one() + S::one();

    -(triple / (two * norm_cubed))
}

/// The Chern number by nested quadrature,
///
/// ```text
/// C = (1 / 2π) ∫∫ Ω dkx dky
/// ```
///
/// over the Brillouin zone. The inner `quadrature` integrates along `ky` at a fixed `kx` and the
/// outer one integrates that, so the two-dimensional integral is the one-dimensional operator
/// applied to itself rather than a second routine written for the purpose.
pub fn chern_quadrature<S: Scalar>(u: S, panels: usize) -> S {
    let pi = S::pi();
    let two = S::one() + S::one();

    let integral = quadrature(
        |kx: S| quadrature(|ky: S| berry_curvature(u, kx, ky), -pi, pi, panels),
        -pi,
        pi,
        panels,
    );

    integral / (two * pi)
}

// =============================================================================
// The lattice cross-check
// =============================================================================

/// The normalised spinor of the lower band at one momentum,
///
/// ```text
/// |−⟩ = ( sin(θ/2),  −cos(θ/2) e^{iφ} )
/// ```
///
/// where `θ` and `φ` are the polar angles of the d-vector.
///
/// This half of the example asks for `RealField` on top of `Scalar`, because it builds `Complex`
/// numbers and a complex number needs a field underneath it. `Scalar` alone admits the dual numbers
/// the tangent functor runs on, and those are not a field. The quadrature route keeps the weaker
/// bound, which is exactly what lets it be differentiated through.
pub fn lower_band_spinor<S: Scalar + RealField>(u: S, kx: S, ky: S) -> (Complex<S>, Complex<S>) {
    let d = d_vector(u, kx, ky);
    let magnitude = Real::sqrt(d[0] * d[0] + d[1] * d[1] + d[2] * d[2]);

    if magnitude < degeneracy_floor::<S>() {
        return (
            Complex::new(S::one(), S::zero()),
            Complex::new(S::zero(), S::zero()),
        );
    }

    let two = S::one() + S::one();
    let half_theta = Real::acos(d[2] / magnitude) / two;
    let phi = Real::atan2(d[1], d[0]);

    let up = Complex::new(Real::sin(half_theta), S::zero());
    let down = Complex::new(
        -Real::cos(half_theta) * Real::cos(phi),
        -Real::cos(half_theta) * Real::sin(phi),
    );

    (up, down)
}

/// The overlap `⟨a|b⟩` of two spinors.
pub fn overlap<S: Scalar + RealField>(
    a: (Complex<S>, Complex<S>),
    b: (Complex<S>, Complex<S>),
) -> Complex<S> {
    conjugate(a.0) * b.0 + conjugate(a.1) * b.1
}

fn conjugate<S: Scalar + RealField>(z: Complex<S>) -> Complex<S> {
    Complex::new(z.re, -z.im)
}

/// The Chern number by the Fukui–Hatsugai–Suzuki lattice method: the sum of Berry fluxes
/// `Im ln W` over a Brillouin-zone grid, where `W` is the Wilson loop around one plaquette.
///
/// It is the independent check on the quadrature above. The two routes share the d-vector and
/// nothing else: this one never differentiates, and the other never forms a spinor. Agreement
/// between them is therefore worth something, which is why the run prints both.
pub fn chern_wilson<S: Scalar + RealField>(u: S, grid: usize) -> S {
    let pi = S::pi();
    let two = S::one() + S::one();
    let steps = lift_usize::<S>(grid);
    let dk = two * pi / steps;

    let mut total_flux = S::zero();

    for i in 0..grid {
        for j in 0..grid {
            let kx = -pi + lift_usize::<S>(i) * dk;
            let ky = -pi + lift_usize::<S>(j) * dk;

            let corner_00 = lower_band_spinor(u, kx, ky);
            let corner_10 = lower_band_spinor(u, kx + dk, ky);
            let corner_11 = lower_band_spinor(u, kx + dk, ky + dk);
            let corner_01 = lower_band_spinor(u, kx, ky + dk);

            let wilson = overlap(corner_00, corner_10)
                * overlap(corner_10, corner_11)
                * overlap(corner_11, corner_01)
                * overlap(corner_01, corner_00);

            total_flux += Real::atan2(wilson.im, wilson.re);
        }
    }

    total_flux / (two * pi)
}

// =============================================================================
// Reading the answer
// =============================================================================

/// Finiteness at the working precision.
pub fn finite<S: Scalar>(x: S) -> bool {
    x.is_finite()
}

/// The nearest integer within the small range a Chern number of this model can take.
///
/// Written as a search over the candidates rather than a call to `round`, so it asks nothing of the
/// scalar beyond subtraction and comparison and stays available at every precision.
pub fn nearest_chern_number<S: Scalar>(x: S) -> i32 {
    let two = S::one() + S::one();
    let half = S::one() / two;

    for candidate in -3i32..=3 {
        if Real::abs(x - integer_scalar::<S>(candidate)) < half {
            return candidate;
        }
    }

    0
}

/// A small signed integer at the working precision.
pub fn integer_scalar<S: Scalar>(n: i32) -> S {
    let magnitude = lift_usize::<S>(n.unsigned_abs() as usize);

    if n < 0 { -magnitude } else { magnitude }
}
