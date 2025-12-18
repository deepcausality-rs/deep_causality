/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::mhd::quantities::{AlfvenSpeed, Diffusivity};
use crate::{PhysicsError, Speed};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::Manifold;

/// Calculates the diffusion term of the induction equation.
/// $$ \frac{\partial \mathbf{B}}{\partial t}_{diff} = \eta \nabla^2 \mathbf{B} $$
/// On a manifold, this is $-\eta \Delta B$ where $\Delta = d\delta + \delta d$ is the Hodge Laplacian.
/// Note that $\nabla^2$ in vector calc usually corresponds to $-\Delta$ (negative Laplacian).
/// The standard diffusion eq is $\partial_t u = + D \nabla^2 u$.
/// The Hodge Laplacian $\Delta$ is positive definite. So $\nabla^2 \approx -\Delta$.
/// Thus $\partial_t B = - \eta \Delta B$.
///
/// # Arguments
/// *   `b_manifold` - Manifold containing the magnetic flux 2-form $B$.
/// *   `diffusivity` - Magnetic diffusivity $\eta$.
///
/// # Returns
/// *   `Result<CausalTensor<f64>, PhysicsError>` - Rate of change tensor (2-form).
pub fn resistive_diffusion_kernel(
    b_manifold: &Manifold<f64>,
    diffusivity: Diffusivity,
) -> Result<CausalTensor<f64>, PhysicsError> {
    let eta = diffusivity.value();

    if eta < 0.0 {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Diffusivity cannot be negative".into(),
        ));
    }

    // Compute Hodge Laplacian Delta B = (d delta + delta d) B
    let laplacian = b_manifold.laplacian(2);

    // Rate = - eta * Laplacian
    // CausalTensor supports scalar multiplication
    let rate = laplacian * (-eta);

    Ok(rate)
}

/// Estimates reconnection rate (Sweet-Parker model simplified).
/// $$ v_{in} = \frac{v_A}{\sqrt{S}} $$
/// where $S$ is the Lundquist number.
///
/// # Arguments
/// *   `alfven_speed` - Alfven speed $v_A$.
/// *   `lundquist` - Lundquist number $S$.
///
/// # Returns
/// *   `Result<Speed, PhysicsError>` - Inflow velocity $v_{in}$.
pub fn magnetic_reconnection_rate_kernel(
    alfven_speed: AlfvenSpeed,
    lundquist: f64,
) -> Result<Speed, PhysicsError> {
    let va = alfven_speed.value();

    if lundquist <= 0.0 {
        return Err(PhysicsError::Singularity(
            "Lundquist number must be positive for reconnection".into(),
        ));
    }

    let vin = va / lundquist.sqrt();
    Speed::new(vin)
}
