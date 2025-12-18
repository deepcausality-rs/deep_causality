/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::PhysicsError;
use crate::dynamics::quantities::Length;
use crate::photonics::quantities::{AbcdMatrix, ComplexBeamParameter, Wavelength};
use deep_causality_num::{Complex, DivisionAlgebra};

use std::f64::consts::PI;

/// Propagates a Gaussian beam's complex $q$-parameter through an ABCD optical system.
///
/// $$ q_{out} = \frac{A q_{in} + B}{C q_{in} + D} $$
///
/// # Arguments
/// *   `q_in` - Input complex beam parameter $q_{in}$.
/// *   `matrix` - ABCD ray transfer matrix.
///
/// # Returns
/// *   `Result<ComplexBeamParameter, PhysicsError>` - Output $q_{out}$.
pub fn gaussian_q_propagation_kernel(
    q_in: ComplexBeamParameter,
    matrix: &AbcdMatrix,
) -> Result<ComplexBeamParameter, PhysicsError> {
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
    let num = q * a + Complex::new(b, 0.0);
    let den = q * c + Complex::new(dd, 0.0);

    if den.norm_sqr() == 0.0 {
        return Err(PhysicsError::Singularity(
            "Gaussian beam propagation singularity (denominator zero)".into(),
        ));
    }

    let q_out = num / den;

    // Invariant check: Im(q) must be related to spot size, should not be negative for physical beams
    // if propagating in free space. However, ABCD matrices for negative index materials could flip it?
    // Standard optics: Im(1/q) = -lambda / (pi w^2).
    // 1/q = (z - i z_R) / (z^2 + z_R^2). Im(1/q) < 0.
    // q = z + i z_R. Im(q) = z_R = pi w0^2 / lambda > 0.
    // We check Im(q_out) > 0.
    // Some matrices (like complex conjugation/phase conjugation) might flip it, but standard ABCD keeps z_R > 0.

    if q_out.im <= 0.0 {
        // Technically Im(q) can be negative if we model converging wavefronts? No, wavefront is Re(1/q).
        // z_R is beam waist parameter, must be positive.
        // Let's enforce physical realizability for now.
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
/// *   `Result<Length, PhysicsError>` - Beam radius $w(z)$.
pub fn beam_spot_size_kernel(
    q: ComplexBeamParameter,
    wavelength: Wavelength,
) -> Result<Length, PhysicsError> {
    let q_val = q.value();
    let lambda = wavelength.value();

    if q_val.norm_sqr() == 0.0 {
        return Err(PhysicsError::Singularity("q parameter is zero".into()));
    }

    let inv_q = Complex::new(1.0, 0.0) / q_val;
    let im_inv_q = inv_q.im;

    // Im(1/q) = - lambda / (pi * w^2)
    // w^2 = - lambda / (pi * Im(1/q))
    // w = sqrt( ... )

    // Check sign. Im(1/q) should be negative for physical beam (since q=z+iz_R, z_R>0 -> 1/q ~ -i).
    // -lambda is negative.
    // If Im(1/q) is positive, we get sqrt(negative).

    if im_inv_q >= 0.0 {
        // q has negative or zero imaginary part?
        // q = z + i z_R. inv_q = (z - i z_R)/... Im is -z_R.
        // If Im(inv_q) >= 0, then z_R <= 0.
        // This should have been caught by ComplexBeamParameter invariant, but double check.
        // If z_R very small, might be 0.
        return Err(PhysicsError::PhysicalInvariantBroken(format!(
            "Invalid q parameter for spot size extraction: Im(1/q) = {}",
            im_inv_q
        )));
    }

    let w_sq = -lambda / (PI * im_inv_q);
    let w = w_sq.sqrt();

    Length::new(w)
}
