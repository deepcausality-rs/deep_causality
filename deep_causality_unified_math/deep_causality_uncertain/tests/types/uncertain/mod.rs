/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
// The two forked modules below reach into the global sample cache directly — clearing it and inserting
// entries to pin what a statistic reads — so they need a process to themselves. `rusty_fork_test!`
// gives them one, and it costs them Miri, which does not implement the `posix_spawn*` shims the
// macro needs. Both the fork and the gate go when the cache does; the other modules here never
// needed either, measured at 0 failures in 300 runs at `--test-threads=16`.
#[cfg(test)]
mod uncertain_arithmetic_tests;
#[cfg(test)]
mod uncertain_comparison_tests;
#[cfg(test)]
mod uncertain_default_tests;
#[cfg(test)]
mod uncertain_logic_tests;
#[cfg(test)]
#[cfg(not(miri))]
mod uncertain_sampling_tests;
#[cfg(test)]
#[cfg(not(miri))]
mod uncertain_statistics_tests;
#[cfg(test)]
mod uncertain_tests;
