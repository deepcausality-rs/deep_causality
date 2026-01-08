/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Covariance analysis for Manifold fields.

// CPU implementation (always included)
mod covariance_cpu;

// MLX implementation (feature-gated)
#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
mod covariance_mlx;
