/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::PhysicsError;
use deep_causality_multivector::CausalMultiVector;
use deep_causality_num::{Field, Float};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::Manifold;

/// Operations for Gauge-Theoretic Electromagnetism — U(1) Gauge Field Theory.
///
/// This trait provides classical electromagnetic operations using the
/// relativistic gauge field formalism. It computes E, B fields, energy density,
/// Lorentz invariants, and radiation properties from the U(1) field strength tensor.
///
/// # Type Parameter
/// - `S`: The scalar field type (e.g., `f64`, `DoubleFloat`)
///
/// # Mathematical Foundation
///
/// The electromagnetic field is represented as a U(1) gauge field:
///
/// ## Gauge Potential (Connection 1-form)
/// ```text
/// A = A_μ dx^μ = (φ, A_x, A_y, A_z)
/// ```
/// where φ is the scalar potential and (A_x, A_y, A_z) is the vector potential.
///
/// ## Field Strength Tensor (Curvature 2-form)
/// ```text
/// F_μν = ∂_μ A_ν - ∂_ν A_μ
/// ```
/// In matrix form (West Coast +--- signature):
/// ```text
///        ⎛  0    E_x   E_y   E_z ⎞
/// F_μν = ⎜-E_x   0    -B_z   B_y ⎟
///        ⎜-E_y  B_z    0    -B_x ⎟
///        ⎝-E_z -B_y   B_x    0   ⎠
/// ```
///
/// ## Metric Convention
/// Uses West Coast signature (+---) following particle physics conventions.
pub trait GaugeEmOps<S>
where
    S: Field + Float + Clone + From<f64> + Into<f64>,
{
    /// Creates a new QED field from electric and magnetic field vectors.
    ///
    /// # Mathematical Structure
    ///
    /// Constructs F_μν from E and B fields:
    /// - F_{0i} = E_i (electric field components)
    /// - F_{ij} = -ε_{ijk} B_k (magnetic field components)
    fn from_fields(
        base: Manifold<S>,
        electric_field: CausalMultiVector<S>,
        magnetic_field: CausalMultiVector<S>,
    ) -> Result<Self, PhysicsError>
    where
        Self: Sized;

    /// Creates a QED field from field components in 3D Euclidean space.
    fn from_components(ex: S, ey: S, ez: S, bx: S, by: S, bz: S) -> Result<Self, PhysicsError>
    where
        Self: Sized;

    /// Creates a QED field for a plane wave with orthogonal E and B.
    ///
    /// # Mathematical Form
    ///
    /// For a plane wave propagating in direction k̂:
    /// - E ⊥ B ⊥ k̂
    /// - |E| = |B| (in natural units where c = 1)
    fn plane_wave(amplitude: S, polarization: usize) -> Result<Self, PhysicsError>
    where
        Self: Sized;

    /// Extracts the electric field vector E from the field tensor F_μν.
    ///
    /// # Mathematical Definition
    /// ```text
    /// E_i = F_{0i}   for i = 1,2,3
    /// ```
    fn electric_field(&self) -> Result<CausalMultiVector<S>, PhysicsError>;

    /// Extracts the magnetic field vector B from the field tensor F_μν.
    ///
    /// # Mathematical Definition
    /// ```text
    /// B_i = ½ ε_{ijk} F^{jk}
    /// ```
    /// Equivalently: B_x = F_{23}, B_y = F_{31}, B_z = F_{12}
    fn magnetic_field(&self) -> Result<CausalMultiVector<S>, PhysicsError>;

    /// Computes the electromagnetic energy density.
    ///
    /// # Mathematical Definition
    /// ```text
    /// u = ½(ε₀|E|² + |B|²/μ₀) = ½(|E|² + |B|²)  [natural units]
    /// ```
    fn energy_density(&self) -> Result<S, PhysicsError>;

    /// Computes the Lagrangian density.
    ///
    /// # Mathematical Definition
    /// ```text
    /// L = -¼ F_μν F^μν = ½(|E|² - |B|²)
    /// ```
    fn lagrangian_density(&self) -> Result<S, PhysicsError>;

    /// Computes the Poynting vector (energy flux).
    ///
    /// # Mathematical Definition
    /// ```text
    /// S = (1/μ₀)(E × B) = E × B  [natural units]
    /// ```
    fn poynting_vector(&self) -> Result<CausalMultiVector<S>, PhysicsError>;

    /// Computes the Lorentz force density on a current.
    ///
    /// # Mathematical Definition
    /// ```text
    /// f^μ = F^μν J_ν = ρE + J × B
    /// ```
    /// where J is the current density 4-vector.
    fn lorentz_force(
        &self,
        current_density: &CausalMultiVector<S>,
    ) -> Result<CausalMultiVector<S>, PhysicsError>;

    /// Computes the first Lorentz invariant (field invariant).
    ///
    /// # Mathematical Definition
    /// ```text
    /// I₁ = F_μν F^μν = 2(|B|² - |E|²)
    /// ```
    /// This quantity is unchanged under Lorentz transformations.
    fn field_invariant(&self) -> Result<S, PhysicsError>;

    /// Computes the second Lorentz invariant (dual invariant).
    ///
    /// # Mathematical Definition
    /// ```text
    /// I₂ = F_μν F̃^μν = -4(E · B)
    /// ```
    /// where F̃^μν is the Hodge dual of F^μν.
    fn dual_invariant(&self) -> Result<S, PhysicsError>;

    /// Checks if the field is a radiation field (E ⊥ B).
    ///
    /// # Mathematical Condition
    /// ```text
    /// E · B = 0  ⟺  |I₂| < ε
    /// ```
    fn is_radiation_field(&self) -> Result<bool, PhysicsError>;

    /// Checks if the field is a null field (|E| = |B|).
    ///
    /// # Mathematical Condition
    /// ```text
    /// |E|² = |B|²  ⟺  I₁ ≈ 0
    /// ```
    fn is_null_field(&self) -> Result<bool, PhysicsError>;

    /// Computes the electromagnetic momentum density.
    ///
    /// # Mathematical Definition
    /// ```text
    /// g = S/c² = ε₀(E × B) = E × B  [natural units]
    /// ```
    /// Equal to the Poynting vector in natural units.
    fn momentum_density(&self) -> Result<CausalMultiVector<S>, PhysicsError>;

    /// Computes the electromagnetic intensity (|S|).
    ///
    /// # Mathematical Definition
    /// ```text
    /// I = |S| = |E × B|
    /// ```
    fn intensity(&self) -> Result<S, PhysicsError>;

    /// Computes the field strength tensor F_μν from the gauge potential A_μ.
    ///
    /// # Mathematical Definition (Single Source of Truth via GaugeFieldWitness)
    /// ```text
    /// F_μν = ∂_μ A_ν - ∂_ν A_μ
    /// ```
    /// Uses `GaugeFieldWitness::compute_field_strength_abelian()` as the
    /// canonical implementation for abelian gauge theories.
    ///
    /// # Returns
    /// Field strength tensor of shape [num_points, 4, 4, 1]
    fn computed_field_strength(&self) -> Result<CausalTensor<S>, PhysicsError>;
}
