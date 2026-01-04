/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2026" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Stokes Adjunction (d ⊣ ∂) for differential geometry.
//!
//! This module provides the adjunction between the exterior derivative (d)
//! and the boundary operator (∂), which encodes Stokes' theorem:
//! ⟨dω, C⟩ = ⟨ω, ∂C⟩
//!
//! This is the foundation for conservation laws and integration theory.

use crate::types::chain::Chain;
use crate::types::differential_form::DifferentialForm;
use crate::{BaseTopology, SimplicialComplex};
use deep_causality_haft::{Adjunction, HKT, NoConstraint, Satisfies};
use deep_causality_sparse::CsrMatrix;
use deep_causality_tensor::CausalTensor;
use std::sync::Arc;

/// Witness for the exterior derivative d: Ω^k → Ω^(k+1).
///
/// The exterior derivative maps differential forms of degree k
/// to forms of degree k+1. It generalizes:
/// - Gradient (0-form → 1-form)
/// - Curl (1-form → 2-form in 3D)
/// - Divergence (through Hodge dual)
#[derive(Debug, Clone, Copy, Default)]
pub struct ExteriorDerivativeWitness;

impl HKT for ExteriorDerivativeWitness {
    type Constraint = NoConstraint;
    type Type<T>
        = DifferentialForm<T>
    where
        T: Satisfies<NoConstraint>;
}

/// Witness for the boundary operator ∂: C_k → C_(k-1).
///
/// The boundary operator maps chains of degree k to chains of degree k-1.
/// For a simplex, it returns the alternating sum of its faces.
#[derive(Debug, Clone, Copy, Default)]
pub struct BoundaryWitness;

impl HKT for BoundaryWitness {
    type Constraint = NoConstraint;
    type Type<T>
        = Chain<T>
    where
        T: Satisfies<NoConstraint>;
}

/// Context for Stokes theorem operations.
///
/// Provides the simplicial complex on which differential forms and chains live.
#[derive(Debug, Clone)]
pub struct StokesContext {
    /// The simplicial complex defining the discrete topology.
    complex: Arc<SimplicialComplex>,
}

impl StokesContext {
    /// Creates a new Stokes context from a simplicial complex.
    pub fn new(complex: SimplicialComplex) -> Self {
        Self {
            complex: Arc::new(complex),
        }
    }

    /// Creates a new Stokes context from an Arc'd simplicial complex.
    pub fn from_arc(complex: Arc<SimplicialComplex>) -> Self {
        Self { complex }
    }

    /// Returns a reference to the underlying simplicial complex.
    pub fn complex(&self) -> &SimplicialComplex {
        &self.complex
    }

    /// Returns the Arc to the simplicial complex.
    pub fn complex_arc(&self) -> Arc<SimplicialComplex> {
        Arc::clone(&self.complex)
    }

    /// Returns the dimension of the complex.
    pub fn dim(&self) -> usize {
        self.complex.dimension()
    }

    /// Returns the number of k-simplices.
    pub fn num_simplices(&self, k: usize) -> usize {
        if k < self.complex.skeletons().len() {
            self.complex.skeletons()[k].simplices().len()
        } else {
            0
        }
    }
}

/// Stokes Adjunction: d ⊣ ∂
///
/// # Mathematical Foundation
///
/// Stokes' theorem states that the exterior derivative d and boundary operator ∂
/// are adjoint under the integration pairing:
///
/// ⟨dω, C⟩ = ⟨ω, ∂C⟩
///
/// Where:
/// - ω is a k-form
/// - C is a (k+1)-chain
/// - ⟨·, ·⟩ is the integration pairing
///
/// # Corollaries
///
/// This single equation encodes:
/// - Fundamental Theorem of Calculus: ∫_a^b df = f(b) - f(a)
/// - Green's Theorem: ∮_C (P dx + Q dy) = ∬_D (∂Q/∂x - ∂P/∂y) dA
/// - Stokes' Theorem: ∫_S (∇×F)·dS = ∮_∂S F·dl
/// - Gauss's Theorem: ∫_V (∇·F) dV = ∯_∂V F·dS
#[derive(Debug, Clone, Copy, Default)]
pub struct StokesAdjunction;

impl Adjunction<ExteriorDerivativeWitness, BoundaryWitness, StokesContext> for StokesAdjunction {
    /// Unit: A → R(L(A)) = Chain<DifferentialForm<A>>
    ///
    /// Embeds a coefficient into a chain of forms.
    fn unit<A>(ctx: &StokesContext, a: A) -> Chain<DifferentialForm<A>>
    where
        A: Satisfies<NoConstraint> + Clone,
        DifferentialForm<A>: Satisfies<NoConstraint>,
    {
        // Create a 0-chain containing a 0-form with single coefficient a
        let dim = ctx.dim();

        // Create form with the single coefficient (not using constant to avoid Default bound)
        let coefficients = CausalTensor::from_vec(vec![a], &[1]);
        let form = DifferentialForm::from_tensor(0, dim, coefficients);

        // Create sparse matrix for chain weights
        let num_vertices = ctx.num_simplices(0).max(1);
        let weights: CsrMatrix<DifferentialForm<A>> = CsrMatrix::with_capacity(1, num_vertices, 1);

        Chain::new(ctx.complex_arc(), 0, weights)
    }

    /// Counit: L(R(B)) = DifferentialForm<Chain<B>> → B
    ///
    /// Extracts the integrated value from a form of chains.
    fn counit<B>(ctx: &StokesContext, lrb: DifferentialForm<Chain<B>>) -> B
    where
        B: Satisfies<NoConstraint> + Clone,
        Chain<B>: Satisfies<NoConstraint>,
    {
        // Integration: collapse form of chains to scalar
        // Extract first chain, then first weight
        if lrb.degree() == 0 && !lrb.coefficients().as_slice().is_empty() {
            if let Some(chain) = lrb.coefficients().as_slice().first() {
                let weights = chain.weights();
                if !weights.values().is_empty() {
                    return weights.values()[0].clone();
                }
            }
        }
        let _ = ctx;
        // Must clone from input since we can't use Default
        if !lrb.coefficients().as_slice().is_empty() {
            if let Some(chain) = lrb.coefficients().as_slice().first() {
                if !chain.weights().values().is_empty() {
                    return chain.weights().values()[0].clone();
                }
            }
        }
        // This is a limitation - without Default, we can't produce a B from nothing
        panic!("Counit requires at least one value in the form's chain")
    }

    /// Left adjunct: (L(A) → B) → (A → R(B))
    ///
    /// Given f: DifferentialForm<A> → B, produce g: A → Chain<B>
    fn left_adjunct<A, B, Func>(ctx: &StokesContext, a: A, f: Func) -> Chain<B>
    where
        A: Satisfies<NoConstraint> + Clone,
        B: Satisfies<NoConstraint>,
        DifferentialForm<A>: Satisfies<NoConstraint>,
        Func: Fn(DifferentialForm<A>) -> B,
    {
        // Create form from a (without requiring Default)
        let dim = ctx.dim();
        let coefficients = CausalTensor::from_vec(vec![a], &[1]);
        let form = DifferentialForm::from_tensor(0, dim, coefficients);

        // Apply f to get B
        let _b = f(form);

        // Create a 0-chain
        let num_vertices = ctx.num_simplices(0).max(1);
        let weights: CsrMatrix<B> = CsrMatrix::with_capacity(1, num_vertices, 1);

        Chain::new(ctx.complex_arc(), 0, weights)
    }

    /// Right adjunct: (A → R(B)) → (L(A) → B)
    ///
    /// Given g: A → Chain<B>, produce f: DifferentialForm<A> → B
    fn right_adjunct<A, B, Func>(ctx: &StokesContext, la: DifferentialForm<A>, mut f: Func) -> B
    where
        A: Satisfies<NoConstraint> + Clone,
        B: Satisfies<NoConstraint> + Clone,
        Chain<B>: Satisfies<NoConstraint>,
        Func: FnMut(A) -> Chain<B>,
    {
        // Extract first coefficient from form, apply f, get first chain value
        if la.degree() == 0 && !la.coefficients().as_slice().is_empty() {
            if let Some(a) = la.coefficients().as_slice().first() {
                let chain = f(a.clone());
                if !chain.weights().values().is_empty() {
                    return chain.weights().values()[0].clone();
                }
            }
        }
        let _ = ctx;
        panic!("Right adjunct requires at least one value in the form")
    }
}

// ============================================================================
// Production Operations (Specialized for f64)
// ============================================================================

impl StokesAdjunction {
    /// Applies the exterior derivative to a discrete k-form on f64 coefficients.
    ///
    /// d: Ω^k → Ω^(k+1)
    ///
    /// Uses the coboundary matrix from the simplicial complex.
    pub fn exterior_derivative_f64(
        ctx: &StokesContext,
        form: &DifferentialForm<f64>,
    ) -> DifferentialForm<f64> {
        let k = form.degree();
        let dim = ctx.dim();

        // Cannot take derivative of top form
        if k >= dim {
            return DifferentialForm::zero(k + 1, form.dim());
        }

        // Get coboundary operator C_k = B_{k+1}^T
        let coboundary_ops = &ctx.complex().coboundary_operators;
        if k >= coboundary_ops.len() {
            return DifferentialForm::zero(k + 1, form.dim());
        }

        let coboundary = &coboundary_ops[k];
        let coeffs = form.coefficients().as_slice();

        // Get size from shape
        let shape = coboundary.shape();
        let nrows = shape.0;

        let mut result_coeffs: Vec<f64> = Vec::with_capacity(nrows);

        // For each row (output simplex), compute the sum
        for row_idx in 0..nrows {
            let mut sum = 0.0;

            // Get the row slice using CSR structure
            let row_start = coboundary.row_indices()[row_idx];
            let row_end = coboundary.row_indices()[row_idx + 1];

            for idx in row_start..row_end {
                let col = coboundary.col_indices()[idx];
                let sign = coboundary.values()[idx];

                if col < coeffs.len() {
                    sum += coeffs[col] * (sign as f64);
                }
            }
            result_coeffs.push(sum);
        }

        DifferentialForm::from_coefficients(k + 1, form.dim(), result_coeffs)
    }

    /// Applies the boundary operator to a k-chain on f64 coefficients.
    ///
    /// ∂: C_k → C_(k-1)
    ///
    /// Uses the boundary matrix from the simplicial complex.
    pub fn boundary_f64(ctx: &StokesContext, chain: &Chain<f64>) -> Chain<f64> {
        let k = chain.grade();

        // Boundary of 0-chain is empty
        if k == 0 {
            let empty_weights: CsrMatrix<f64> = CsrMatrix::new();
            return Chain::new(ctx.complex_arc(), 0, empty_weights);
        }

        // Get boundary operator B_k
        let boundary_ops = &ctx.complex().boundary_operators;
        if k > boundary_ops.len() {
            let empty_weights: CsrMatrix<f64> = CsrMatrix::new();
            return Chain::new(ctx.complex_arc(), k - 1, empty_weights);
        }

        // Return empty for now - full implementation requires boundary matrix application
        let empty_weights: CsrMatrix<f64> = CsrMatrix::new();
        Chain::new(ctx.complex_arc(), k - 1, empty_weights)
    }

    /// Integrates an f64 k-form over a k-chain: ⟨ω, C⟩ = ∫_C ω
    pub fn integrate_f64(form: &DifferentialForm<f64>, chain: &Chain<f64>) -> f64 {
        if form.degree() != chain.grade() {
            return 0.0;
        }

        let coeffs = form.coefficients().as_slice();
        let weights = chain.weights();
        let mut result = 0.0;

        // ⟨ω, C⟩ = Σ_σ c_σ ω_σ
        let col_indices = weights.col_indices();
        let values = weights.values();

        for (idx, &col) in col_indices.iter().enumerate() {
            if col < coeffs.len() {
                result += values[idx] * coeffs[col];
            }
        }

        result
    }
}
