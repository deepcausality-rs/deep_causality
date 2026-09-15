/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// Disabled under Miri: software-emulated floats produce different last-bit
// results for transcendental ops, so exact equality cannot hold. The test
// itself is correct and runs under normal CI.
#[cfg(test)]
#[cfg(not(miri))]
mod bfloat16_impl_tests;
// Disabled under Miri: software-emulated floats produce different last-bit
// results for transcendental ops, so exact equality cannot hold. The test
// itself is correct and runs under normal CI.
#[cfg(test)]
#[cfg(not(miri))]
mod float_32_tests;
// Disabled under Miri: software-emulated floats produce different last-bit
// results for transcendental ops, so exact equality cannot hold. The test
// itself is correct and runs under normal CI.
#[cfg(test)]
#[cfg(not(miri))]
mod float_64_tests;

// The fused multiply-add promise, checked across all four implementations at once.
// Disabled under Miri for the reason above: the probe turns on exact last-bit behaviour.
#[cfg(test)]
#[cfg(not(miri))]
mod fma_single_rounding_tests;
