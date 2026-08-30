/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
// `uncertain_maybe_f64_*` modules are skipped under Miri: they use
// `rusty_fork_test!`, which spawns a subprocess via `posix_spawn`. Miri does
// not implement the `posix_spawn*` libc shims, so the tests abort. They are
// correct and run under normal CI.
#[cfg(test)]
#[cfg(not(miri))]
mod uncertain_maybe_bool_tests;
#[cfg(test)]
#[cfg(not(miri))]
mod uncertain_maybe_f64_tests;

#[cfg(test)]
#[cfg(not(miri))]
mod uncertain_maybe_f64_arithmetic_tests;

#[cfg(test)]
#[cfg(not(miri))]
mod uncertain_maybe_f106_tests;
