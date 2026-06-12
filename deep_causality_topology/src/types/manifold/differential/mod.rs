/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Differential operators for Manifold.
//!
//! Contains Hodge-Laplacian, exterior derivative, codifferential,
//! and Hodge star operators. These are sparse matrix operations.

mod codifferential;
mod de_rham;
mod exterior;
mod hodge;
pub(super) mod hodge_decomposition_impl;
mod interior_product;
mod laplacian;
mod leray;
mod spectral_poisson;
mod wedge;

pub use hodge_decomposition_impl::HodgeDecomposeOptions;

// Shared utilities
pub(super) mod utils_differential;
