/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Shared test fixtures. They live in the source tree because Bazel test targets cannot reach a
//! helper under `tests/`, so they are compiled into the library and tested like any other module.

#[cfg(feature = "qcm")]
pub(crate) mod crosstalk;
pub(crate) mod fixtures;
pub(crate) mod hand_built_complex;

#[cfg(feature = "qcm")]
pub use crosstalk::*;
pub use fixtures::*;
pub use hand_built_complex::*;
