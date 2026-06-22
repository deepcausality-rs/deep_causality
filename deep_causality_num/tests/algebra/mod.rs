/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
#[cfg(test)]
mod algebra_tests;
#[cfg(test)]
mod domain_euclidean_tests;
// Disabled under Miri: software-emulated floats produce different last-bit
// results for transcendental ops, so exact equality cannot hold. The test
// itself is correct and runs under normal CI.
#[cfg(test)]
#[cfg(not(miri))]
mod field_real_f32_tests;
// Disabled under Miri: software-emulated floats produce different last-bit
// results for transcendental ops, so exact equality cannot hold. The test
// itself is correct and runs under normal CI.
#[cfg(test)]
#[cfg(not(miri))]
pub(crate) mod field_real_f64_tests;
// Disabled under Miri: software-emulated floats produce different last-bit
// results for transcendental ops, so exact equality cannot hold. The test
// itself is correct and runs under normal CI.
#[cfg(test)]
#[cfg(not(miri))]
mod real_tests;
