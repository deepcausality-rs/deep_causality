/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Differential operators for Manifold.
//!
//! Contains Hodge-Laplacian, exterior derivative, codifferential,
//! and Hodge star operators. These are sparse matrix operations.

mod codifferential;
mod exterior;
mod hodge;
mod hodge_decomposition_impl;
mod laplacian;

pub use hodge_decomposition_impl::HodgeDecomposeOptions;

// Shared utilities
pub(super) mod utils_differential;
