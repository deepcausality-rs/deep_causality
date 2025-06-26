/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#[cfg(not(feature = "unsafe"))]
pub use crate::grid_type::grid_safe::Grid;
#[cfg(feature = "unsafe")]
pub use crate::grid_type::grid_unsafe::Grid;
