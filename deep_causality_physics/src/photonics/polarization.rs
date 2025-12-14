/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{JonesVector, PhysicsError, PhysicsErrorEnum, Ratio, RayAngle, StokesVector};
use deep_causality_num::{Complex, DivisionAlgebra};
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
/// *   `Result<CausalTensor<Complex<f64>>, PhysicsError>` - The rotated Jones matrix.
pub fn jones_rotation_kernel(
    jones_matrix: &CausalTensor<Complex<f64>>,
    angle: RayAngle,
) -> Result<CausalTensor<Complex<f64>>, PhysicsError> {
    if jones_matrix.shape() != [2, 2] {
        return Err(PhysicsError::new(PhysicsErrorEnum::DimensionMismatch(
            "Jones matrix must be 2x2".into(),
        )));
    }

    let phi = angle.value();
    let c = phi.cos();
    let s = phi.sin();

    // M' = R(-phi) * M * R(phi)
    // We compute this manually since Complex<f64> does not implement PartialOrd required by CausalTensor::matmul

    // jones_matrix M data: [m00, m01, m10, m11]
    let m_data = jones_matrix.data();
    let m00 = m_data[0];
    let m01 = m_data[1];
    let m10 = m_data[2];
    let m11 = m_data[3];

    // R(phi) components
    let rc = Complex::new(c, 0.0);
    let rs = Complex::new(s, 0.0);

    // Intermediate: T = M * R(phi)
    // T = [m00, m01] * [ c, s]
    //     [m10, m11]   [-s, c]
    // t00 = m00*c - m01*s
    // t01 = m00*s + m01*c
    // t10 = m10*c - m11*s
    // t11 = m10*s + m11*c

    let t00 = m00 * rc - m01 * rs;
    let t01 = m00 * rs + m01 * rc;
    let t10 = m10 * rc - m11 * rs;
    let t11 = m10 * rs + m11 * rc;

    // Final: M' = R(-phi) * T
    // R(-phi) = [ c, -s]
    //           [ s,  c]
    // m'00 = c*t00 - s*t10
    // m'01 = c*t01 - s*t11
    // m'10 = s*t00 + c*t10
    // m'11 = s*t01 + c*t11

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
/// *   `Result<StokesVector, PhysicsError>` - The corresponding Stokes vector.
pub fn stokes_from_jones_kernel(jones: &JonesVector) -> Result<StokesVector, PhysicsError> {
    let t = jones.inner();
    if t.shape() != [2] {
        return Err(PhysicsError::new(PhysicsErrorEnum::DimensionMismatch(
            "Jones vector must be length 2".into(),
        )));
    }

    let ex = t.data()[0];
    let ey = t.data()[1];

    let ex_sq = ex.norm_sqr();
    let ey_sq = ey.norm_sqr();

    // conj(ey)
    let ey_conj = Complex::new(ey.re, -ey.im);
    // Ex * Ey*
    let cross = ex * ey_conj;

    let s0 = ex_sq + ey_sq;
    let s1 = ex_sq - ey_sq;
    let s2 = 2.0 * cross.re;
    let s3 = -2.0 * cross.im;

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
/// *   `Result<Ratio, PhysicsError>` - DOP (0 to 1).
pub fn degree_of_polarization_kernel(stokes: &StokesVector) -> Result<Ratio, PhysicsError> {
    let t = stokes.inner();
    if t.shape() != [4] {
        return Err(PhysicsError::new(PhysicsErrorEnum::DimensionMismatch(
            "Stokes vector must be length 4".into(),
        )));
    }

    let d = t.data();
    let s0 = d[0];
    let s1 = d[1];
    let s2 = d[2];
    let s3 = d[3];

    if s0 <= 0.0 {
        if s0 == 0.0 && s1 == 0.0 && s2 == 0.0 && s3 == 0.0 {
            return Ratio::new(0.0); // Zero intensity, undefined DOP, return 0
        }
        return Err(PhysicsError::new(
            PhysicsErrorEnum::PhysicalInvariantBroken("S0 must be positive".into()),
        ));
    }

    let pol_mag = (s1 * s1 + s2 * s2 + s3 * s3).sqrt();
    let dop = pol_mag / s0;

    if dop > 1.000001 {
        // Allow tiny float error
        return Err(PhysicsError::new(
            PhysicsErrorEnum::PhysicalInvariantBroken(format!(
                "DOP > 1 ({}), unphysical Stokes vector",
                dop
            )),
        ));
    }

    Ratio::new(dop.min(1.0))
}
