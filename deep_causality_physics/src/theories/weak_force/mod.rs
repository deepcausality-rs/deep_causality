/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Weak Force — SU(2)_L Gauge Theory Module
//!
//! # Mathematical Foundation
//!
//! The weak force is described by an SU(2)_L gauge theory with spontaneous
//! symmetry breaking via the Higgs mechanism.
//!
//! ## Gauge Field (Non-Abelian Connection)
//! ```text
//! W_μ = W_μ^a T_a = W_μ^a (σ_a/2)   for a = 1,2,3
//! ```
//! where σ_a are the Pauli matrices and T_a = σ_a/2 are the SU(2) generators.
//!
//! ## Field Strength Tensor (Non-Abelian Curvature)
//! ```text
//! W_μν^a = ∂_μ W_ν^a - ∂_ν W_μ^a + g ε^{abc} W_μ^b W_ν^c
//! ```
//! The extra term gε^{abc}W^b W^c arises from the non-commutativity of SU(2).
//!
//! ## Physical Bosons (After Symmetry Breaking)
//! ```text
//! W^± = (W^1 ∓ iW^2) / √2     (charged current)
//! Z^0 = W^3 cos θ_W - B sin θ_W   (neutral current)
//! A   = W^3 sin θ_W + B cos θ_W   (photon)
//! ```
//!
//! ## Fermi Theory (Low Energy Limit)
//! ```text
//! G_F / √2 = g² / (8 M_W²)
//! ```
pub mod utils_generators;
mod weak_field_ops;
mod weak_field_ops_impl;
mod weak_isospin;

pub use utils_generators::*;
pub use weak_field_ops::*;
pub use weak_isospin::*;
