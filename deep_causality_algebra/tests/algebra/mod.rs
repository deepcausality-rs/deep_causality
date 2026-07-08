/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// Disabled under Miri: software-emulated floats produce different last-bit
// results for transcendental ops, so exact equality cannot hold. The test
// itself is correct and runs under normal CI.
#[cfg(test)]
#[cfg(not(miri))]
mod algebra_tests;
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
mod commutative_semilattice_tests;
#[cfg(test)]
mod monoid_generic_tests;
#[cfg(test)]
#[cfg(not(miri))]
mod real_tests;
#[cfg(test)]
mod scalar_conjugate_tests;
#[cfg(test)]
mod scalar_normed_tests;
#[cfg(test)]
mod verdict_tests;
