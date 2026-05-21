/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{JonesVector, PhysicsError, Ratio, RayAngle, StokesVector};
use deep_causality_num::{Complex, DivisionAlgebra, FromPrimitive, RealField};
use deep_causality_tensor::CausalTensor;

/// Rotates a Jones Matrix (operator) by an angle $\phi$.
///
/// $$ M'(\\phi) = R(-\\phi) \cdot M \cdot R(\\phi) $$
///
/// where $R(\\phi)$ is the 2D rotation matrix:
/// $$ R(\\phi) = \begin{pmatrix} \cos\\phi & \sin\\phi \\ -\sin\\phi & \cos\\phi \end{pmatrix} $$
///
/// This transforms an optical element (like a polarizer or waveplate) aligned along
/// the horizontal axis to an arbitrary axis $\\phi$.
///
/// # Arguments
/// *   `jones_matrix` - The $2 \times 2$ complex Jones matrix of the element.
/// *   `angle` - Rotation angle $\\phi$.
///
/// # Returns
/// *   `Result<CausalTensor<Complex<R>>, PhysicsError>` - The rotated Jones matrix.
pub fn jones_rotation_kernel<R>(
    jones_matrix: &CausalTensor<Complex<R>>,
    angle: RayAngle<R>,
) -> Result<CausalTensor<Complex<R>>, PhysicsError>
where
    R: RealField + Default,
{
    if jones_matrix.shape() != [2, 2] {
        return Err(PhysicsError::DimensionMismatch(
            "Jones matrix must be 2x2".into(),
        ));
    }

    let phi = angle.value();
    let c = phi.cos();
    let s = phi.sin();

    // M' = R(-phi) * M * R(phi)
    // jones_matrix M data: [m00, m01, m10, m11]
    let m_data = jones_matrix.data();
    let m00 = m_data[0];
    let m01 = m_data[1];
    let m10 = m_data[2];
    let m11 = m_data[3];

    // R(phi) components as complex scalars
    let rc = Complex::new(c, R::zero());
    let rs = Complex::new(s, R::zero());

    // Intermediate: T = M * R(phi)
    let t00 = m00 * rc - m01 * rs;
    let t01 = m00 * rs + m01 * rc;
    let t10 = m10 * rc - m11 * rs;
    let t11 = m10 * rs + m11 * rc;

    // Final: M' = R(-phi) * T
    let res00 = rc * t00 - rs * t10;
    let res01 = rc * t01 - rs * t11;
    let res10 = rs * t00 + rc * t10;
    let res11 = rs * t01 + rc * t11;

    let rotated_data = vec![res00, res01, res10, res11];
    let rotated = CausalTensor::new(rotated_data, vec![2, 2])?;

    Ok(rotated)
}

/// Converts a pure state Jones vector to a Stokes vector.
///
/// $$ S_0 = |E_x|^2 + |E_y|^2 $$
/// $$ S_1 = |E_x|^2 - |E_y|^2 $$
/// $$ S_2 = 2\text{Re}(E_x E_y^*) $$
/// $$ S_3 = -2\text{Im}(E_x E_y^*) $$
///
/// # Arguments
/// *   `jones` - Input Jones vector $\begin{pmatrix} E_x \\ E_y \end{pmatrix}$.
///
/// # Returns
/// *   `Result<StokesVector<R>, PhysicsError>` - The corresponding Stokes vector.
pub fn stokes_from_jones_kernel<R>(jones: &JonesVector<R>) -> Result<StokesVector<R>, PhysicsError>
where
    R: RealField + FromPrimitive + Default,
{
    let t = jones.inner();
    if t.shape() != [2] {
        return Err(PhysicsError::DimensionMismatch(
            "Jones vector must be length 2".into(),
        ));
    }

    let ex = t.data()[0];
    let ey = t.data()[1];

    let ex_sq = ex.norm_sqr();
    let ey_sq = ey.norm_sqr();

    // conj(ey)
    let ey_conj = Complex::new(ey.re, -ey.im);
    // Ex * Ey*
    let cross = ex * ey_conj;

    let two = R::from_f64(2.0)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(2.0) failed".into()))?;

    let s0 = ex_sq + ey_sq;
    let s1 = ex_sq - ey_sq;
    let s2 = two * cross.re;
    let s3 = -two * cross.im;

    let stokes_data = vec![s0, s1, s2, s3];
    let stokes_tensor = CausalTensor::new(stokes_data, vec![4])?;

    StokesVector::new(stokes_tensor)
}

/// Calculates the Degree of Polarization (DOP) from a Stokes vector.
///
/// $$ DOP = \frac{\sqrt{S_1^2 + S_2^2 + S_3^2}}{S_0} $$
///
/// # Arguments
/// *   `stokes` - Input Stokes vector.
///
/// # Returns
/// *   `Result<Ratio<R>, PhysicsError>` - DOP (0 to 1).
pub fn degree_of_polarization_kernel<R>(stokes: &StokesVector<R>) -> Result<Ratio<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let t = stokes.inner();
    if t.shape() != [4] {
        return Err(PhysicsError::DimensionMismatch(
            "Stokes vector must be length 4".into(),
        ));
    }

    let d = t.data();
    let s0 = d[0];
    let s1 = d[1];
    let s2 = d[2];
    let s3 = d[3];

    if s0 <= R::zero() {
        if s0 == R::zero() && s1 == R::zero() && s2 == R::zero() && s3 == R::zero() {
            return Ratio::<R>::new(R::zero()); // Zero intensity, undefined DOP, return 0
        }
        return Err(PhysicsError::PhysicalInvariantBroken(
            "S0 must be positive".into(),
        ));
    }

    let pol_mag = (s1 * s1 + s2 * s2 + s3 * s3).sqrt();
    let dop = pol_mag / s0;

    let one = R::one();
    let tolerance = R::from_f64(1.000001)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(1.000001) failed".into()))?;
    if dop > tolerance {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "DOP > 1, unphysical Stokes vector".into(),
        ));
    }

    let clamped = if dop > one { one } else { dop };
    Ratio::<R>::new(clamped)
}
