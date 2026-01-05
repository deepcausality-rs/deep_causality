/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! General Relativity — SO(3,1) Lorentz Gauge Theory Module
//!
//! # Mathematical Foundation
//!
//! General Relativity is formulated as a gauge theory of the Lorentz group SO(3,1).
//! The gauge field structure maps directly to GR concepts:
//!
//! ## Gauge Field Correspondence
//! ```text
//! GaugeField<Lorentz, f64, f64>
//!   ├── Connection = Christoffel symbols Γ^ρ_μν
//!   └── Field Strength = Riemann tensor R^ρ_σμν
//! ```
//!
//! ## Christoffel Symbols (Connection)
//! ```text
//! Γ^ρ_μν = ½ g^ρσ (∂_μ g_νσ + ∂_ν g_μσ - ∂_σ g_μν)
//! ```
//! These describe parallel transport and geodesic motion.
//!
//! ## Riemann Curvature Tensor (Field Strength)
//! ```text
//! R^ρ_σμν = ∂_μ Γ^ρ_νσ - ∂_ν Γ^ρ_μσ + Γ^ρ_μλ Γ^λ_νσ - Γ^ρ_νλ Γ^λ_μσ
//! ```
//! The non-abelian term Γ·Γ arises from Lorentz structure constants.
//!
//! ## Derived Tensors
//! ```text
//! Ricci Tensor:    R_μν = R^ρ_μρν    (contraction)
//! Ricci Scalar:    R = g^μν R_μν     (trace)
//! Einstein Tensor: G_μν = R_μν - ½ R g_μν
//! ```
//!
//! ## Einstein Field Equations
//! ```text
//! G_μν + Λ g_μν = (8πG/c⁴) T_μν
//! ```
//!
//! # Metric Convention
//!
//! GR uses East Coast convention: η_μν = diag(-1, +1, +1, +1)
mod adm_ops;
mod adm_state;
mod gr_ops;
mod gr_ops_impl;
mod metrics;

pub use adm_ops::*;
pub use adm_state::*;
pub use gr_ops::*;
pub use metrics::*;
