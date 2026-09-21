/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::*;

#[test]
fn test_space_temporal_interval_time_is_raw_seconds() {
    // TangentSpacetime has no time_scale; `time()` returns `t` unchanged.
    let s = TangentSpacetime::new(1, 1.0, 2.0, 3.0, 42.0, 1.0, 0.0, 0.0, 0.0);
    assert_eq!(SpaceTemporalInterval::time(&s), 42.0);
}

#[test]
fn test_space_temporal_interval_position() {
    let s = TangentSpacetime::new(1, 1.0, 2.0, 3.0, 4.0, 1.0, 0.0, 0.0, 0.0);
    assert_eq!(SpaceTemporalInterval::position(&s), [1.0, 2.0, 3.0]);
}
