/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// Feature-gated product implementations
#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
mod ops_product_mlx;

#[cfg(not(all(feature = "mlx", target_os = "macos", target_arch = "aarch64")))]
mod ops_product_cpu;

// Shared implementations (backend-agnostic)
mod ops_matrix_rep;
mod ops_misc_impl;
mod ops_norm_impl;
mod ops_product_impl;
