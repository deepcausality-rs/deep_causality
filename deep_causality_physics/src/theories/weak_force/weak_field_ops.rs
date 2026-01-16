/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// =============================================================================
// Weak Field Operations Trait
// =============================================================================

use crate::PhysicsError;
use deep_causality_num::{RealField, ToPrimitive};
use deep_causality_tensor::CausalTensor;

/// Operations for the Weak Force — SU(2)_L gauge theory.
///
/// # Mathematical Foundation
///
/// The weak force is a non-abelian SU(2)_L gauge theory. Key features:
///
/// ## Field Strength (Non-Abelian)
/// ```text
/// W_μν^a = ∂_μ W_ν^a - ∂_ν W_μ^a + g ε^{abc} W_μ^b W_ν^c
/// ```
///
/// ## Propagators
/// ```text
/// D_W(q²) = 1 / (q² - M_W²)           (W boson)
/// D_Z(q²) = (g_V² + g_A²) / (q² - M_Z²)  (Z boson)
/// ```
///
/// ## Decay Width (Fermi Theory)
/// ```text
/// Γ = G_F² m⁵ / (192 π³)
/// ```
pub trait WeakFieldOps<S>
where
    S: RealField + ToPrimitive + Clone + From<f64> + Into<f64>,
{
    /// Creates a new Weak Interaction Field (SU(2)) with West Coast metric.
    fn new_field(
        base: deep_causality_topology::Manifold<S, S>,
        connection: CausalTensor<S>,
    ) -> Result<Self, PhysicsError>
    where
        Self: Sized;

    fn fermi_constant(&self) -> S;
    fn w_mass(&self) -> S;
    fn z_mass(&self) -> S;
    fn sin2_theta_w(&self) -> S;
    fn charged_current_propagator(momentum_transfer_sq: S) -> Result<S, PhysicsError>;
    fn neutral_current_propagator(
        momentum_transfer_sq: S,
        fermion: &crate::WeakIsospin,
    ) -> Result<S, PhysicsError>;
    fn weak_decay_width(mass: S) -> Result<S, PhysicsError>;
    fn muon_lifetime() -> S;
    fn w_boson_width() -> S;
    fn z_boson_width() -> S;
    fn weak_field_strength(&self) -> CausalTensor<S>;
}
