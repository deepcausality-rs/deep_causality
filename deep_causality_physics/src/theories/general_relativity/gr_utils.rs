/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::PhysicsError;
use deep_causality_tensor::CausalTensor;

/// Computes the inverse of a 4x4 matrix, extracting it from a potentially larger tensor (e.g. 4x6 connection).
/// Returns error if determinant is near zero (singular metric).
pub(crate) fn invert_4x4(t: &CausalTensor<f64>) -> Result<[f64; 16], PhysicsError> {
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

    // Extract 4x4 block with stride `cols`
    // Element at (r, c) is at index r * cols + c
    let at = |r: usize, c: usize| data[r * cols + c];

    // Sub-expressions for cofactor expansion
    let s0 = at(0, 0) * at(1, 1) - at(0, 1) * at(1, 0);
    let s1 = at(0, 0) * at(1, 2) - at(0, 2) * at(1, 0);
    let s2 = at(0, 0) * at(1, 3) - at(0, 3) * at(1, 0);
    let s3 = at(0, 1) * at(1, 2) - at(0, 2) * at(1, 1);
    let s4 = at(0, 1) * at(1, 3) - at(0, 3) * at(1, 1);
    let s5 = at(0, 2) * at(1, 3) - at(0, 3) * at(1, 2);

    let c5 = at(2, 2) * at(3, 3) - at(2, 3) * at(3, 2);
    let c4 = at(2, 1) * at(3, 3) - at(2, 3) * at(3, 1);
    let c3 = at(2, 1) * at(3, 2) - at(2, 2) * at(3, 1);
    let c2 = at(2, 0) * at(3, 3) - at(2, 3) * at(3, 0);
    let c1 = at(2, 0) * at(3, 2) - at(2, 2) * at(3, 0);
    let c0 = at(2, 0) * at(3, 1) - at(2, 1) * at(3, 0);

    // Determinant
    let det = s0 * c5 - s1 * c4 + s2 * c3 + s3 * c2 - s4 * c1 + s5 * c0;

    if det.abs() < 1e-12 {
        return Err(PhysicsError::NumericalInstability(
            "Metric determinant is near zero (singular)".into(),
        ));
    }

    let inv_det = 1.0 / det;

    // Adjugate matrix elements (transposed cofactors)
    // Row 0
    let b00 = at(1, 1) * c5 - at(1, 2) * c4 + at(1, 3) * c3;
    let b01 = -at(0, 1) * c5 + at(0, 2) * c4 - at(0, 3) * c3;
    let b02 = at(3, 1) * s5 - at(3, 2) * s4 + at(3, 3) * s3;
    let b03 = -at(2, 1) * s5 + at(2, 2) * s4 - at(2, 3) * s3;

    // Row 1
    let b10 = -at(1, 0) * c5 + at(1, 2) * c2 - at(1, 3) * c1;
    let b11 = at(0, 0) * c5 - at(0, 2) * c2 + at(0, 3) * c1;
    let b12 = -at(3, 0) * s5 + at(3, 2) * s2 - at(3, 3) * s1;
    let b13 = at(2, 0) * s5 - at(2, 2) * s2 + at(2, 3) * s1;

    // Row 2
    let b20 = at(1, 0) * c4 - at(1, 1) * c2 + at(1, 3) * c0;
    let b21 = -at(0, 0) * c4 + at(0, 1) * c2 - at(0, 3) * c0;
    let b22 = at(3, 0) * s4 - at(3, 1) * s2 + at(3, 3) * s0;
    let b23 = -at(2, 0) * s4 + at(2, 1) * s2 - at(2, 3) * s0;

    // Row 3
    let b30 = -at(1, 0) * c3 + at(1, 1) * c1 - at(1, 2) * c0;
    let b31 = at(0, 0) * c3 - at(0, 1) * c1 + at(0, 2) * c0;
    let b32 = -at(3, 0) * s3 + at(3, 1) * s1 - at(3, 2) * s0;
    let b33 = at(2, 0) * s3 - at(2, 1) * s1 + at(2, 2) * s0;

    Ok([
        inv_det * b00,
        inv_det * b01,
        inv_det * b02,
        inv_det * b03,
        inv_det * b10,
        inv_det * b11,
        inv_det * b12,
        inv_det * b13,
        inv_det * b20,
        inv_det * b21,
        inv_det * b22,
        inv_det * b23,
        inv_det * b30,
        inv_det * b31,
        inv_det * b32,
        inv_det * b33,
    ])
}
