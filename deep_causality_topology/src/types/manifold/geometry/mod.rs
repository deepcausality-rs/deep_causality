/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Geometry implementations for Manifold.

// CPU implementation (always included)
mod geometry_cpu;

// MLX implementation (feature-gated)
#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
mod geometry_mlx;
