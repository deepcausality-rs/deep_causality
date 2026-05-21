/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::photonics::quantities::{
    AbcdMatrix, ComplexBeamParameter, JonesVector, OpticalPower, RayAngle, RayHeight, StokesVector,
    Wavelength,
};
use crate::{IndexOfRefraction, Length, Ratio};
use core::fmt::Debug;
use deep_causality_core::{CausalityError, PropagatingEffect};
use deep_causality_num::{Complex, FromPrimitive, RealField};
use deep_causality_tensor::CausalTensor;

// Import all kernels from their respective modules
use crate::photonics::{beam, diffraction, polarization, ray};

// ============================================================================
// Ray Optics
// ============================================================================

pub fn ray_transfer<R>(
    m: &AbcdMatrix<R>,
    h: RayHeight<R>,
    a: RayAngle<R>,
) -> PropagatingEffect<(RayHeight<R>, RayAngle<R>)>
where
    R: RealField + Default + Debug,
{
    match ray::ray_transfer_kernel(m, h, a) {
        Ok(res) => PropagatingEffect::pure(res),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn snells_law<R>(
    n1: IndexOfRefraction<R>,
    n2: IndexOfRefraction<R>,
    theta1: RayAngle<R>,
) -> PropagatingEffect<RayAngle<R>>
where
    R: RealField + Debug,
{
    match ray::snells_law_kernel(n1, n2, theta1) {
        Ok(theta2) => PropagatingEffect::pure(theta2),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn lens_maker<R>(n: IndexOfRefraction<R>, r1: R, r2: R) -> PropagatingEffect<OpticalPower<R>>
where
    R: RealField + Debug,
{
    match ray::lens_maker_kernel(n, r1, r2) {
        Ok(p) => PropagatingEffect::pure(p),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

// ============================================================================
// Polarization
// ============================================================================

pub fn jones_rotation<R>(
    jones_matrix: &CausalTensor<Complex<R>>,
    angle: RayAngle<R>,
) -> PropagatingEffect<CausalTensor<Complex<R>>>
where
    R: RealField + Default + Debug,
{
    match polarization::jones_rotation_kernel(jones_matrix, angle) {
        Ok(m) => PropagatingEffect::pure(m),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn stokes_from_jones<R>(jones: &JonesVector<R>) -> PropagatingEffect<StokesVector<R>>
where
    R: RealField + FromPrimitive + Default + Debug,
{
    match polarization::stokes_from_jones_kernel(jones) {
        Ok(s) => PropagatingEffect::pure(s),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn degree_of_polarization<R>(stokes: &StokesVector<R>) -> PropagatingEffect<Ratio>
where
    R: RealField + FromPrimitive + Into<f64>,
{
    match polarization::degree_of_polarization_kernel(stokes) {
        Ok(r) => PropagatingEffect::pure(r),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

// ============================================================================
// Gaussian Beam
// ============================================================================

pub fn gaussian_q_propagation<R>(
    q_in: ComplexBeamParameter<R>,
    matrix: &AbcdMatrix<R>,
) -> PropagatingEffect<ComplexBeamParameter<R>>
where
    R: RealField + Default + Debug,
{
    match beam::gaussian_q_propagation_kernel(q_in, matrix) {
        Ok(q) => PropagatingEffect::pure(q),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn beam_spot_size<R>(
    q: ComplexBeamParameter<R>,
    wavelength: Wavelength<R>,
) -> PropagatingEffect<Length>
where
    R: RealField + FromPrimitive + Into<f64>,
{
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
    theta: RayAngle<f64>,
    wavelength: Wavelength<f64>,
) -> PropagatingEffect<f64> {
    match diffraction::single_slit_irradiance_kernel(i0, slit_width, theta, wavelength) {
        Ok(i) => PropagatingEffect::pure(i),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn grating_equation(
    pitch: Length,
    order: i32,
    incidence: RayAngle<f64>,
    wavelength: Wavelength<f64>,
) -> PropagatingEffect<RayAngle<f64>> {
    match diffraction::grating_equation_kernel(pitch, order, incidence, wavelength) {
        Ok(angle) => PropagatingEffect::pure(angle),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}
