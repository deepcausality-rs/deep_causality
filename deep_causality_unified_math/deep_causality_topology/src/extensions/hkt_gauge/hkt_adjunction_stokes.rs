/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Stokes Adjunction (d ⊣ ∂) for differential geometry.
//!
//! This module provides the adjunction between the exterior derivative (d)
//! and the boundary operator (∂), which encodes Stokes' theorem:
//! ⟨dω, C⟩ = ⟨ω, ∂C⟩
//!
//! This is the foundation for conservation laws and integration theory.

use crate::errors::topology_error::{TopologyError, TopologyErrorEnum};
use crate::types::chain::Chain;
use crate::types::differential_form::DifferentialForm;
use crate::{BaseTopology, SimplicialComplex};
use deep_causality_haft::Pure; // Added Pure
use deep_causality_haft::{Adjunction, HKT};
use deep_causality_linear::CsrMatrix;
use deep_causality_linear::CsrMatrixWitness; // Added Witness
use deep_causality_num::Float;
use std::collections::HashMap;
use std::sync::Arc;

/// Witness for the exterior derivative d: Ω^k → Ω^(k+1).
#[derive(Debug, Clone, Copy, Default)]
/// # Why `NoConstraint`
///
/// `DifferentialForm<T>` carries no element bound, and the categorical operations here move elements without
/// computing on them: `fmap` maps `A` to an unrelated `B`. Constraining the element type would
/// forbid mappings that are legitimate and work today, so `NoConstraint` is the accurate statement
/// rather than a placeholder. Operations that compute carry real trait bounds on the concrete
/// types. See `openspec/notes/archive/hkt_gat/hkt_gat_topology.md` §4.
pub struct ExteriorDerivativeWitness;

impl HKT for ExteriorDerivativeWitness {
    type Type<T> = DifferentialForm<T>;
}

/// Witness for the boundary operator ∂: C_k → C_(k-1).
#[derive(Debug, Clone, Copy, Default)]
/// # Why `NoConstraint`
///
/// `Chain<R, G>` carries no bound on its coefficient group `G`, and the categorical operations here move elements without
/// computing on them: `fmap` maps `A` to an unrelated `B`. Constraining the element type would
/// forbid mappings that are legitimate and work today, so `NoConstraint` is the accurate statement
/// rather than a placeholder. Operations that compute carry real trait bounds on the concrete
/// types. See `openspec/notes/archive/hkt_gat/hkt_gat_topology.md` §4.
///
/// # `fmap` preserves the complex
///
/// The complex is indexed by the precision parameter, which mapping the coefficients does not
/// touch, so it is carried across and its Hodge ⋆ operators survive. This used to be false: a
/// single parameter served both roles, `fmap` had to rebuild the complex with
/// `..Default::default()`, and the functor identity law failed for any complex carrying geometry.
pub struct BoundaryWitness<R>(core::marker::PhantomData<R>);

impl<R> HKT for BoundaryWitness<R> {
    type Type<G> = Chain<R, G>;
}

/// Context for Stokes theorem operations.
#[derive(Debug, Clone)]
pub struct StokesContext<R> {
    /// The simplicial complex defining the discrete topology.
    complex: Arc<SimplicialComplex<R>>,
}

impl<R> StokesContext<R> {
    /// Creates a new Stokes context from a simplicial complex.
    pub fn new(complex: SimplicialComplex<R>) -> Self {
        Self {
            complex: Arc::new(complex),
        }
    }

    /// Creates a new Stokes context from an Arc'd simplicial complex.
    pub fn from_arc(complex: Arc<SimplicialComplex<R>>) -> Self {
        Self { complex }
    }

    /// Returns a reference to the underlying simplicial complex.
    pub fn complex(&self) -> &SimplicialComplex<R> {
        &self.complex
    }

    /// Returns the Arc to the simplicial complex.
    pub fn complex_arc(&self) -> Arc<SimplicialComplex<R>> {
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
#[derive(Debug, Clone, Copy, Default)]
pub struct StokesAdjunction;

impl<R> Adjunction<ExteriorDerivativeWitness, BoundaryWitness<R>, StokesContext<R>>
    for StokesAdjunction
{
    type Error = TopologyError;

    /// Unit: `A → R(L(A)) = Chain<DifferentialForm<A>>`
    ///
    /// Embeds a coefficient into a chain of forms.
    /// Semantically, this maps a value `a` to a 0-chain where each vertex has the 0-form `a`.
    fn unit<A>(ctx: &StokesContext<R>, a: A) -> Chain<R, DifferentialForm<A>>
    where
        A: Clone,
    {
        // Dimension of the complex
        let dim = ctx.dim();

        // Create a 0-form with the single coefficient 'a'.
        // This represents a constant scalar field with value 'a' at a single point.
        let coefficients = vec![a];
        let form = DifferentialForm::from_coefficients(0, dim, coefficients);

        // The chain's complex is indexed by the precision `R`, which the coefficient type does not
        // touch, so the context's complex carries across whole and its geometry survives.
        let inner_weights = <CsrMatrixWitness as Pure<CsrMatrixWitness>>::pure(form);

        // Chain grade 0
        Chain::new(ctx.complex_arc(), 0, inner_weights)
    }

    /// Counit: `L(R(B)) = DifferentialForm<Chain<B>> → B`
    ///
    /// Extracts the integrated value from a form of chains.
    /// # Errors
    ///
    /// Returns [`TopologyError`] when the form carries no coefficient, or when the chain it holds
    /// stores no weight, so there is no `B` to integrate to.
    fn counit<B>(
        _ctx: &StokesContext<R>,
        lrb: DifferentialForm<Chain<R, B>>,
    ) -> Result<B, Self::Error>
    where
        B: Clone,
    {
        // Integration: collapse form of chains to scalar (B).
        // The counit evaluation doesn't strictly depend on the topological context
        // if we assume the form and chain already encode the necessary structure.
        //
        // The form is non-empty by construction, but that is an invariant of the constructors
        // rather than of the type, so it is checked rather than indexed.
        let chain = lrb.coefficients().as_slice().first().ok_or_else(|| {
            TopologyError(TopologyErrorEnum::InvalidInput(
                "Adjunction::counit: the differential form carries no coefficient, so there is \
                 no chain to integrate"
                    .into(),
            ))
        })?;

        chain.weights().values().first().cloned().ok_or_else(|| {
            TopologyError(TopologyErrorEnum::InvalidInput(
                "Adjunction::counit: the form's chain stores no weight, so there is no B to \
                 evaluate to"
                    .into(),
            ))
        })
    }

    /// Left adjunct: (L(A) → B) → (A → R(B))
    ///
    /// Given `f: DifferentialForm<A> → B`, produce `g: A → Chain<B>`
    fn left_adjunct<A, B, Func>(ctx: &StokesContext<R>, a: A, f: Func) -> Chain<R, B>
    where
        A: Clone,
        Func: Fn(DifferentialForm<A>) -> B,
    {
        // 1. Create a representative 0-form from 'a'
        let dim = ctx.dim();
        let form = DifferentialForm::from_coefficients(0, dim, vec![a]);

        // 2. Apply the morphism f to get the result in B
        let b = f(form);

        // 3. Wrap result 'b' into a 0-chain. The chain's complex is indexed by the precision `R`,
        //    which `B` does not touch, so the context's complex carries across with its geometry.
        let weights = <CsrMatrixWitness as Pure<CsrMatrixWitness>>::pure(b);

        Chain::new(ctx.complex_arc(), 0, weights)
    }

    /// Right adjunct: (A → R(B)) → (L(A) → B)
    ///
    /// Given `g: A → Chain<B>`, produce `f: DifferentialForm<A> → B`
    /// # Errors
    ///
    /// Returns [`TopologyError`] when the form carries no coefficient, so there is no `A` to apply
    /// `f` to, or when the chain `f` returns stores no weight.
    fn right_adjunct<A, B, Func>(
        _ctx: &StokesContext<R>,
        la: DifferentialForm<A>,
        mut f: Func,
    ) -> Result<B, Self::Error>
    where
        A: Clone,
        B: Clone,
        Func: FnMut(A) -> Chain<R, B>,
    {
        // Extract value 'a' from the form 'la'. Non-empty by construction, but that is a
        // constructor invariant rather than a type-level one, so it is checked.
        let a = la.coefficients().as_slice().first().ok_or_else(|| {
            TopologyError(TopologyErrorEnum::InvalidInput(
                "Adjunction::right_adjunct: the differential form carries no coefficient, so \
                 there is no A to apply f to"
                    .into(),
            ))
        })?;

        // Apply morphism g (here 'f') to get Chain<B>
        let chain = f(a.clone());

        chain.weights().values().first().cloned().ok_or_else(|| {
            TopologyError(TopologyErrorEnum::InvalidInput(
                "Adjunction::right_adjunct: f returned a Chain that stores no weight, so there \
                 is no B to return"
                    .into(),
            ))
        })
    }
}

// ============================================================================
// Production Operations (Generic over Float)
// ============================================================================

impl StokesAdjunction {
    /// Applies the exterior derivative to a discrete k-form.
    ///
    /// d: Ω^k → Ω^(k+1)
    ///
    /// Uses the coboundary matrix from the simplicial complex.
    pub fn exterior_derivative<T>(
        ctx: &StokesContext<T>,
        form: &DifferentialForm<T>,
    ) -> DifferentialForm<T>
    where
        T: Float + Default + From<f64>,
    {
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

        let mut result_coeffs: Vec<T> = Vec::with_capacity(nrows);

        // For each row (output simplex), compute the sum
        for row_idx in 0..nrows {
            let mut sum = T::zero();

            // Get the row slice using CSR structure
            let row_start = coboundary.row_indices()[row_idx];
            let row_end = coboundary.row_indices()[row_idx + 1];

            for idx in row_start..row_end {
                let col = coboundary.col_indices()[idx];
                let sign = coboundary.values()[idx]; // i8, -1 or 1

                if col < coeffs.len() {
                    // Convert sign to T
                    let sign_t = if sign > 0 { T::one() } else { -T::one() };
                    // Float usually implies Add, Mul. Summing requires explicit Add.
                    // But Float implies Num implies Add.
                    sum += coeffs[col] * sign_t;
                }
            }
            result_coeffs.push(sum);
        }

        DifferentialForm::from_coefficients(k + 1, form.dim(), result_coeffs)
    }

    /// Applies the boundary operator to a k-chain.
    ///
    /// ∂: C_k → C_(k-1)
    ///
    /// Uses the boundary matrix from the simplicial complex.
    pub fn boundary<R, G>(ctx: &StokesContext<R>, chain: &Chain<R, G>) -> Chain<R, G>
    where
        G: Float + Default,
    {
        let k = chain.grade();

        // Boundary of 0-chain is empty
        if k == 0 {
            let empty_weights: CsrMatrix<G> = CsrMatrix::new();
            return Chain::new(ctx.complex_arc(), 0, empty_weights);
        }

        // Get boundary operator B_k: C_k -> C_{k-1}
        let boundary_ops = &ctx.complex().boundary_operators;
        if k > boundary_ops.len() {
            let empty_weights: CsrMatrix<G> = CsrMatrix::new();
            return Chain::new(ctx.complex_arc(), k - 1, empty_weights);
        }

        // `boundary_operators[i]` holds the operator that maps `C_{i+1} -> C_i`, so the operator
        // for a k-chain is at index `k - 1`. This is the convention `boundary_operator_impl`
        // states: it returns `boundary_operators[k - 1]` and rejects `k == 0` outright. Reading
        // index `k` here applied the *next* operator up, which for a 1-chain on a triangle meant
        // dotting the chain against the triangle-to-edge matrix instead of the edge-to-vertex one.
        let boundary_op = &boundary_ops[k - 1];
        let shape = boundary_op.shape();
        let nrows = shape.0; // num_(k-1)_simplices

        let chain_weights = chain.weights();
        let row_indices = chain_weights.row_indices();
        let col_indices = chain_weights.col_indices();
        let values = chain_weights.values();

        if row_indices.is_empty() {
            let empty_weights: CsrMatrix<G> = CsrMatrix::default();
            return Chain::new(ctx.complex_arc(), k - 1, empty_weights);
        }

        // Optimization: Collect chain weights into a HashMap for O(1) lookups
        let chain_map: HashMap<usize, G> = col_indices
            .iter()
            .zip(values.iter())
            .map(|(&c, v)| (c, *v))
            .collect();

        let mut result_triplets: Vec<(usize, usize, G)> = Vec::new();

        // Iterate over each row of the boundary matrix (each (k-1)-simplex)
        for row_idx in 0..nrows {
            let mut sum = G::zero();
            let row_start = boundary_op.row_indices()[row_idx];
            let row_end = boundary_op.row_indices()[row_idx + 1];

            // Dot product: row(B) . v
            for idx in row_start..row_end {
                let col = boundary_op.col_indices()[idx]; // index of k-simplex (j)
                let sign = boundary_op.values()[idx]; // B_ij (orientation)

                if let Some(val) = chain_map.get(&col) {
                    let sign_t = if sign > 0 { G::one() } else { -G::one() };
                    sum += *val * sign_t;
                }
            }

            if sum != G::zero() {
                result_triplets.push((0, row_idx, sum));
            }
        }

        let num_k_minus_1_simplices = ctx.num_simplices(k - 1);
        let result_matrix = CsrMatrix::from_triplets(1, num_k_minus_1_simplices, &result_triplets)
            .unwrap_or_else(|_| CsrMatrix::new());

        Chain::new(ctx.complex_arc(), k - 1, result_matrix)
    }

    /// Integrates a k-form over a k-chain: ⟨ω, C⟩ = ∫_C ω
    pub fn integrate<R, G>(form: &DifferentialForm<G>, chain: &Chain<R, G>) -> G
    where
        G: Float + Default,
    {
        if form.degree() != chain.grade() {
            return G::zero();
        }

        let coeffs = form.coefficients().as_slice();
        let weights = chain.weights();
        let mut result = G::zero();

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
