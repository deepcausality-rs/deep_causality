/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
// MLX-backed tensor - only available on Apple Silicon with mlx feature
#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
pub mod mlx_causal_tensor;
