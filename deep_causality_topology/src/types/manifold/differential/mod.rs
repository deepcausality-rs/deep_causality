/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Differential operators for Manifold.
//!
//! Contains Hodge-Laplacian, exterior derivative, codifferential,
//! and Hodge star operators. These are sparse matrix operations.

// CPU implementations (sparse matrix operations)
mod codifferential_cpu;
mod exterior_cpu;
mod hodge_cpu;
mod laplacian_cpu;

// Shared utilities
pub(super) mod utils_differential;
