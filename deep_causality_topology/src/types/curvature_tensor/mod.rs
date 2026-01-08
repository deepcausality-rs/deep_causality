/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Curvature tensor for RiemannMap HKT trait.
//!
//! The curvature tensor R^d_abc measures the holonomy of parallel transport
//! around infinitesimal loops in a manifold.

use crate::TensorVector;
use deep_causality_metric::Metric;
use deep_causality_num::{Field, Float};
use deep_causality_tensor::{CausalTensor, TensorData};
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

/// Type alias for generic curvature tensor
pub type CurvatureTensorVector<T> =
    CurvatureTensor<T, TensorVector<T>, TensorVector<T>, TensorVector<T>, TensorVector<T>>;

/// A rank-4 curvature tensor for RiemannMap operations.
///
/// This type represents curvature tensors like the Riemann tensor R^d_abc,
/// which measures how vectors rotate when parallel transported around loops.
///
/// # Type Parameters
///
/// * `T` - Scalar type (e.g., f64, DoubleFloat)
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
pub struct CurvatureTensor<T, A, B, C, D> {
    /// Tensor components R^d_abc.
    /// Shape: [dim, dim, dim, dim]
    components: CausalTensor<T>,

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

impl<T, A, B, C, D> CurvatureTensor<T, A, B, C, D>
where
    T: TensorData + Clone,
{
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
        components: CausalTensor<T>,
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
}

impl<T, A, B, C, D> CurvatureTensor<T, A, B, C, D>
where
    T: TensorData + Field + Float + Clone + From<f64> + Into<f64>,
{
    /// Creates a flat (zero curvature) tensor with Minkowski metric.
    pub fn flat(dim: usize) -> Self {
        Self::flat_with_metric(dim, Metric::Minkowski(dim))
    }

    /// Creates a flat (zero curvature) tensor with specified metric.
    pub fn flat_with_metric(dim: usize, metric: Metric) -> Self {
        let shape = vec![dim, dim, dim, dim];
        let total = dim * dim * dim * dim;
        let data: Vec<T> = (0..total).map(|_| <T as From<f64>>::from(0.0)).collect();
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
        F: FnMut(usize, usize, usize, usize) -> T,
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

impl<T, A, B, C, D> CurvatureTensor<T, A, B, C, D>
where
    T: TensorData + Clone,
{
    /// Returns a reference to the tensor components.
    #[inline]
    pub fn components(&self) -> &CausalTensor<T> {
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
}

impl<T, A, B, C, D> CurvatureTensor<T, A, B, C, D>
where
    T: TensorData + Field + Float + Clone + From<f64> + Into<f64>,
{
    /// Checks if the tensor is flat (all zero).
    pub fn is_flat(&self) -> bool {
        let eps: f64 = f64::EPSILON;
        self.components.as_slice().iter().all(|x| {
            let val: f64 = x.clone().into();
            val.abs() < eps
        })
    }

    /// Gets component R^d_abc using row-major indexing.
    #[inline]
    pub fn get(&self, d: usize, a: usize, b: usize, c: usize) -> T {
        debug_assert!(d < self.dim && a < self.dim && b < self.dim && c < self.dim);
        let idx = d * self.dim * self.dim * self.dim + a * self.dim * self.dim + b * self.dim + c;
        self.components.as_slice()[idx].clone()
    }
}

// ============================================================================
// Tensor Operations
// ============================================================================

impl<T, A, B, C, D> CurvatureTensor<T, A, B, C, D>
where
    T: TensorData + Field + Float + Clone + From<f64> + Into<f64>,
{
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
    pub fn contract(&self, u: &[T], v: &[T], w: &[T]) -> Vec<T> {
        assert_eq!(u.len(), self.dim, "u dimension mismatch");
        assert_eq!(v.len(), self.dim, "v dimension mismatch");
        assert_eq!(w.len(), self.dim, "w dimension mismatch");

        let mut result: Vec<T> = (0..self.dim).map(|_| <T as From<f64>>::from(0.0)).collect();

        // (R(u,v)w)^d = R^d_abc u^a v^b w^c
        for (d, res_val) in result.iter_mut().enumerate() {
            let mut sum = <T as From<f64>>::from(0.0);
            for (a, u_val) in u.iter().enumerate() {
                for (b, v_val) in v.iter().enumerate() {
                    for (c, w_val) in w.iter().enumerate() {
                        let r = self.get(d, a, b, c);
                        sum = sum + r * u_val.clone() * v_val.clone() * w_val.clone();
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
    pub fn ricci_tensor(&self) -> Vec<T> {
        let mut ricci: Vec<T> = (0..self.dim * self.dim)
            .map(|_| <T as From<f64>>::from(0.0))
            .collect();

        for mu in 0..self.dim {
            for nu in 0..self.dim {
                let mut sum = <T as From<f64>>::from(0.0);
                for rho in 0..self.dim {
                    // R_μν = R^ρ_μρν
                    sum = sum + self.get(rho, mu, rho, nu);
                }
                ricci[mu * self.dim + nu] = sum;
            }
        }

        ricci
    }

    /// Computes the Ricci scalar R = g^μν R_μν.
    pub fn ricci_scalar(&self) -> T {
        let ricci = self.ricci_tensor();
        let mut scalar = <T as From<f64>>::from(0.0);

        for mu in 0..self.dim {
            // Get metric component g^μμ (inverse metric diagonal for Minkowski-like)
            let g_inv = <T as From<f64>>::from(self.metric.sign_of_sq(mu) as f64);
            scalar = scalar + g_inv * ricci[mu * self.dim + mu].clone();
        }

        scalar
    }

    /// Computes the Kretschmann scalar K = R_abcd R^abcd.
    ///
    /// This is a curvature invariant useful for detecting singularities.
    pub fn kretschmann_scalar(&self) -> T {
        let mut k = <T as From<f64>>::from(0.0);

        for a in 0..self.dim {
            for b in 0..self.dim {
                for c in 0..self.dim {
                    for d in 0..self.dim {
                        // For simplicity, use R^d_abc directly
                        // Full implementation would lower indices with metric
                        let r = self.get(d, a, b, c);
                        k = k + r.clone() * r;
                    }
                }
            }
        }

        k
    }

    /// Computes the Einstein tensor G_μν = R_μν - (1/2) g_μν R.
    ///
    /// Returns a dim×dim matrix as a flat vector in row-major order.
    pub fn einstein_tensor(&self) -> Vec<T> {
        let ricci = self.ricci_tensor();
        let r = self.ricci_scalar();
        let half = <T as From<f64>>::from(0.5);
        let mut einstein: Vec<T> = (0..self.dim * self.dim)
            .map(|_| <T as From<f64>>::from(0.0))
            .collect();

        for mu in 0..self.dim {
            for nu in 0..self.dim {
                // g_μν for Minkowski-like metrics
                let g_munu = if mu == nu {
                    <T as From<f64>>::from(self.metric.sign_of_sq(mu) as f64)
                } else {
                    <T as From<f64>>::from(0.0)
                };
                einstein[mu * self.dim + nu] =
                    ricci[mu * self.dim + nu].clone() - half.clone() * g_munu * r.clone();
            }
        }

        einstein
    }

    /// Computes the Weyl conformal curvature tensor.
    ///
    /// The Weyl tensor is the traceless part of the Riemann tensor,
    /// representing the purely gravitational degrees of freedom (tidal forces).
    ///
    /// C_abcd = R_abcd - (2/(n-2))(g_a[c R_d]b - g_b[c R_d]a)
    ///        + (2/((n-1)(n-2))) R g_a[c g_d]b
    ///
    /// where n is the dimension (must be >= 3).
    ///
    /// Returns a rank-4 tensor [dim, dim, dim, dim] representing C^a_bcd.
    pub fn weyl_tensor(&self) -> Vec<T> {
        let n = self.dim;
        if n < 3 {
            // Weyl tensor is identically zero in dimensions < 3
            return (0..n * n * n * n)
                .map(|_| <T as From<f64>>::from(0.0))
                .collect();
        }

        let ricci = self.ricci_tensor();
        let r = self.ricci_scalar();

        let mut weyl: Vec<T> = (0..n * n * n * n)
            .map(|_| <T as From<f64>>::from(0.0))
            .collect();

        // Prefactors
        let factor1 = <T as From<f64>>::from(2.0 / (n as f64 - 2.0));
        let factor2 = <T as From<f64>>::from(2.0 / ((n as f64 - 1.0) * (n as f64 - 2.0)));
        let half = <T as From<f64>>::from(0.5);

        for a in 0..n {
            for b in 0..n {
                for c in 0..n {
                    for d in 0..n {
                        // R^a_bcd component (in the convention where first index is up)
                        let r_abcd = self.get(a, b, c, d);

                        // Metric components (using our stored metric for diagonal)
                        let g_ac = if a == c {
                            <T as From<f64>>::from(self.metric.sign_of_sq(a) as f64)
                        } else {
                            <T as From<f64>>::from(0.0)
                        };
                        let g_bd = if b == d {
                            <T as From<f64>>::from(self.metric.sign_of_sq(b) as f64)
                        } else {
                            <T as From<f64>>::from(0.0)
                        };
                        let g_ad = if a == d {
                            <T as From<f64>>::from(self.metric.sign_of_sq(a) as f64)
                        } else {
                            <T as From<f64>>::from(0.0)
                        };
                        let g_bc = if b == c {
                            <T as From<f64>>::from(self.metric.sign_of_sq(b) as f64)
                        } else {
                            <T as From<f64>>::from(0.0)
                        };

                        // Ricci components
                        let r_ac = ricci[a * n + c].clone();
                        let r_bd = ricci[b * n + d].clone();
                        let r_ad = ricci[a * n + d].clone();
                        let r_bc = ricci[b * n + c].clone();

                        // Weyl formula
                        let term1 = r_abcd;
                        let term2 = factor1.clone()
                            * half.clone()
                            * (g_ac.clone() * r_bd - g_ad.clone() * r_bc - g_bc.clone() * r_ad
                                + g_bd.clone() * r_ac);
                        let term3 = factor2.clone()
                            * half.clone()
                            * r.clone()
                            * (g_ac * g_bd - g_ad * g_bc);

                        weyl[a * n * n * n + b * n * n + c * n + d] = term1 - term2 + term3;
                    }
                }
            }
        }

        weyl
    }

    /// Verifies the first Bianchi identity: R_[abc]d = 0.
    ///
    /// The cyclic sum R_abcd + R_bcad + R_cabd = 0.
    ///
    /// Returns the maximum violation (should be ~0 for valid Riemann tensors).
    pub fn check_bianchi_identity(&self) -> T {
        let mut max_violation = <T as From<f64>>::from(0.0);

        for a in 0..self.dim {
            for b in 0..self.dim {
                for c in 0..self.dim {
                    for d in 0..self.dim {
                        // R_abcd + R_bcad + R_cabd should = 0
                        let sum =
                            self.get(d, a, b, c) + self.get(d, b, c, a) + self.get(d, c, a, b);
                        let abs_sum = <T as Float>::abs(sum);
                        if abs_sum > max_violation.clone() {
                            max_violation = abs_sum;
                        }
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

impl<T, A, B, C, D> CurvatureTensor<T, A, B, C, D>
where
    T: TensorData + Clone,
{
    /// Converts to a CurvatureTensor with different type parameters.
    ///
    /// This is safe because the type parameters are phantom data only.
    pub fn cast<A2, B2, C2, D2>(self) -> CurvatureTensor<T, A2, B2, C2, D2> {
        CurvatureTensor {
            components: self.components,
            metric: self.metric,
            symmetry: self.symmetry,
            dim: self.dim,
            _phantom: PhantomData,
        }
    }
}
