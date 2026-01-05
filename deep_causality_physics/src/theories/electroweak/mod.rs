/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2026" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Electroweak Theory — SU(2)_L × U(1)_Y Gauge Theory Module
//!
//! # Mathematical Foundation
//!
//! The electroweak theory unifies electromagnetic and weak forces via the
//! gauge group SU(2)_L × U(1)_Y with spontaneous symmetry breaking.
//!
//! ## Gauge Fields (Before Symmetry Breaking)
//! ```text
//! SU(2)_L: W^a_μ (a = 1,2,3) with coupling g
//! U(1)_Y:  B_μ with coupling g'
//! ```
//!
//! ## Symmetry Breaking (Higgs Mechanism)
//! ```text
//! Higgs doublet: φ = (φ⁺, φ⁰)ᵀ
//! Vacuum: ⟨φ⟩ = (0, v/√2)ᵀ  where v ≈ 246 GeV
//! ```
//!
//! ## Physical Bosons (After Breaking SU(2)×U(1) → U(1)_EM)
//! ```text
//! W^± = (W¹ ∓ iW²) / √2       — charged, mass M_W = gv/2
//! Z⁰  = W³ cos θ_W - B sin θ_W — neutral, mass M_Z = M_W/cos θ_W
//! A   = W³ sin θ_W + B cos θ_W — photon, massless
//! ```
//!
//! ## Weinberg Angle
//! ```text
//! tan θ_W = g'/g        sin² θ_W ≈ 0.231
//! e = g sin θ_W = g' cos θ_W
//! ```
//!
//! ## Mass Generation
//! ```text
//! Gauge boson masses: M_W = gv/2, M_Z = M_W/cos θ_W
//! Fermion masses: m_f = y_f v / √2  (Yukawa coupling)
//! Higgs mass: M_H = √(2λ) v ≈ 125 GeV
//! ```

mod electroweak_impl;
mod electroweak_ops;
mod electroweak_params;

pub use electroweak_ops::*;
pub use electroweak_params::*;
