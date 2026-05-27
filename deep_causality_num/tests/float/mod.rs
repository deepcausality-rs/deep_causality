/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
#[cfg(test)]
mod float_32_tests;
// Skipped wholesale under Miri: this module exercises f64 floating-point
// behavior, much of it via exact `assert_eq!` on transcendental results
// (sin/cos/cbrt/atan2/...). Miri's soft-float emulation differs from hardware
// in the last bit, so these comparisons spuriously fail one by one. The tests
// are correct and pass under normal CI.
#[cfg(all(test, not(miri)))]
mod float_64_tests;
