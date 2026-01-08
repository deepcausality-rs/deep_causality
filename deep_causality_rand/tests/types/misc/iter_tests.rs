/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use core::marker::PhantomData;
use deep_causality_rand::types::misc::iter::Iter;

// Dummy types for testing
#[derive(Debug, Clone)]
struct DummyDistr;

#[derive(Debug, Clone)]
struct DummyRng;

#[test]
fn test_iter_creation_and_access() {
    let distr = DummyDistr;
    let rng = DummyRng;
    let iter = Iter {
        distr: distr.clone(),
        rng: rng.clone(),
        phantom: PhantomData::<f64>,
    };

    // Test field access
    assert!(format!("{:?}", iter.distr).contains("DummyDistr"));
    assert!(format!("{:?}", iter.rng).contains("DummyRng"));
}

#[test]
fn test_iter_debug_trait() {
    let distr = DummyDistr;
    let rng = DummyRng;
    let iter = Iter {
        distr,
        rng,
        phantom: PhantomData::<u32>,
    };

    let debug_str = format!("{:?}", iter);
    assert!(debug_str.contains("Iter"));
    assert!(debug_str.contains("distr: DummyDistr"));
    assert!(debug_str.contains("rng: DummyRng"));
    assert!(debug_str.contains("phantom: PhantomData<u32>"));
}

#[test]
fn test_iter_clone_trait() {
    let distr = DummyDistr;
    let rng = DummyRng;
    let iter = Iter {
        distr,
        rng,
        phantom: PhantomData::<bool>,
    };

    let cloned_iter = iter.clone();
    assert_eq!(format!("{:?}", iter), format!("{:?}", cloned_iter));
}
