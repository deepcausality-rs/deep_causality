/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2026" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Curvature tensor for RiemannMap HKT trait.
//!
//! The curvature tensor R^d_abc measures the holonomy of parallel transport
//! around infinitesimal loops in a manifold.

use deep_causality_metric::Metric;
use deep_causality_tensor::CausalTensor;
use std::marker::PhantomData;

/// Symmetry properties of curvature tensors.
///
/// Different curvature tensors have different symmetry properties
/// that can be exploited for storage and computation efficiency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CurvatureSymmetry {
    /// Riemann curvature tensor symmetries:
    /// - R_abcd = -R_bacd (antisymmetric in first pair)
    /// - R_abcd = -R_abdc (antisymmetric in second pair)
    /// - R_abcd = R_cdab (pair exchange)
    /// - R_[abc]d = 0 (first Bianchi identity)
    Riemann,

    /// Weyl tensor (traceless part of Riemann).
    /// Satisfies all Riemann symmetries plus tracelessness.
    Weyl,

    /// Ricci tensor (contraction of Riemann).
    /// Symmetric rank-2 tensor: R_μν = R^ρ_μρν
    Ricci,

    /// No special symmetry.
    None,
}

/// A rank-4 curvature tensor for RiemannMap operations.
///
/// This type represents curvature tensors like the Riemann tensor R^d_abc,
/// which measures how vectors rotate when parallel transported around loops.
///
/// # Type Parameters
///
/// * `A` - First direction type (u in R(u,v)w)
/// * `B` - Second direction type (v in R(u,v)w)
/// * `C` - Vector being transported (w in R(u,v)w)
/// * `D` - Result type (output direction)
///
/// # Mathematical Definition
///
/// The Riemann curvature tensor measures parallel transport holonomy:
/// R(u,v)w = ∇_u∇_v w - ∇_v∇_u w - ∇_[u,v] w
///
/// In components: (R(u,v)w)^d = R^d_abc u^a v^b w^c
#[derive(Debug, Clone)]
pub struct CurvatureTensor<A, B, C, D> {
    /// Tensor components R^d_abc.
    /// Shape: [dim, dim, dim, dim]
    components: CausalTensor<f64>,

    /// Spacetime metric for index raising/lowering.
    metric: Metric,

    /// Symmetry information for optimization.
    symmetry: CurvatureSymmetry,

    /// Spacetime dimension.
    dim: usize,

    /// Phantom data for type parameters.
    _phantom: PhantomData<(A, B, C, D)>,
}

// ============================================================================
// Constructors
// ============================================================================

impl<A, B, C, D> CurvatureTensor<A, B, C, D> {
    /// Creates a new curvature tensor from components.
    ///
    /// # Arguments
    ///
    /// * `components` - Tensor components, shape [dim, dim, dim, dim]
    /// * `metric` - Spacetime metric signature
    /// * `symmetry` - Symmetry properties of the tensor
    /// * `dim` - Spacetime dimension
    ///
    /// # Panics
    ///
    /// Panics if component shape doesn't match [dim, dim, dim, dim].
    pub fn new(
        components: CausalTensor<f64>,
        metric: Metric,
        symmetry: CurvatureSymmetry,
        dim: usize,
    ) -> Self {
        let shape = components.shape();
        assert_eq!(
            shape,
            [dim, dim, dim, dim],
            "CurvatureTensor components must have shape [{d}, {d}, {d}, {d}], got {:?}",
            shape,
            d = dim
        );

        Self {
            components,
            metric,
            symmetry,
            dim,
            _phantom: PhantomData,
        }
    }

    /// Creates a flat (zero curvature) tensor with Minkowski metric.
    pub fn flat(dim: usize) -> Self {
        Self::flat_with_metric(dim, Metric::Minkowski(dim))
    }

    /// Creates a flat (zero curvature) tensor with specified metric.
    pub fn flat_with_metric(dim: usize, metric: Metric) -> Self {
        let shape = vec![dim, dim, dim, dim];
        let total = dim * dim * dim * dim;
        let data = vec![0.0; total];
        let components = CausalTensor::from_vec(data, &shape);

        Self {
            components,
            metric,
            symmetry: CurvatureSymmetry::Riemann,
            dim,
            _phantom: PhantomData,
        }
    }

    /// Creates a curvature tensor from a closure that generates components.
    ///
    /// # Arguments
    ///
    /// * `dim` - Spacetime dimension
    /// * `metric` - Spacetime metric
    /// * `symmetry` - Symmetry type
    /// * `generator` - Function (d, a, b, c) -> R^d_abc
    pub fn from_generator<F>(
        dim: usize,
        metric: Metric,
        symmetry: CurvatureSymmetry,
        mut generator: F,
    ) -> Self
    where
        F: FnMut(usize, usize, usize, usize) -> f64,
    {
        let total = dim * dim * dim * dim;
        let mut data = Vec::with_capacity(total);

        for d in 0..dim {
            for a in 0..dim {
                for b in 0..dim {
                    for c in 0..dim {
                        data.push(generator(d, a, b, c));
                    }
                }
            }
        }

        let components = CausalTensor::from_vec(data, &[dim, dim, dim, dim]);
        Self::new(components, metric, symmetry, dim)
    }
}

// ============================================================================
// Getters
// ============================================================================

impl<A, B, C, D> CurvatureTensor<A, B, C, D> {
    /// Returns a reference to the tensor components.
    #[inline]
    pub fn components(&self) -> &CausalTensor<f64> {
        &self.components
    }

    /// Returns the metric.
    #[inline]
    pub fn metric(&self) -> Metric {
        self.metric
    }

    /// Returns the symmetry type.
    #[inline]
    pub fn symmetry(&self) -> CurvatureSymmetry {
        self.symmetry
    }

    /// Returns the spacetime dimension.
    #[inline]
    pub fn dim(&self) -> usize {
        self.dim
    }

    /// Checks if the tensor is flat (all zero).
    pub fn is_flat(&self) -> bool {
        self.components
            .as_slice()
            .iter()
            .all(|&x| x.abs() < f64::EPSILON)
    }

    /// Gets component R^d_abc using row-major indexing.
    #[inline]
    pub fn get(&self, d: usize, a: usize, b: usize, c: usize) -> f64 {
        debug_assert!(d < self.dim && a < self.dim && b < self.dim && c < self.dim);
        let idx = d * self.dim * self.dim * self.dim + a * self.dim * self.dim + b * self.dim + c;
        self.components.as_slice()[idx]
    }
}

// ============================================================================
// Tensor Operations
// ============================================================================

impl<A, B, C, D> CurvatureTensor<A, B, C, D> {
    /// Contracts the curvature tensor with three vectors: R(u,v)w.
    ///
    /// Computes (R(u,v)w)^d = R^d_abc u^a v^b w^c
    ///
    /// # Arguments
    ///
    /// * `u` - First loop direction (contracts with index a)
    /// * `v` - Second loop direction (contracts with index b)
    /// * `w` - Vector being transported (contracts with index c)
    ///
    /// # Returns
    ///
    /// A vector representing the geodesic deviation.
    pub fn contract(&self, u: &[f64], v: &[f64], w: &[f64]) -> Vec<f64> {
        assert_eq!(u.len(), self.dim, "u dimension mismatch");
        assert_eq!(v.len(), self.dim, "v dimension mismatch");
        assert_eq!(w.len(), self.dim, "w dimension mismatch");

        let mut result = vec![0.0; self.dim];

        // (R(u,v)w)^d = R^d_abc u^a v^b w^c
        for (d, res_val) in result.iter_mut().enumerate() {
            let mut sum = 0.0;
            for (a, u_val) in u.iter().enumerate() {
                for (b, v_val) in v.iter().enumerate() {
                    for (c, w_val) in w.iter().enumerate() {
                        sum += self.get(d, a, b, c) * u_val * v_val * w_val;
                    }
                }
            }
            *res_val = sum;
        }

        result
    }

    /// Computes the Ricci tensor by contraction: R_μν = R^ρ_μρν.
    ///
    /// Returns a dim×dim matrix as a flat vector in row-major order.
    pub fn ricci_tensor(&self) -> Vec<f64> {
        let mut ricci = vec![0.0; self.dim * self.dim];

        for mu in 0..self.dim {
            for nu in 0..self.dim {
                let mut sum = 0.0;
                for rho in 0..self.dim {
                    // R_μν = R^ρ_μρν
                    sum += self.get(rho, mu, rho, nu);
                }
                ricci[mu * self.dim + nu] = sum;
            }
        }

        ricci
    }

    /// Computes the Ricci scalar R = g^μν R_μν.
    pub fn ricci_scalar(&self) -> f64 {
        let ricci = self.ricci_tensor();
        let mut scalar = 0.0;

        for mu in 0..self.dim {
            // Get metric component g^μμ (inverse metric diagonal for Minkowski-like)
            let g_inv = self.metric.sign_of_sq(mu) as f64;
            scalar += g_inv * ricci[mu * self.dim + mu];
        }

        scalar
    }

    /// Computes the Kretschmann scalar K = R_abcd R^abcd.
    ///
    /// This is a curvature invariant useful for detecting singularities.
    pub fn kretschmann_scalar(&self) -> f64 {
        let mut k = 0.0;

        for a in 0..self.dim {
            for b in 0..self.dim {
                for c in 0..self.dim {
                    for d in 0..self.dim {
                        // For simplicity, use R^d_abc directly
                        // Full implementation would lower indices with metric
                        let r = self.get(d, a, b, c);
                        k += r * r;
                    }
                }
            }
        }

        k
    }

    /// Computes the Einstein tensor G_μν = R_μν - (1/2) g_μν R.
    ///
    /// Returns a dim×dim matrix as a flat vector in row-major order.
    pub fn einstein_tensor(&self) -> Vec<f64> {
        let ricci = self.ricci_tensor();
        let r = self.ricci_scalar();
        let mut einstein = vec![0.0; self.dim * self.dim];

        for mu in 0..self.dim {
            for nu in 0..self.dim {
                // g_μν for Minkowski-like metrics
                let g_munu = if mu == nu {
                    self.metric.sign_of_sq(mu) as f64
                } else {
                    0.0
                };
                einstein[mu * self.dim + nu] = ricci[mu * self.dim + nu] - 0.5 * g_munu * r;
            }
        }

        einstein
    }

    /// Verifies the first Bianchi identity: R_[abc]d = 0.
    ///
    /// The cyclic sum R_abcd + R_bcad + R_cabd = 0.
    ///
    /// Returns the maximum violation (should be ~0 for valid Riemann tensors).
    pub fn check_bianchi_identity(&self) -> f64 {
        let mut max_violation: f64 = 0.0;

        for a in 0..self.dim {
            for b in 0..self.dim {
                for c in 0..self.dim {
                    for d in 0..self.dim {
                        // R_abcd + R_bcad + R_cabd should = 0
                        // Using R^d_abc convention
                        let sum =
                            self.get(d, a, b, c) + self.get(d, b, c, a) + self.get(d, c, a, b);
                        max_violation = max_violation.max(sum.abs());
                    }
                }
            }
        }

        max_violation
    }
}

// ============================================================================
// Type Conversion
// ============================================================================

impl<A, B, C, D> CurvatureTensor<A, B, C, D> {
    /// Converts to a CurvatureTensor with different type parameters.
    ///
    /// This is safe because the type parameters are phantom data only.
    pub fn cast<A2, B2, C2, D2>(self) -> CurvatureTensor<A2, B2, C2, D2> {
        CurvatureTensor {
            components: self.components,
            metric: self.metric,
            symmetry: self.symmetry,
            dim: self.dim,
            _phantom: PhantomData,
        }
    }
}
