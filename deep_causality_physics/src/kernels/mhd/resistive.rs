/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{AlfvenSpeed, Diffusivity};
use crate::{PhysicsError, Speed};
use core::fmt::Debug;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;
use deep_causality_par::MaybeParallel;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::SimplicialManifold;

/// Calculates the diffusion term of the induction equation.
/// $$ \frac{\partial \mathbf{B}}{\partial t}_{diff} = \eta \nabla^2 \mathbf{B} $$
pub fn resistive_diffusion_kernel<R>(
    b_manifold: &SimplicialManifold<R, R>,
    diffusivity: Diffusivity<R>,
) -> Result<CausalTensor<R>, PhysicsError>
where
    R: RealField + MaybeParallel + FromPrimitive + Default + PartialEq + Debug,
{
    let eta = diffusivity.value();

    if eta < R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Diffusivity cannot be negative".into(),
        ));
    }

    // Compute Hodge Laplacian Delta B = (d delta + delta d) B
    let laplacian = b_manifold.laplacian(2);

    // Rate = - eta * Laplacian
    let rate = laplacian * (-eta);

    Ok(rate)
}

/// Estimates reconnection rate (Sweet-Parker model simplified).
/// $$ v_{in} = \frac{v_A}{\sqrt{S}} $$
pub fn magnetic_reconnection_rate_kernel<R>(
    alfven_speed: AlfvenSpeed<R>,
    lundquist: R,
) -> Result<Speed<R>, PhysicsError>
where
    R: RealField + MaybeParallel,
{
    let va = alfven_speed.value();

    if lundquist <= R::zero() {
        return Err(PhysicsError::Singularity(
            "Lundquist number must be positive for reconnection".into(),
        ));
    }

    let vin = va / lundquist.sqrt();
    Speed::new(vin)
}
