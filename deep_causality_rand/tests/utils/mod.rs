/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
#[cfg(test)]
// Disabled under Miri: Miri's soft-float emulation drifts by ~1 ULP (got 0.9999999996 vs 1)
#[cfg(not(miri))]
mod inverse_cdf_tests;
#[cfg(test)]
// Disabled under Miri: Miri's soft-float emulation drifts by ~1 ULP (got 0.9999999996 vs 1)
#[cfg(not(miri))]
mod sobol_tests;
