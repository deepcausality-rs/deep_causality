/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
// Modules using `rusty_fork_test!` are skipped under Miri: that macro spawns
// a subprocess via `posix_spawn`, and Miri does not implement the
// `posix_spawn*` libc shims, so the tests abort. They are correct and run
// under normal CI.
#[cfg(test)]
#[cfg(not(miri))]
mod uncertain_arithmetic_tests;
#[cfg(test)]
#[cfg(not(miri))]
mod uncertain_comparison_tests;
#[cfg(test)]
#[cfg(not(miri))]
mod uncertain_default;
#[cfg(test)]
#[cfg(not(miri))]
mod uncertain_logic_tests;
#[cfg(test)]
#[cfg(not(miri))]
mod uncertain_sampling_tests;
#[cfg(test)]
#[cfg(not(miri))]
mod uncertain_statistics_tests;
#[cfg(test)]
#[cfg(not(miri))]
mod uncertain_tests;
