// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

pub(crate) mod unsafe_storage_array;
pub(crate) mod unsafe_storage_vec;

#[cfg(feature = "unsafe")]
pub use unsafe_storage_array::UnsafeArrayStorage;
#[cfg(feature = "unsafe")]
pub use unsafe_storage_vec::UnsafeVectorStorage;
