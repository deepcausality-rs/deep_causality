/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{Complex64, ComplexMultiVector, Metric};

impl ComplexMultiVector {
    /// Cl_C(2): Complex Quaternions / Pauli Algebra over C
    /// Isomorphic to Biquaternions.
    pub fn new_complex_pauli(data: Vec<Complex64>) -> Self {
        Self::new(data, Metric::Euclidean(2)).unwrap()
    }

    /// The Dixon Algebra (Cl_C(6))
    /// Often associated with the algebra acting on Octonions.
    pub fn new_dixon_algebra(data: Vec<Complex64>) -> Self {
        Self::new(data, Metric::Euclidean(6)).unwrap()
    }

    /// Cl_C(6): The algebra acting on Octonions (via Left Multiplication)
    /// As described in the paper (Eq 21).
    pub fn new_octonion_operator(data: Vec<Complex64>) -> Self {
        Self::new(data, Metric::Euclidean(6)).unwrap()
    }

    /// Cl_C(10): The Grand Unified Algebra (Spin(10))
    /// This is the container for the entire roadmap in the paper.
    pub fn new_gut_algebra(data: Vec<Complex64>) -> Self {
        // 10 Dimensions, 1024 Complex Coefficients.
        Self::new(data, Metric::Euclidean(10)).unwrap()
    }
}
