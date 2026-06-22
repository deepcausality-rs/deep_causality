/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Test module for DoubleFloat high-precision type.
#[cfg(test)]
mod double_algebra_tests;
#[cfg(test)]
mod double_arithmetic_tests;
#[cfg(test)]
mod double_attributes_tests;
#[cfg(test)]
mod double_comparison_tests;
#[cfg(test)]
mod double_display_tests;
// Disabled under Miri: software-emulated floats produce different last-bit
// results for transcendental ops, so exact equality cannot hold. The test
// itself is correct and runs under normal CI.
#[cfg(test)]
#[cfg(not(miri))]
mod double_erf_tests;
#[cfg(test)]
mod double_float_tests;
#[cfg(test)]
mod double_from_tests;
#[cfg(test)]
mod double_num_traits_tests;
#[cfg(test)]
mod double_ops_tests;
#[cfg(test)]
mod double_traits_tests;
// Disabled under Miri: software-emulated floats produce different last-bit
// results for transcendental ops, so exact equality cannot hold. The test
// itself is correct and runs under normal CI.
#[cfg(test)]
#[cfg(not(miri))]
mod double_transcendental_tests;
