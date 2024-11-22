// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

#[cfg(not(feature = "unsafe"))]
pub use crate::grid_type::grid_safe::Grid;
#[cfg(feature = "unsafe")]
pub use crate::grid_type::grid_unsafe::Grid;
