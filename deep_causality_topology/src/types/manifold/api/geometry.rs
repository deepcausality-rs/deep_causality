/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Public geometry API for Manifold.
//!
//! Dispatches to CPU or MLX implementations based on feature flags and heuristics.

// No re-export needed for inherent impls

#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
mod geometry_mlx;

#[cfg(not(all(feature = "mlx", target_os = "macos", target_arch = "aarch64")))]
mod geometry_cpu;
