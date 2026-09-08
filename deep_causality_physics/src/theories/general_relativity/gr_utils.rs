/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::PhysicsError;
use deep_causality_algebra::Field;
use deep_causality_linear::{determinant_3x3, determinant_4x4, inverse_3x3, inverse_4x4};
use deep_causality_num::Float;
use deep_causality_tensor::CausalTensor;

/// Computes the inverse of a 4x4 matrix, extracting it from a potentially larger tensor (e.g. 4x6 connection).
/// Returns error if determinant is near zero (singular metric).
pub(crate) fn invert_4x4<T>(t: &CausalTensor<T>) -> Result<[T; 16], PhysicsError>
where
    T: Field + Float + Copy + From<f64> + Into<f64>,
{
    let shape = t.shape();
    let data = t.as_slice();

    // Determine stride based on last dimension
    // Expecting structure [..., 4, last_dim] or just [4, last_dim]
    // If rank is 2: [rows, cols]
    // If rank 3: [points, rows, cols] - we invert the first point or check consistency?
    let cols = *shape.last().unwrap_or(&4);
    if cols < 4 {
        return Err(PhysicsError::DimensionMismatch(
            "Metric tensor last dimension must be at least 4".into(),
        ));
    }

    // Check total size
    if data.len() < 4 * cols {
        return Err(PhysicsError::DimensionMismatch(
            "Metric tensor too small".into(),
        ));
    }

    // Extract the 4×4 block with stride `cols`; element (r, c) sits at index r·cols + c.
    let block: [[T; 4]; 4] = core::array::from_fn(|r| core::array::from_fn(|c| data[r * cols + c]));

    // The near-singular threshold stays here rather than moving into `deep_causality_linear`.
    // "Near singular" is a statement about *this metric* — a coordinate chart breaking down, an
    // horizon being approached — not a property of the matrix, and the crate refuses only an
    // exactly zero determinant for that reason (`unified-math-next` task 6.7).
    let eps = <T as From<f64>>::from(1e-12);
    if determinant_4x4(&block).abs() < eps {
        return Err(PhysicsError::NumericalInstability(
            "Metric determinant is near zero (singular)".into(),
        ));
    }

    let inv = inverse_4x4(&block)?;
    let mut out = [T::zero(); 16];
    for (r, row) in inv.iter().enumerate() {
        out[r * 4..r * 4 + 4].copy_from_slice(row);
    }
    Ok(out)
}

/// Computes the inverse of a 3x3 matrix.
pub(crate) fn invert_3x3<T>(m: [[T; 3]; 3]) -> Result<[[T; 3]; 3], PhysicsError>
where
    T: Field + Float + Copy + From<f64> + Into<f64>,
{
    // As above: the `1e-14` near-singularity threshold is this solver's judgement about its own
    // spatial metric and stays at the call site; the crate refuses only an exact zero.
    let eps = <T as From<f64>>::from(1e-14);
    if determinant_3x3(&m).abs() < eps {
        return Err(PhysicsError::NumericalInstability(
            "Singular spatial metric (det ~ 0)".to_string(),
        ));
    }

    Ok(inverse_3x3(&m)?)
}
