/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::PhysicsError;
use crate::dynamics::quantities::Length;
use crate::photonics::quantities::{AbcdMatrix, ComplexBeamParameter, Wavelength};
use deep_causality_num::{Complex, DivisionAlgebra, FromPrimitive, RealField};

/// Propagates a Gaussian beam's complex $q$-parameter through an ABCD optical system.
///
/// $$ q_{out} = \frac{A q_{in} + B}{C q_{in} + D} $$
///
/// # Arguments
/// *   `q_in` - Input complex beam parameter $q_{in}$.
/// *   `matrix` - ABCD ray transfer matrix.
///
/// # Returns
/// *   `Result<ComplexBeamParameter<R>, PhysicsError>` - Output $q_{out}$.
pub fn gaussian_q_propagation_kernel<R>(
    q_in: ComplexBeamParameter<R>,
    matrix: &AbcdMatrix<R>,
) -> Result<ComplexBeamParameter<R>, PhysicsError>
where
    R: RealField + Default,
{
    let m = matrix.inner();
    if m.shape() != [2, 2] {
        return Err(PhysicsError::DimensionMismatch(
            "ABCD matrix must be 2x2".into(),
        ));
    }

    let d = m.data();
    let a = d[0];
    let b = d[1];
    let c = d[2];
    let dd = d[3]; // D

    let q = q_in.value();

    // q_out = (A*q + B) / (C*q + D)
    let num = q * a + Complex::new(b, R::zero());
    let den = q * c + Complex::new(dd, R::zero());

    if den.norm_sqr() == R::zero() {
        return Err(PhysicsError::Singularity(
            "Gaussian beam propagation singularity (denominator zero)".into(),
        ));
    }

    let q_out = num / den;

    if q_out.im <= R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Resulting Gaussian q-parameter has non-positive imaginary part".into(),
        ));
    }

    ComplexBeamParameter::new(q_out)
}

/// Extracts the beam spot size $w(z)$ from the complex beam parameter $q$.
///
/// Relation:
/// $$ \frac{1}{q} = \frac{1}{R(z)} - i \frac{\lambda}{\pi w(z)^2} $$
///
/// Therefore:
/// $$ w(z) = \sqrt{\frac{-\lambda}{\pi \text{Im}(1/q)}} $$
///
/// # Arguments
/// *   `q` - Complex beam parameter.
/// *   `wavelength` - Wavelength $\lambda$.
///
/// # Returns
/// *   `Result<Length<f64>, PhysicsError>` - Beam radius $w(z)$. Length output is pinned to `f64`
///     until the Length wrapper is retyped (see mech-hub slice).
pub fn beam_spot_size_kernel<R>(
    q: ComplexBeamParameter<R>,
    wavelength: Wavelength<R>,
) -> Result<Length<f64>, PhysicsError>
where
    R: RealField + FromPrimitive + Into<f64>,
{
    let q_val = q.value();
    let lambda = wavelength.value();

    if q_val.norm_sqr() == R::zero() {
        return Err(PhysicsError::Singularity("q parameter is zero".into()));
    }

    let inv_q = Complex::new(R::one(), R::zero()) / q_val;
    let im_inv_q = inv_q.im;

    if im_inv_q >= R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Invalid q parameter for spot size extraction: Im(1/q) >= 0".into(),
        ));
    }

    let pi = R::pi();
    let w_sq = -lambda / (pi * im_inv_q);
    let w = w_sq.sqrt();

    Length::new(w.into())
}
