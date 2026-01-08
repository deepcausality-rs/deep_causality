/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Public covariance analysis API for Manifold.
//!
//! Dispatches to CPU or MLX implementations based on feature flags and data size.

// No re-export needed for inherent impls

#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
mod covariance_mlx;

#[cfg(not(all(feature = "mlx", target_os = "macos", target_arch = "aarch64")))]
mod covariance_cpu;
