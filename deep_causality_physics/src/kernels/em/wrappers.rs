/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{MagneticFlux, PhysicalField};
use crate::{fields, forces};
use core::fmt::Debug;
use deep_causality_core::{CausalityError, PropagatingEffect};
use deep_causality_multivector::CausalMultiVector;
use deep_causality_num::{FromPrimitive, RealField};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::MaybeParallel;
use deep_causality_topology::SimplicialManifold;

/// Causal wrapper for [`forces::lorentz_force_kernel`].
pub fn lorentz_force<R>(
    j: &CausalMultiVector<R>,
    b: &CausalMultiVector<R>,
) -> PropagatingEffect<PhysicalField<R>>
where
    R: RealField + MaybeParallel + Debug,
{
    match forces::lorentz_force_kernel(j, b) {
        Ok(f) => PropagatingEffect::pure(PhysicalField(f)),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`fields::maxwell_gradient_kernel`].
pub fn maxwell_gradient<R>(
    potential_manifold: &SimplicialManifold<R, R>,
) -> PropagatingEffect<CausalTensor<R>>
where
    R: RealField + MaybeParallel + Default + PartialEq + Debug,
{
    match fields::maxwell_gradient_kernel(potential_manifold) {
        Ok(f) => PropagatingEffect::pure(f),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`fields::lorenz_gauge_kernel`].
pub fn lorenz_gauge<R>(
    potential_manifold: &SimplicialManifold<R, R>,
) -> PropagatingEffect<CausalTensor<R>>
where
    R: RealField + MaybeParallel + FromPrimitive + Default + PartialEq + Debug,
{
    match fields::lorenz_gauge_kernel(potential_manifold) {
        Ok(val) => PropagatingEffect::pure(val),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`fields::poynting_vector_kernel`].
pub fn poynting_vector<R>(
    e: &CausalMultiVector<R>,
    b: &CausalMultiVector<R>,
) -> PropagatingEffect<PhysicalField<R>>
where
    R: RealField + MaybeParallel + Debug,
{
    match fields::poynting_vector_kernel(e, b) {
        Ok(val) => PropagatingEffect::pure(PhysicalField(val)),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`fields::magnetic_helicity_density_kernel`].
pub fn magnetic_helicity_density<R>(
    potential: &CausalMultiVector<R>,
    field: &CausalMultiVector<R>,
) -> PropagatingEffect<MagneticFlux<R>>
where
    R: RealField + MaybeParallel + Debug,
{
    match fields::magnetic_helicity_density_kernel(potential, field) {
        Ok(val) => match MagneticFlux::<R>::new(val) {
            Ok(h) => PropagatingEffect::pure(h),
            Err(e) => PropagatingEffect::from_error(e),
        },
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`fields::proca_equation_kernel`].
pub fn proca_equation<R>(
    field_manifold: &SimplicialManifold<R, R>,
    potential_manifold: &SimplicialManifold<R, R>,
    mass: R,
) -> PropagatingEffect<CausalTensor<R>>
where
    R: RealField + MaybeParallel + FromPrimitive + Default + PartialEq + Debug,
{
    match fields::proca_equation_kernel(field_manifold, potential_manifold, mass) {
        Ok(j) => PropagatingEffect::pure(j),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}
