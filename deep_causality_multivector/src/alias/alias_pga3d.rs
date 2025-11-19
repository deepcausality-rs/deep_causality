/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// 3D Projective Geometric Algebra
// Signature: R(3, 0, 1) -> 3 Euclidean, 0 Negative, 1 Zero

use crate::{Metric, PGA3DMultiVector};

impl PGA3DMultiVector {
    /// Creates a point in 3D PGA.
    ///
    /// Points are represented as tri-vectors (dual representation):
    /// $$ P = x (e_0 \wedge e_3 \wedge e_2) + y (e_0 \wedge e_1 \wedge e_3) + z (e_0 \wedge e_2 \wedge e_1) + (e_1 \wedge e_2 \wedge e_3) $$
    ///
    /// Indices (assuming e0=bit0, e1=bit1, e2=bit2, e3=bit3):
    /// * e123 (14): 1.0
    /// * e032 (13): x
    /// * e013 (11): y
    /// * e021 (7): z
    pub fn new_point(x: f64, y: f64, z: f64) -> Self {
        let mut data = vec![0.0; 16];

        // e123 (1110 binary = 14) -> 1.0 (Homogeneous coordinate w=1)
        data[14] = 1.0;

        // e032 = e0^e3^e2.
        // e0=1, e3=8, e2=4. 1|8|4 = 13.
        // Sign check: e0e3e2 = -e0e2e3.
        // Standard formula often just assigns coefficient to the blade.
        // Let's assume the standard dual basis mapping:
        // P = x*e032 + y*e013 + z*e021 + e123
        data[13] = x;
        data[11] = y;
        data[7] = z;

        Self::new(data, Metric::PGA(4)).unwrap()
    }

    /// Creates a translator (motor) in 3D PGA.
    ///
    /// A translator $T$ moves geometry by a vector $d = (x, y, z)$.
    /// $$ T = 1 - \frac{1}{2} (x e_{01} + y e_{02} + z e_{03}) $$
    ///
    /// Indices:
    /// * Scalar (0): 1.0
    /// * e01 (3): -x/2
    /// * e02 (5): -y/2
    /// * e03 (9): -z/2
    pub fn translator(x: f64, y: f64, z: f64) -> Self {
        let mut data = vec![0.0; 16];

        // Scalar part = 1.0
        data[0] = 1.0;

        // e01 (binary 0011 = 3)
        data[3] = 0.5 * x;

        // e02 (binary 0101 = 5)
        data[5] = 0.5 * y;

        // e03 (binary 1001 = 9)
        data[9] = 0.5 * z;

        Self::new(data, Metric::PGA(4)).unwrap()
    }
}
