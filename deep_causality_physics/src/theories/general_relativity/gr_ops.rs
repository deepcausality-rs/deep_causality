/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// =============================================================================
// GR Operations Trait
// =============================================================================

// =============================================================================
// GR Operations Trait
// =============================================================================

use crate::{NEWTONIAN_CONSTANT_OF_GRAVITATION, PhysicsError, SPEED_OF_LIGHT};
use deep_causality_num::{Field, Float};
use deep_causality_tensor::CausalTensor;

/// Represents (Position, Velocity) state vector
pub type GeodesicState<S> = (Vec<S>, Vec<S>);

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
pub trait GrOps<S>
where
    S: Field + Float + Clone + From<f64> + Into<f64>,
{
    // -------------------------------------------------------------------------
    // Curvature Invariants
    // -------------------------------------------------------------------------

    /// Computes the Ricci tensor R_μν by contraction of Riemann.
    ///
    /// # Mathematical Definition
    /// ```text
    /// R_μν = R^ρ_μρν = g^ρσ R_ρμσν
    /// ```
    fn ricci_tensor(&self) -> Result<CausalTensor<S>, PhysicsError>;

    /// Computes the Ricci scalar R (scalar curvature).
    ///
    /// # Mathematical Definition
    /// ```text
    /// R = g^μν R_μν
    /// ```
    fn ricci_scalar(&self) -> Result<S, PhysicsError>;

    /// Computes the Einstein tensor G_μν.
    ///
    /// # Mathematical Definition
    /// ```text
    /// G_μν = R_μν - ½ R g_μν
    /// ```
    /// Uses `einstein_tensor_kernel`.
    fn einstein_tensor(&self) -> Result<CausalTensor<S>, PhysicsError>;

    /// Computes the Kretschmann scalar K in **geometric units**.
    ///
    /// # Mathematical Definition
    /// ```text
    /// K = R_μνρσ R^μνρσ
    /// ```
    /// For Schwarzschild: K = 48M²/r⁶
    ///
    /// # Units
    /// Returns scalar curvature in **geometric units** (`m⁻⁴`).
    /// For curvature radius in meters, use [`Self::kretschmann_curvature_radius`].
    fn kretschmann_scalar(&self) -> Result<S, PhysicsError>;

    /// Computes the curvature radius from Kretschmann scalar in **SI units**.
    ///
    /// # Mathematical Definition
    /// ```text
    /// R_curv = K^(-1/4)
    /// ```
    ///
    /// # Units
    /// Returns curvature radius in **meters**.
    ///
    /// # Conversion
    /// Takes the 4th root of 1/K to convert from `m⁻⁴` to `m`.
    fn kretschmann_curvature_radius(&self) -> Result<S, PhysicsError> {
        let k = self.kretschmann_scalar()?;
        if k <= S::zero() {
            return Ok(S::infinity()); // Flat spacetime
        }
        // 1.0 / k.powf(0.25)
        let quart = <S as From<f64>>::from(0.25);
        Ok(S::one() / k.powf(quart))
    }

    // -------------------------------------------------------------------------
    // Geodesic Motion
    // -------------------------------------------------------------------------

    /// Computes geodesic deviation (tidal acceleration) in **geometric units**.
    ///
    /// # Mathematical Definition
    /// ```text
    /// D²ξ^μ/Dτ² = R^μ_νρσ u^ν ξ^ρ u^σ
    /// ```
    /// Uses `CurvatureTensorWitness::curvature` via RiemannMap.
    ///
    /// # Units
    /// Returns acceleration in **geometric units** (`m⁻²`).
    /// For SI units (`m/s²`), use [`Self::geodesic_deviation_si`].
    fn geodesic_deviation(&self, velocity: &[S], separation: &[S]) -> Result<Vec<S>, PhysicsError>;

    /// Computes geodesic deviation (tidal acceleration) in **SI units**.
    ///
    /// # Mathematical Definition
    /// ```text
    /// a^μ = c² × R^μ_νρσ u^ν ξ^ρ u^σ
    /// ```
    ///
    /// # Units
    /// Returns acceleration in **SI units** (`m/s²`).
    ///
    /// # Conversion
    /// Multiplies geometric result by `c² ≈ 8.99 × 10¹⁶ m²/s²`.
    fn geodesic_deviation_si(
        &self,
        velocity: &[S],
        separation: &[S],
    ) -> Result<Vec<S>, PhysicsError> {
        let geometric = self.geodesic_deviation(velocity, separation)?;
        let c = <S as From<f64>>::from(SPEED_OF_LIGHT);
        let c2 = c * c;
        Ok(geometric.into_iter().map(|v| v * c2).collect())
    }

    /// Integrates the geodesic equation numerically.
    ///
    /// # Mathematical Definition
    /// ```text
    /// d²x^μ/dτ² + Γ^μ_νρ (dx^ν/dτ)(dx^ρ/dτ) = 0
    /// ```
    /// Uses `geodesic_integrator_kernel` (RK4).
    fn solve_geodesic(
        &self,
        initial_position: &[S],
        initial_velocity: &[S],
        proper_time_step: S,
        num_steps: usize,
    ) -> Result<Vec<GeodesicState<S>>, PhysicsError>;

    /// Computes proper time along a worldline in **geometric units**.
    ///
    /// # Mathematical Definition
    /// ```text
    /// τ = ∫ √(-g_μν dx^μ dx^ν)
    /// ```
    /// Uses `proper_time_kernel`.
    ///
    /// # Units
    /// Returns proper time in **geometric units** (meters).
    /// For SI units (seconds), use [`Self::proper_time_si`].
    fn proper_time(&self, path: &[Vec<S>]) -> Result<S, PhysicsError>;

    /// Computes proper time along a worldline in **SI units**.
    ///
    /// # Units
    /// Returns proper time in **seconds**.
    ///
    /// # Conversion
    /// Divides geometric result by `c ≈ 2.998 × 10⁸ m/s`.
    fn proper_time_si(&self, path: &[Vec<S>]) -> Result<S, PhysicsError> {
        let geometric = self.proper_time(path)?;
        let c = <S as From<f64>>::from(SPEED_OF_LIGHT);
        Ok(geometric / c)
    }

    /// Parallel transports a vector along a path.
    ///
    /// # Mathematical Definition
    /// ```text
    /// Dv^μ/dλ = dv^μ/dλ + Γ^μ_νρ (dx^ν/dλ) v^ρ = 0
    /// ```
    /// Uses `parallel_transport_kernel`.
    fn parallel_transport(
        &self,
        initial_vector: &[S],
        path: &[Vec<S>],
    ) -> Result<Vec<S>, PhysicsError>;

    // -------------------------------------------------------------------------
    // Metric Utilities
    // -------------------------------------------------------------------------

    /// Returns the metric tensor g_μν.
    fn metric_tensor(&self) -> &CausalTensor<S>;

    /// Computes the Schwarzschild radius for a given mass.
    ///
    /// # Mathematical Definition
    /// ```text
    /// r_s = 2GM/c²
    /// ```
    fn schwarzschild_radius(mass_kg: S) -> S {
        let two = <S as From<f64>>::from(2.0);
        let g = <S as From<f64>>::from(NEWTONIAN_CONSTANT_OF_GRAVITATION);
        let c = <S as From<f64>>::from(SPEED_OF_LIGHT);
        (two * g * mass_kg) / (c * c)
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
    fn compute_riemann_from_christoffel(&self) -> CausalTensor<S>;

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
        extrinsic_curvature: &CausalTensor<S>,
        matter_momentum: Option<&CausalTensor<S>>,
    ) -> Result<CausalTensor<S>, PhysicsError>;
}
