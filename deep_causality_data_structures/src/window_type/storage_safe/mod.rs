/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#![forbid(unsafe_code)]

pub(crate) mod storage_array;
// Backed by a `Vec`, so it needs `alloc`.
#[cfg(feature = "alloc")]
pub(crate) mod storage_vec;
