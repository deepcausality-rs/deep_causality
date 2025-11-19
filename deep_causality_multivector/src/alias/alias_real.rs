/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{Metric, RealMultiVector};

// Division Algebras (Standard Math)
impl RealMultiVector {
    /// Cl(0, 1): Isomorphic to Complex Numbers C
    /// Basis: {1, e1} where e1^2 = -1 (acts as i)
    pub fn new_complex_number(real: f64, imag: f64) -> Self {
        let data = vec![real, imag];
        Self::new(data, Metric::NonEuclidean(1)).unwrap()
    }

    /// Cl(0, 2): Isomorphic to Quaternions H
    /// Basis: {1, e1, e2, e12}. e1^2 = e2^2 = -1.
    pub fn new_quaternion(w: f64, x: f64, y: f64, z: f64) -> Self {
        let data = vec![w, x, y, z];
        Self::new(data, Metric::NonEuclidean(2)).unwrap()
    }

    /// Cl(1, 0): Isomorphic to Split-Complex (Hyperbolic) Numbers
    /// Basis: {1, e1} where e1^2 = +1 (acts as j)
    pub fn new_split_complex(a: f64, b: f64) -> Self {
        let data = vec![a, b];
        Self::new(data, Metric::Euclidean(1)).unwrap()
    }
}

// The Physics Algebras (Spacetime)

impl RealMultiVector {
    /// Cl(3, 0): Algebra of Physical Space (APS) / Pauli Algebra
    /// Used for non-relativistic quantum mechanics (Pauli Matrices).
    pub fn new_aps_vector(data: Vec<f64>) -> Self {
        // Dimension 3, Euclidean Metric (+ + +)
        Self::new(data, Metric::Euclidean(3)).unwrap()
    }

    /// Cl(1, 3): Space-Time Algebra (STA) / Dirac Algebra
    /// Used for Special Relativity and Maxwell's Equations.
    /// Metric: (+ - - -)
    pub fn new_spacetime_vector(data: Vec<f64>) -> Self {
        // Dimension 4, Minkowski Metric (+ - - -)
        Self::new(data, Metric::Minkowski(4)).unwrap()
    }

    /// Cl(4, 1): Conformal Geometric Algebra (CGA)
    /// Used for computer graphics and advanced robotics.
    /// 5 Dimensions. Metric (+ + + + -).
    ///
    /// Basis: e1, e2, e3, e+ (e4), e- (e5).
    /// e5^2 = -1.
    pub fn new_cga_vector(data: Vec<f64>) -> Self {
        // 5 Dimensions.
        // Mask: We want one dimension to be negative.
        // Index 4 (e5) corresponds to bit 4 (16).
        // neg_mask = 16. zero_mask = 0.
        Self::new(
            data,
            Metric::Custom {
                dim: 5,
                neg_mask: 16,
                zero_mask: 0,
            },
        )
        .unwrap()
    }
}
