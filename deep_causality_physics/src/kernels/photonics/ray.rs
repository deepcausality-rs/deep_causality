/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::kernels::photonics::quantities::{AbcdMatrix, OpticalPower, RayAngle, RayHeight};

use crate::{IndexOfRefraction, PhysicsError};
use deep_causality_num::RealField;
use deep_causality_tensor::{CausalTensor, Tensor};

/// Applies an ABCD matrix to a ray vector.
///
/// $$ \begin{pmatrix} y_{out} \\ \theta_{out} \end{pmatrix} = \begin{pmatrix} A & B \\ C & D \end{pmatrix} \begin{pmatrix} y_{in} \\ \theta_{in} \end{pmatrix} $$
///
/// The ray transfer matrix (ABCD matrix) describes the optical system in the paraxial approximation.
/// The vector consists of the ray height $y$ and angle $\theta$.
///
/// # Arguments
/// *   `matrix` - The $2 \times 2$ ray transfer matrix (ABCD).
/// *   `height` - Input ray height $y$.
/// *   `angle` - Input ray angle $\theta$.
///
/// # Returns
/// *   `Result<(RayHeight<R>, RayAngle<R>), PhysicsError>` - The output ray height and angle.
pub fn ray_transfer_kernel<R>(
    matrix: &AbcdMatrix<R>,
    height: RayHeight<R>,
    angle: RayAngle<R>,
) -> Result<(RayHeight<R>, RayAngle<R>), PhysicsError>
where
    R: RealField + Default,
{
    let m = matrix.inner();
    if m.shape() != [2, 2] {
        return Err(PhysicsError::DimensionMismatch(
            "ABCD matrix must be 2x2".into(),
        ));
    }

    let y_in = height.value();
    let theta_in = angle.value();

    // Create input column vector [y, theta]
    let input_vec = CausalTensor::new(vec![y_in, theta_in], vec![2, 1])?;

    // Matrix multiplication: [y_out, theta_out] = M * [y_in, theta_in]
    let output_vec = m.matmul(&input_vec)?;

    let y_out = output_vec.data()[0];
    let theta_out = output_vec.data()[1];

    Ok((RayHeight::new(y_out)?, RayAngle::new(theta_out)?))
}

/// Calculates refracted angle using Snell's Law or returns a Critical Angle error.
///
/// $$ n_1 \sin \theta_1 = n_2 \sin \theta_2 $$
///
/// # Arguments
/// *   `n1` - Refractive index of medium 1.
/// *   `n2` - Refractive index of medium 2.
/// *   `theta1` - Angle of incidence (relative to normal).
///
/// # Returns
/// *   `Result<RayAngle<R>, PhysicsError>` - Angle of refraction.
///
/// # Errors
/// *   `PhysicalInvariantBroken` - If total internal reflection occurs ($\sin \theta_2 > 1$).
pub fn snells_law_kernel<R>(
    n1: IndexOfRefraction<R>,
    n2: IndexOfRefraction<R>,
    theta1: RayAngle<R>,
) -> Result<RayAngle<R>, PhysicsError>
where
    R: RealField,
{
    let n1_val = n1.value();
    let n2_val = n2.value();
    let theta1_val = theta1.value();

    // sin(theta2) = (n1 / n2) * sin(theta1)
    let sin_theta2 = (n1_val / n2_val) * theta1_val.sin();

    if sin_theta2.abs() > R::one() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Total Internal Reflection: sin(theta2) > 1".into(),
        ));
    }

    let theta2 = sin_theta2.asin();
    RayAngle::new(theta2)
}

/// Calculates optical power and focal length using the Lens Maker's Equation.
///
/// $$ P = (n - 1) \left( \frac{1}{R_1} - \frac{1}{R_2} \right) $$
///
/// Uses the sign convention where:
/// *   Light travels from left to right.
/// *   Radius of curvature $R$ is positive if the center of curvature is to the right of the surface.
/// *   Therefore, for a biconvex lens, $R_1 > 0$ (front surface convex) and $R_2 < 0$ (back surface convex).
///
/// # Arguments
/// *   `n` - Refractive index of the lens material (assuming ambient index is 1).
/// *   `r1` - Radius of curvature of the first surface.
/// *   `r2` - Radius of curvature of the second surface.
///
/// # Returns
/// *   `Result<OpticalPower<R>, PhysicsError>` - Optical power in Diopters.
pub fn lens_maker_kernel<R>(
    n: IndexOfRefraction<R>,
    r1: R,
    r2: R,
) -> Result<OpticalPower<R>, PhysicsError>
where
    R: RealField,
{
    // P = (n - 1) * (1/R1 - 1/R2)
    let n_val = n.value();

    if r1 == R::zero() || r2 == R::zero() {
        return Err(PhysicsError::Singularity(
            "Radius of curvature cannot be zero".into(),
        ));
    }

    let power = (n_val - R::one()) * ((R::one() / r1) - (R::one() / r2));
    OpticalPower::new(power)
}
