/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The Cayley-Menger regression, which is why elimination must search for its pivot.
//!
//! # What this guards
//!
//! `deep_causality_topology` computes simplex volumes from Cayley-Menger determinants at
//! `regge_geometry/curvature.rs:254` and `manifold/geometry/mod.rs:72`. A Cayley-Menger matrix has
//! **`m[0][0] = 0` by construction** — `manifold/geometry/mod.rs:41` allocates zeros and writes
//! `one` only into indices `1..matrix_dim`.
//!
//! The determinant this replaced, `gaussian_determinant` in `deep_causality_topology`'s
//! `simplicial_complex/lazy_hodge_star.rs`, took `mat[i][i]`
//! as its pivot and returns zero when it is small. Consolidating the two Laplace determinants onto
//! it unpivoted returns **zero for every simplex volume**. Measured on the regular unit tetrahedron
//! below:
//!
//! | method | det | vol² | vol |
//! |---|---|---|---|
//! | Laplace, what topology does today | 4.0 | 0.013888888889 | 0.117851130198 |
//! | elimination as `gaussian_determinant` was written | **0.0** | 0.0 | **NaN** |
//! | elimination with partial pivoting | 4.0 | 0.013888888889 | 0.117851130198 |
//!
//! That helper is correct today only because its own caller feeds it a Gram matrix, whose diagonal
//! is strictly positive.

use alloc::vec;
use alloc::vec::Vec;

/// The 5×5 Cayley-Menger matrix of a regular tetrahedron with unit edge length.
///
/// Row-major. The border is `0` then four `1`s; the interior is the squared edge lengths, all `1`,
/// with a zero diagonal.
pub fn regular_unit_tetrahedron() -> (Vec<f64>, usize, usize) {
    (
        vec![
            0.0, 1.0, 1.0, 1.0, 1.0, //
            1.0, 0.0, 1.0, 1.0, 1.0, //
            1.0, 1.0, 0.0, 1.0, 1.0, //
            1.0, 1.0, 1.0, 0.0, 1.0, //
            1.0, 1.0, 1.0, 1.0, 0.0,
        ],
        5,
        5,
    )
}

/// The determinant of [`regular_unit_tetrahedron`].
pub const TETRAHEDRON_CM_DETERMINANT: f64 = 4.0;

/// The volume of a regular unit tetrahedron, `√2 ⁄ 12`.
///
/// The exact value, not a measured one. `vol² = det / 288`, so `vol² = 4/288 = 1/72` and
/// `vol = 1/√72 = √2/12`.
pub const TETRAHEDRON_VOLUME: f64 = 0.117_851_130_197_757_92;

/// The 4×4 Cayley-Menger matrix of a right triangle with legs of length 1.
///
/// The lower-order case, which behaves the same way: determinant `-4`, and `0` unpivoted.
pub fn right_triangle() -> (Vec<f64>, usize, usize) {
    (
        vec![
            0.0, 1.0, 1.0, 1.0, //
            1.0, 0.0, 1.0, 1.0, //
            1.0, 1.0, 0.0, 2.0, //
            1.0, 1.0, 2.0, 0.0,
        ],
        4,
        4,
    )
}

/// The determinant of [`right_triangle`].
pub const RIGHT_TRIANGLE_CM_DETERMINANT: f64 = -4.0;

/// Converts a Cayley-Menger determinant of order `n` into the squared content of the simplex.
///
/// For a `k`-simplex the matrix has order `k + 2` and the divisor is `(-1)^(k+1) · 2^k · (k!)²`.
/// For the tetrahedron, `k = 3`, so the divisor is `288`.
pub fn cm_determinant_to_volume_squared(det: f64, order: usize) -> f64 {
    let k = order as i32 - 2;
    let sign = if (k + 1) % 2 == 0 { 1.0 } else { -1.0 };
    let two_k = (1u64 << k) as f64;
    let mut factorial = 1.0_f64;
    for i in 1..=k {
        factorial *= i as f64;
    }
    det / (sign * two_k * factorial * factorial)
}
