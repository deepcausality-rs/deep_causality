/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Trait definitions for the tensor crate.

// Core tensor trait (contains the Tensor trait for CausalTensor)
pub(crate) mod tensor;

// Backend traits
pub(crate) mod backend_linear_algebra;
pub(crate) mod backend_tensor;
pub(crate) mod tensor_data;
