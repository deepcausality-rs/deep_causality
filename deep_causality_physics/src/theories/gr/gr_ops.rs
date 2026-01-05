/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// =============================================================================
// GR Operations Trait
// =============================================================================

use crate::{NEWTONIAN_CONSTANT_OF_GRAVITATION, PhysicsError, SPEED_OF_LIGHT};
use deep_causality_tensor::CausalTensor;

/// Represents (Position, Velocity) state vector
pub type GeodesicState = (Vec<f64>, Vec<f64>);

/// Operations for General Relativity — SO(3,1) Lorentz Gauge Theory.
///
/// # Mathematical Foundation
///
/// GR is formulated as a gauge theory with the Lorentz group:
///
/// ## Connection (Christoffel Symbols)
/// ```text
/// Γ^ρ_μν = ½ g^ρσ (∂_μ g_νσ + ∂_ν g_μσ - ∂_σ g_μν)
/// ```
///
/// ## Curvature (Riemann Tensor)
/// ```text
/// R^ρ_σμν = ∂_μ Γ^ρ_νσ - ∂_ν Γ^ρ_μσ + Γ^ρ_μλ Γ^λ_νσ - Γ^ρ_νλ Γ^λ_μσ
/// ```
/// Computed via `GaugeFieldWitness::compute_field_strength_non_abelian`.
///
/// ## Geodesic Deviation
/// ```text
/// D²ξ^μ/Dτ² = R^μ_νρσ u^ν ξ^ρ u^σ
/// ```
/// Computed via `CurvatureTensorWitness::curvature` (RiemannMap trait).
pub trait GrOps {
    // -------------------------------------------------------------------------
    // Curvature Invariants
    // -------------------------------------------------------------------------

    /// Computes the Ricci tensor R_μν by contraction of Riemann.
    ///
    /// # Mathematical Definition
    /// ```text
    /// R_μν = R^ρ_μρν = g^ρσ R_ρμσν
    /// ```
    fn ricci_tensor(&self) -> Result<CausalTensor<f64>, PhysicsError>;

    /// Computes the Ricci scalar R (scalar curvature).
    ///
    /// # Mathematical Definition
    /// ```text
    /// R = g^μν R_μν
    /// ```
    fn ricci_scalar(&self) -> Result<f64, PhysicsError>;

    /// Computes the Einstein tensor G_μν.
    ///
    /// # Mathematical Definition
    /// ```text
    /// G_μν = R_μν - ½ R g_μν
    /// ```
    /// Uses `einstein_tensor_kernel`.
    fn einstein_tensor(&self) -> Result<CausalTensor<f64>, PhysicsError>;

    /// Computes the Kretschmann scalar K.
    ///
    /// # Mathematical Definition
    /// ```text
    /// K = R_μνρσ R^μνρσ
    /// ```
    /// For Schwarzschild: K = 48M²/r⁶
    fn kretschmann_scalar(&self) -> Result<f64, PhysicsError>;

    // -------------------------------------------------------------------------
    // Geodesic Motion
    // -------------------------------------------------------------------------

    /// Computes geodesic deviation (tidal acceleration).
    ///
    /// # Mathematical Definition
    /// ```text
    /// D²ξ^μ/Dτ² = R^μ_νρσ u^ν ξ^ρ u^σ
    /// ```
    /// Uses `CurvatureTensorWitness::curvature` via RiemannMap.
    fn geodesic_deviation(
        &self,
        velocity: &[f64],
        separation: &[f64],
    ) -> Result<Vec<f64>, PhysicsError>;

    /// Integrates the geodesic equation numerically.
    ///
    /// # Mathematical Definition
    /// ```text
    /// d²x^μ/dτ² + Γ^μ_νρ (dx^ν/dτ)(dx^ρ/dτ) = 0
    /// ```
    /// Uses `geodesic_integrator_kernel` (RK4).
    fn solve_geodesic(
        &self,
        initial_position: &[f64],
        initial_velocity: &[f64],
        proper_time_step: f64,
        num_steps: usize,
    ) -> Result<Vec<GeodesicState>, PhysicsError>;

    /// Computes proper time along a worldline.
    ///
    /// # Mathematical Definition
    /// ```text
    /// τ = ∫ √(-g_μν dx^μ dx^ν)
    /// ```
    /// Uses `proper_time_kernel`.
    fn proper_time(&self, path: &[Vec<f64>]) -> Result<f64, PhysicsError>;

    /// Parallel transports a vector along a path.
    ///
    /// # Mathematical Definition
    /// ```text
    /// Dv^μ/dλ = dv^μ/dλ + Γ^μ_νρ (dx^ν/dλ) v^ρ = 0
    /// ```
    /// Uses `parallel_transport_kernel`.
    fn parallel_transport(
        &self,
        initial_vector: &[f64],
        path: &[Vec<f64>],
    ) -> Result<Vec<f64>, PhysicsError>;

    // -------------------------------------------------------------------------
    // Metric Utilities
    // -------------------------------------------------------------------------

    /// Returns the metric tensor g_μν.
    fn metric_tensor(&self) -> &CausalTensor<f64>;

    /// Computes the Schwarzschild radius for a given mass.
    ///
    /// # Mathematical Definition
    /// ```text
    /// r_s = 2GM/c²
    /// ```
    fn schwarzschild_radius(mass_kg: f64) -> f64 {
        2.0 * NEWTONIAN_CONSTANT_OF_GRAVITATION * mass_kg / (SPEED_OF_LIGHT * SPEED_OF_LIGHT)
    }

    /// Computes the Riemann tensor from Christoffel symbols using the HKT witness.
    ///
    /// # Mathematical Definition
    ///
    /// Uses `GaugeFieldWitness::compute_field_strength_non_abelian` to compute:
    /// ```text
    /// R^ρ_σμν = ∂_μ Γ^ρ_νσ - ∂_ν Γ^ρ_μσ + Γ^ρ_μλ Γ^λ_νσ - Γ^ρ_νλ Γ^λ_μσ
    /// ```
    ///
    /// # Prerequisites
    ///
    /// For this method to produce meaningful results, the GR field's `connection` slot
    /// must contain Christoffel symbols (Γ^ρ_μν) rather than the metric tensor.
    ///
    /// # Returns
    ///
    /// The Riemann tensor in Lie-algebra storage form `[N, 4, 4, 6]`.
    fn compute_riemann_from_christoffel(&self) -> CausalTensor<f64>;

    /// Computes the ADM momentum constraint across all manifold points.
    ///
    /// # Mathematical Definition
    ///
    /// The momentum constraint in the 3+1 formalism is:
    /// ```text
    /// M_i = D_j (K^j_i - δ^j_i K) - 8πj_i
    /// ```
    ///
    /// where:
    /// - D_j is the covariant derivative on the spatial slice
    /// - K^j_i is the extrinsic curvature (mixed indices)
    /// - K = K^j_j is the trace (mean curvature)
    /// - j_i is the matter momentum density
    ///
    /// # Implementation
    ///
    /// Uses the **StokesAdjunction HKT** infrastructure:
    /// 1. The divergence D_j T^j_i is computed via the adjoint relationship d ⊣ ∂
    /// 2. Uses `StokesAdjunction::exterior_derivative` for the derivative terms
    /// 3. Adds Christoffel connection terms from the metric
    ///
    /// # Arguments
    ///
    /// * `extrinsic_curvature` - K_ij tensor at all manifold points, shape `[N, 3, 3]`
    /// * `matter_momentum` - Optional momentum density j_i, shape `[N, 3]`
    ///
    /// # Returns
    ///
    /// Momentum constraint M_i at all manifold points, shape `[N, 3]`.
    fn momentum_constraint_field(
        &self,
        extrinsic_curvature: &CausalTensor<f64>,
        matter_momentum: Option<&CausalTensor<f64>>,
    ) -> Result<CausalTensor<f64>, PhysicsError>;
}
