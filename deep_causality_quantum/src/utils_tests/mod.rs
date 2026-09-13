/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Shared test fixtures. They live in the source tree because Bazel test targets cannot reach a
//! helper under `tests/`, so they are compiled into the library and tested like any other module.

pub(crate) mod fixtures;
pub(crate) mod hand_built_complex;

pub use fixtures::*;
pub use hand_built_complex::*;
