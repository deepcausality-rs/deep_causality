/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
pub(crate) mod ext_hkt;
pub(crate) mod ext_math;
mod ext_math_f32_impl;
mod ext_math_f64_impl;
pub(crate) mod ext_stack;
mod ext_stack_impl;

// MLX bridge module - only available on Apple Silicon with mlx feature
#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
pub(crate) mod ext_mlx;
