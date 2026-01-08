/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use criterion::criterion_main;

mod benchmarks;

criterion_main! {
    benchmarks::manifold_bench::manifold_benches
}
