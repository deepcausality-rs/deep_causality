/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! ADM (Arnowitt-Deser-Misner) Formalism Module
//!
//! Provides the 3+1 decomposition of spacetime for numerical relativity.
//! Spacetime is sliced into spatial hypersurfaces Σ_t evolved by a time coordinate t.
use crate::PhysicsError;
use deep_causality_algebra::Field;
use deep_causality_tensor::CausalTensor;

/// ADM Formalism operations, generic over scalar type `S`.
///
/// # Type Parameters
/// * `S` - Scalar type (e.g., `f32`, `f64`, `DoubleFloat`)
pub trait AdmOps<S>
where
    S: Field + Clone + From<f64> + Into<f64>,
{
    /// Computes the Hamiltonian constraint.
    ///
    /// # Mathematical Definition
    /// ```text
    /// H = R + K² - K_ij K^ij - 16πρ
    /// ```
    /// Returns 0 if constraint is satisfied (in vacuum).
    ///
    /// # Arguments
    /// * `matter_density` - Energy density ρ (default 0 for vacuum)
    fn hamiltonian_constraint(
        &self,
        matter_density: Option<&CausalTensor<S>>,
    ) -> Result<CausalTensor<S>, PhysicsError>;

    /// Computes the Momentum constraint.
    ///
    /// # Mathematical Definition
    /// ```text
    /// M_i = D_j (K^j_i - δ^j_i K) - 8πj_i
    /// ```
    /// Returns a 3-vector; should be zero when the constraint is satisfied.
    ///
    /// # Implementation
    ///
    /// This method requires spatial Christoffel symbols ^(3)Γ^k_ij to compute the
    /// covariant derivative D_j. Use `AdmState::with_christoffel()` to provide them.
    ///
    /// ## Why Spatial Christoffel Symbols?
    ///
    /// Two options were considered:
    /// 1. **Pre-computed Christoffel symbols** (chosen) — The caller provides Γ^k_ij
    /// 2. **Manifold integration** — Compute derivatives via finite differences on neighbors
    ///
    /// Option 1 was chosen because:
    /// - **Flexibility**: Works with any data source (analytic metrics, numerical grids, FEM meshes)
    /// - **Performance**: Avoids repeated neighbor lookups; Christoffel symbols are typically
    ///   already computed by numerical relativity codes
    /// - **Decoupling**: `AdmState` remains a simple data container without `Manifold` dependency
    /// - **Accuracy**: Caller can use high-order finite difference stencils or analytic formulas
    ///
    /// ## Current Limitation
    ///
    /// The implementation computes only the **Christoffel connection terms** (Γ-dependent parts).
    /// The **partial derivative terms** (∂_j T^j_i) require values at neighboring points, which
    /// are not available in the current point-wise `AdmState` structure. For a complete
    /// constraint evaluation, use this on a grid and add the finite-difference derivative.
    fn momentum_constraint(
        &self,
        matter_momentum: Option<&CausalTensor<S>>,
    ) -> Result<CausalTensor<S>, PhysicsError>;

    /// Returns the trace of extrinsic curvature K.
    fn mean_curvature(&self) -> Result<CausalTensor<S>, PhysicsError>;
}
