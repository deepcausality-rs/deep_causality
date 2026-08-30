/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
// Skipped under Miri: this module uses `rusty_fork_test!`, which spawns a
// subprocess via `posix_spawn`. Miri does not implement the `posix_spawn*`
// libc shims, so any test wrapped in `rusty_fork_test!` aborts under Miri.
// The tests themselves are correct and run under normal CI.
#[cfg(all(test, not(miri)))]
mod sprt_eval_tests;
