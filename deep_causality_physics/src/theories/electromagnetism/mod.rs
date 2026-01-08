/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Gauge-Theoretic Electromagnetism (Gauge EM) Module
//!
//! Provides a unified interface for electromagnetic field operations using the
//! U(1) gauge field structure. This implements classical electrodynamics in the
//! language of gauge theory, with manifest Lorentz covariance.
//!
//! **Note**: This module computes classical observables (E, B, energy density, etc.)
//! using gauge field formalism. For true QED with quantum corrections, propagators,
//! and loop calculations, a separate quantum module would be needed.

mod gauge_em_ops;
mod gauge_em_ops_impl;

pub use gauge_em_ops::*;
