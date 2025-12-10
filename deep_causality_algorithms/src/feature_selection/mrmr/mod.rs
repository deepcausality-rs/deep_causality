/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

pub mod mrmr_algo;
pub mod mrmr_error;
pub mod mrmr_utils;

pub use mrmr_algo::*;
pub use mrmr_error::MrmrError;

pub mod mrmr_result;
#[cfg(test)]
mod mrmr_utils_tests;

pub use mrmr_result::MrmrResult;
