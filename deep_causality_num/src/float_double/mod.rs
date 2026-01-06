/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! High-precision double-double arithmetic module.
//!
//! This module provides `DoubleFloat`, a type that represents numbers as the
//! unevaluated sum of two `f64`s, achieving approximately 31 decimal digits
//! of precision.

mod ops_arithmetic;
mod ops_comparison;
mod traits_algebra;
mod traits_float;
mod traits_num;
pub mod types;
mod debug;
mod display;
mod from;
mod getters;
mod attributes;

pub use types::DoubleFloat;