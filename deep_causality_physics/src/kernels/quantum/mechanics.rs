/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::PhysicsError;
use core::fmt::Debug;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;
use deep_causality_par::MaybeParallel;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::SimplicialManifold;

/// Calculates the Klein-Gordon operator action: $(\Delta + m^2)\psi$.
pub fn klein_gordon_kernel<R>(
    psi_manifold: &SimplicialManifold<R, R>,
    mass: R,
) -> Result<CausalTensor<R>, PhysicsError>
where
    R: RealField + MaybeParallel + FromPrimitive + Default + PartialEq + Debug,
{
    let laplacian = psi_manifold.laplacian(0);

    if !mass.is_finite() {
        return Err(PhysicsError::NumericalInstability(
            "Mass is not finite in Klein-Gordon".into(),
        ));
    }
    if laplacian.as_slice().iter().any(|v: &R| !v.is_finite()) {
        return Err(PhysicsError::NumericalInstability(
            "Laplacian contains non-finite entries".into(),
        ));
    }

    let m2 = mass * mass;
    if !m2.is_finite() {
        return Err(PhysicsError::NumericalInstability(
            "m^2 overflowed in Klein-Gordon".into(),
        ));
    }
    let psi_data = psi_manifold.data();

    let vertex_count = laplacian.len();
    if psi_data.len() < vertex_count {
        return Err(PhysicsError::DimensionMismatch(
            "psi_data is smaller than laplacian data".to_string(),
        ));
    }
    let psi_vertex_data = &psi_data.as_slice()[..vertex_count];
    if psi_vertex_data.iter().any(|v: &R| !v.is_finite()) {
        return Err(PhysicsError::NumericalInstability(
            "psi data contains non-finite entries".into(),
        ));
    }
    let psi_vertex_tensor =
        CausalTensor::new(psi_vertex_data.to_vec(), laplacian.shape().to_vec())?;
    let m2_psi = psi_vertex_tensor * m2;
    if m2_psi.as_slice().iter().any(|v| !v.is_finite()) {
        return Err(PhysicsError::NumericalInstability(
            "m^2 * psi produced non-finite entries".into(),
        ));
    }

    let result = laplacian + m2_psi;
    if result.as_slice().iter().any(|v: &R| !v.is_finite()) {
        return Err(PhysicsError::NumericalInstability(
            "Klein-Gordon result contains non-finite entries".into(),
        ));
    }

    Ok(result)
}
