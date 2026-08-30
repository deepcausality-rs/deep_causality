/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Shared fixtures for the test suite.
//!
//! # Why these live in `src`
//!
//! Bazel cannot reach a helper file inside `tests/` — only the `src` tree is available to a test
//! target — so a helper placed there builds under Cargo and fails under Bazel. Living in `src` has a
//! consequence that is not optional: these are library code, the repository's coverage requirement
//! applies to them, and each has its own test at `tests/utils_tests/<name>_tests.rs`.

pub mod fixtures_cayley_menger;
pub mod fixtures_matrix;
