/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

pub mod causal_tensor_type;
pub mod grid_type;
pub mod window_type;

// Causal sensor type
pub use crate::causal_tensor_type::CausalTensor;
pub use crate::causal_tensor_type::error::CausalTensorError;
// Grid type
pub use crate::grid_type::ArrayGrid;
pub use crate::grid_type::ArrayType;
pub use crate::grid_type::ArrayType::*;
// Array Grid types
pub use crate::grid_type::grid::Grid;
pub use crate::grid_type::point::PointIndex;
pub use crate::grid_type::point::PointIndexType;
pub use crate::grid_type::storage::Storage;
// window types
pub use crate::window_type::SlidingWindow;

pub use crate::window_type::storage::WindowStorage;
pub use crate::window_type::storage_safe::storage_array::ArrayStorage;
pub use crate::window_type::storage_safe::storage_vec::VectorStorage;

#[cfg(feature = "unsafe")]
pub use crate::window_type::storage_unsafe::{UnsafeArrayStorage, UnsafeVectorStorage};
