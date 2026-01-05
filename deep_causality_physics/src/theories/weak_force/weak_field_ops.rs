/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// =============================================================================
// Weak Field Operations Trait
// =============================================================================

use crate::{PhysicsError, WeakIsospin};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::Manifold;

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
pub trait WeakFieldOps {
    /// Creates a new Weak Field (SU(2)) with West Coast metric.
    fn new_field(base: Manifold<f64>, connection: CausalTensor<f64>) -> Result<Self, PhysicsError>
    where
        Self: Sized;

    fn fermi_constant(&self) -> f64;
    fn w_mass(&self) -> f64;
    fn z_mass(&self) -> f64;
    fn sin2_theta_w(&self) -> f64;
    fn charged_current_propagator(momentum_transfer_sq: f64) -> Result<f64, PhysicsError>;
    fn neutral_current_propagator(
        momentum_transfer_sq: f64,
        fermion: &WeakIsospin,
    ) -> Result<f64, PhysicsError>;
    fn weak_decay_width(mass: f64) -> Result<f64, PhysicsError>;
    fn muon_lifetime() -> f64;
    fn w_boson_width() -> f64;
    fn z_boson_width() -> f64;
    fn weak_field_strength(&self) -> CausalTensor<f64>;
}
