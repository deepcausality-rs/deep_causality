/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! ADM (Arnowitt-Deser-Misner) Formalism Module
//!
//! Provides the 3+1 decomposition of spacetime for numerical relativity.
//! Spacetime is sliced into spatial hypersurfaces Σ_t evolved by a time coordinate t.
use crate::PhysicsError;
use deep_causality_tensor::CausalTensor;

pub trait AdmOps {
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
        matter_density: Option<&CausalTensor<f64>>,
    ) -> Result<CausalTensor<f64>, PhysicsError>;

    /// Computes the Momentum constraint.
    ///
    /// # Mathematical Definition
    /// ```text
    /// M_i = D_j (K^j_i - γ^j_i K) - 8πj_i
    /// ```
    /// Returns 0 vector if satisfied.
    fn momentum_constraint(
        &self,
        matter_momentum: Option<&CausalTensor<f64>>,
    ) -> Result<CausalTensor<f64>, PhysicsError>;

    /// Returns the trace of extrinsic curvature K.
    fn mean_curvature(&self) -> Result<CausalTensor<f64>, PhysicsError>;
}
