/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

pub const SMALL: usize = 10;
pub const MEDIUM: usize = 100;
pub const LARGE: usize = 1_000;

/// The sizes every scaling benchmark runs at; each becomes the parameter of a `BenchmarkId`.
pub const SIZES: [usize; 3] = [SMALL, MEDIUM, LARGE];

/// Seed of every random draw, so that two runs, before and after a change, measure the same
/// indices.
pub const SEED: u64 = 0x5EED;

/// Length of a precomputed index sequence; a power of two, so a cursor wraps with a mask.
pub const INDEX_COUNT: usize = 1_024;
