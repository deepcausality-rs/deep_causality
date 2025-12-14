/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::photonics::quantities::{
    AbcdMatrix, ComplexBeamParameter, JonesVector, OpticalPower, RayAngle, RayHeight, StokesVector,
    Wavelength,
};
use crate::{IndexOfRefraction, Length, Ratio};
use deep_causality_core::{CausalityError, PropagatingEffect};
use deep_causality_num::Complex;
use deep_causality_tensor::CausalTensor;

// Import all kernels from their respective modules
use crate::photonics::{beam, diffraction, polarization, ray};

// ============================================================================
// Ray Optics
// ============================================================================

pub fn ray_transfer(
    m: &AbcdMatrix,
    h: RayHeight,
    a: RayAngle,
) -> PropagatingEffect<(RayHeight, RayAngle)> {
    match ray::ray_transfer_kernel(m, h, a) {
        Ok(res) => PropagatingEffect::pure(res),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn snells_law(
    n1: IndexOfRefraction,
    n2: IndexOfRefraction,
    theta1: RayAngle,
) -> PropagatingEffect<RayAngle> {
    match ray::snells_law_kernel(n1, n2, theta1) {
        Ok(theta2) => PropagatingEffect::pure(theta2),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn lens_maker(n: IndexOfRefraction, r1: f64, r2: f64) -> PropagatingEffect<OpticalPower> {
    match ray::lens_maker_kernel(n, r1, r2) {
        Ok(p) => PropagatingEffect::pure(p),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

// ============================================================================
// Polarization
// ============================================================================

pub fn jones_rotation(
    jones_matrix: &CausalTensor<Complex<f64>>,
    angle: RayAngle,
) -> PropagatingEffect<CausalTensor<Complex<f64>>> {
    match polarization::jones_rotation_kernel(jones_matrix, angle) {
        Ok(m) => PropagatingEffect::pure(m),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn stokes_from_jones(jones: &JonesVector) -> PropagatingEffect<StokesVector> {
    match polarization::stokes_from_jones_kernel(jones) {
        Ok(s) => PropagatingEffect::pure(s),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn degree_of_polarization(stokes: &StokesVector) -> PropagatingEffect<Ratio> {
    match polarization::degree_of_polarization_kernel(stokes) {
        Ok(r) => PropagatingEffect::pure(r),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

// ============================================================================
// Gaussian Beam
// ============================================================================

pub fn gaussian_q_propagation(
    q_in: ComplexBeamParameter,
    matrix: &AbcdMatrix,
) -> PropagatingEffect<ComplexBeamParameter> {
    match beam::gaussian_q_propagation_kernel(q_in, matrix) {
        Ok(q) => PropagatingEffect::pure(q),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn beam_spot_size(
    q: ComplexBeamParameter,
    wavelength: Wavelength,
) -> PropagatingEffect<Length> {
    match beam::beam_spot_size_kernel(q, wavelength) {
        Ok(w) => PropagatingEffect::pure(w),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

// ============================================================================
// Diffraction
// ============================================================================

pub fn single_slit_irradiance(
    i0: f64,
    slit_width: Length,
    theta: RayAngle,
    wavelength: Wavelength,
) -> PropagatingEffect<f64> {
    match diffraction::single_slit_irradiance_kernel(i0, slit_width, theta, wavelength) {
        Ok(i) => PropagatingEffect::pure(i),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn grating_equation(
    pitch: Length,
    order: i32,
    incidence: RayAngle,
    wavelength: Wavelength,
) -> PropagatingEffect<RayAngle> {
    match diffraction::grating_equation_kernel(pitch, order, incidence, wavelength) {
        Ok(angle) => PropagatingEffect::pure(angle),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}
