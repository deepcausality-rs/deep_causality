/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use core::marker::PhantomData;
use deep_causality_rand::types::misc::map::Map;

// Dummy types for testing
#[derive(Debug, Clone)]
struct DummyDistr;

#[derive(Debug, Clone)]
struct DummyFunc;

#[test]
fn test_map_creation_and_access() {
    let distr = DummyDistr;
    let func = DummyFunc;
    let map = Map {
        distr: distr.clone(),
        func: func.clone(),
        phantom: PhantomData::<(f64, u32)>,
    };

    // Test field access
    assert!(format!("{:?}", map.distr).contains("DummyDistr"));
    assert!(format!("{:?}", map.func).contains("DummyFunc"));
}

#[test]
fn test_map_debug_trait() {
    let distr = DummyDistr;
    let func = DummyFunc;
    let map = Map {
        distr,
        func,
        phantom: PhantomData::<(u32, bool)>,
    };

    let debug_str = format!("{:?}", map);
    assert!(debug_str.contains("Map"));
    assert!(debug_str.contains("distr: DummyDistr"));
    assert!(debug_str.contains("func: DummyFunc"));
    assert!(debug_str.contains("phantom: PhantomData<(u32, bool)>"));
}

#[test]
fn test_map_clone_trait() {
    let distr = DummyDistr;
    let func = DummyFunc;
    let map = Map {
        distr,
        func,
        phantom: PhantomData::<(f32, f64)>,
    };

    let cloned_map = map.clone();
    assert_eq!(format!("{:?}", map), format!("{:?}", cloned_map));
}
