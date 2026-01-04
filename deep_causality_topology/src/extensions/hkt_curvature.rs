/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2026" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! HKT4 witness and RiemannMap implementation for CurvatureTensor.
//!
//! This module provides the RiemannMap trait implementation for CurvatureTensor,
//! enabling curvature contraction and scattering matrix operations.

use crate::types::curvature_tensor::CurvatureTensor;
use deep_causality_haft::{HKT4Unbound, NoConstraint, RiemannMap, Satisfies};

/// HKT4 witness for CurvatureTensor<A, B, C, D>.
///
/// This witness enables CurvatureTensor to participate in HKT4 operations
/// like RiemannMap (curvature contraction and scattering).
///
/// # Type Structure
///
/// CurvatureTensor<A, B, C, D> where:
/// - A: First index type (u in R(u,v)w)
/// - B: Second index type (v)
/// - C: Third index type (w)
/// - D: Result index type (output direction)
///
/// Note: The type parameters are phantom data; actual components are f64.
#[derive(Debug, Clone, Copy, Default)]
pub struct CurvatureTensorWitness;

impl HKT4Unbound for CurvatureTensorWitness {
    type Constraint = NoConstraint;
    type Type<A, B, C, D>
        = CurvatureTensor<A, B, C, D>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        C: Satisfies<NoConstraint>,
        D: Satisfies<NoConstraint>;
}

// ============================================================================
// RiemannMap Implementation
// ============================================================================

/// Vector wrapper for RiemannMap operations.
///
/// Since RiemannMap expects generic types A, B, C, D as input vectors,
/// we provide a concrete vector type for physics operations.
#[derive(Debug, Clone, PartialEq)]
pub struct TensorVector {
    /// Vector components.
    pub data: Vec<f64>,
}

impl TensorVector {
    /// Creates a new tensor vector from a slice.
    pub fn new(data: &[f64]) -> Self {
        Self {
            data: data.to_vec(),
        }
    }

    /// Creates a zero vector of given dimension.
    pub fn zeros(dim: usize) -> Self {
        Self {
            data: vec![0.0; dim],
        }
    }

    /// Creates a basis vector e_i.
    pub fn basis(dim: usize, i: usize) -> Self {
        let mut data = vec![0.0; dim];
        if i < dim {
            data[i] = 1.0;
        }
        Self { data }
    }

    /// Returns the dimension.
    pub fn dim(&self) -> usize {
        self.data.len()
    }

    /// Returns a slice of the data.
    pub fn as_slice(&self) -> &[f64] {
        &self.data
    }
}

impl From<Vec<f64>> for TensorVector {
    fn from(data: Vec<f64>) -> Self {
        Self { data }
    }
}

impl From<TensorVector> for Vec<f64> {
    fn from(v: TensorVector) -> Self {
        v.data
    }
}

impl RiemannMap<CurvatureTensorWitness> for CurvatureTensorWitness {
    /// Computes curvature contraction R(u,v)w.
    ///
    /// # Mathematical Definition
    ///
    /// The Riemann curvature tensor measures parallel transport holonomy:
    /// R(u,v)w = ∇_u∇_v w - ∇_v∇_u w - ∇_[u,v] w
    ///
    /// In components: (R(u,v)w)^d = R^d_abc u^a v^b w^c
    ///
    /// # Type Specialization
    ///
    /// This implementation works with `TensorVector` types. For other types
    /// that implement `Into<Vec<f64>>`, convert before calling.
    fn curvature<A, B, C, D>(tensor: CurvatureTensor<A, B, C, D>, u: A, v: B, w: C) -> D
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        C: Satisfies<NoConstraint>,
        D: Satisfies<NoConstraint>,
    {
        // This generic implementation requires runtime type conversion.
        // For type-safe operations, use CurvatureTensor::contract directly.
        //
        // The generic HKT trait signature doesn't allow us to constrain
        // A, B, C to be specifically TensorVector. The caller must ensure
        // proper types are used or use the concrete methods.
        //
        // For production use with typed vectors:
        let _ = (tensor, u, v, w);
        panic!(
            "Use CurvatureTensor::contract() for curvature computation. \
             The generic RiemannMap trait cannot be fully type-safe for tensors."
        )
    }

    /// Computes S-matrix scattering: (A, B) → (C, D).
    ///
    /// # Physics Interpretation
    ///
    /// Models a 2-to-2 scattering process where two incoming particles
    /// interact and produce two outgoing particles.
    ///
    /// The tensor encodes the scattering amplitude:
    /// M = ⟨out| S |in⟩ = ⟨C, D| S |A, B⟩
    fn scatter<A, B, C, D>(interaction: CurvatureTensor<A, B, C, D>, in_1: A, in_2: B) -> (C, D)
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        C: Satisfies<NoConstraint>,
        D: Satisfies<NoConstraint>,
    {
        // Same limitation as curvature - the generic trait signature
        // doesn't allow type-safe tensor operations.
        let _ = (interaction, in_1, in_2);
        panic!(
            "Use specialized scattering methods for production S-matrix computation. \
             The generic RiemannMap trait cannot be fully type-safe for tensors."
        )
    }
}

// ============================================================================
// Concrete RiemannMap Operations (Production-Ready)
// ============================================================================

impl CurvatureTensorWitness {
    /// Computes geodesic deviation for concrete vector types.
    ///
    /// This is the production-ready version of the curvature operation.
    ///
    /// # Arguments
    ///
    /// * `tensor` - The Riemann curvature tensor
    /// * `u` - First loop direction
    /// * `v` - Second loop direction
    /// * `w` - Separation vector
    ///
    /// # Returns
    ///
    /// The geodesic deviation vector (acceleration between nearby geodesics).
    pub fn geodesic_deviation(
        tensor: &CurvatureTensor<TensorVector, TensorVector, TensorVector, TensorVector>,
        u: &TensorVector,
        v: &TensorVector,
        w: &TensorVector,
    ) -> TensorVector {
        let result = tensor.contract(u.as_slice(), v.as_slice(), w.as_slice());
        TensorVector::from(result)
    }

    /// Computes scattering amplitude for concrete vector inputs.
    ///
    /// # Arguments
    ///
    /// * `tensor` - The interaction tensor
    /// * `in_1` - First incoming state
    /// * `in_2` - Second incoming state
    ///
    /// # Returns
    ///
    /// A tuple of (out_1, out_2) outgoing states.
    pub fn scatter_vectors(
        tensor: &CurvatureTensor<TensorVector, TensorVector, TensorVector, TensorVector>,
        in_1: &TensorVector,
        in_2: &TensorVector,
    ) -> (TensorVector, TensorVector) {
        let dim = tensor.dim();

        // S-matrix scattering: sum over internal indices
        // out_c^μ = S^μν_αβ in_1^α in_2^β (contracted over α, β for first output)
        // out_d^μ = S^μν_αβ in_1^α in_2^β (contracted differently for second output)

        let mut out_1 = vec![0.0; dim];
        let mut out_2 = vec![0.0; dim];

        for c in 0..dim {
            for d in 0..dim {
                let mut amplitude = 0.0;
                for a in 0..dim {
                    for b in 0..dim {
                        amplitude += tensor.get(c, a, b, d) * in_1.data[a] * in_2.data[b];
                    }
                }
                // Split amplitude between two outputs
                out_1[c] += amplitude * 0.5;
                out_2[d] += amplitude * 0.5;
            }
        }

        (TensorVector::from(out_1), TensorVector::from(out_2))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::curvature_tensor::CurvatureSymmetry;
    use deep_causality_metric::Metric;

    #[test]
    fn test_geodesic_deviation_flat() {
        let flat: CurvatureTensor<TensorVector, TensorVector, TensorVector, TensorVector> =
            CurvatureTensor::flat(4);

        let u = TensorVector::basis(4, 0);
        let v = TensorVector::basis(4, 1);
        let w = TensorVector::new(&[1.0, 2.0, 3.0, 4.0]);

        let deviation = CurvatureTensorWitness::geodesic_deviation(&flat, &u, &v, &w);

        // Flat spacetime has zero geodesic deviation
        assert!(deviation.data.iter().all(|&x| x.abs() < f64::EPSILON));
    }

    #[test]
    fn test_tensor_vector_operations() {
        let v = TensorVector::new(&[1.0, 2.0, 3.0]);
        assert_eq!(v.dim(), 3);

        let basis = TensorVector::basis(4, 2);
        assert_eq!(basis.data[2], 1.0);
        assert_eq!(basis.data[0], 0.0);
    }

    #[test]
    fn test_curved_tensor_contraction() {
        // Create a simple non-flat curvature tensor
        let tensor: CurvatureTensor<TensorVector, TensorVector, TensorVector, TensorVector> =
            CurvatureTensor::from_generator(
                2,
                Metric::Euclidean(2),
                CurvatureSymmetry::None,
                |d, a, b, c| {
                    if d == 0 && a == 0 && b == 1 && c == 0 {
                        1.0 // R^0_010 = 1
                    } else {
                        0.0
                    }
                },
            );

        let u = TensorVector::new(&[1.0, 0.0]);
        let v = TensorVector::new(&[0.0, 1.0]);
        let w = TensorVector::new(&[1.0, 0.0]);

        let result = CurvatureTensorWitness::geodesic_deviation(&tensor, &u, &v, &w);

        // R(u,v)w with R^0_010 = 1 should give [1, 0]
        assert!((result.data[0] - 1.0).abs() < f64::EPSILON);
        assert!(result.data[1].abs() < f64::EPSILON);
    }
}
