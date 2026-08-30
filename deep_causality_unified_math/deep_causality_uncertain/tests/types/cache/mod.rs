/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
// `cache_tests` is skipped under Miri: it uses `rusty_fork_test!`, which
// spawns a subprocess via `posix_spawn`. Miri does not implement the
// `posix_spawn*` libc shims, so it aborts. The tests are correct and run
// under normal CI.
#[cfg(all(test, not(miri)))]
mod cache_tests;
#[cfg(test)]
mod sampled_value_tests;
