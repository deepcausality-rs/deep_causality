/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
#[cfg(test)]
// Disabled under Miri: Miri's soft-float emulation drifts by ~1 ULP (got 0.9999999996 vs 1)
#[cfg(all(test, not(miri)))]
mod qmc_sampler_tests;
#[cfg(test)]
mod sampler_seed_tests;
#[cfg(test)]
// Disabled under Miri: Miri's soft-float emulation drifts by ~1 ULP (got 0.9999999996 vs 1)
#[cfg(all(test, not(miri)))]
mod sequential_sampler_tests;
