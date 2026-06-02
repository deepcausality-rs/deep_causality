/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::brcd::brcd_cache::{FamilyCache, family_key};
use deep_causality_algorithms::brcd::brcd_error::{BrcdError, BrcdErrorEnum};
use std::cell::Cell;

#[test]
fn family_key_sorts_and_dedups_parents() {
    assert_eq!(family_key(2, &[3, 1, 1]), (2, vec![1, 3]));
    assert_eq!(family_key(0, &[]), (0, vec![]));
    // Order-independent.
    assert_eq!(family_key(0, &[1, 2]), family_key(0, &[2, 1]));
}

#[test]
fn computes_once_then_reuses() {
    let mut cache: FamilyCache<f64> = FamilyCache::new();
    let calls = Cell::new(0);
    let compute = || {
        calls.set(calls.get() + 1);
        Ok(vec![1.0, 2.0, 3.0])
    };

    let first = cache
        .get_or_try_insert_with(0, &[1, 2], compute)
        .unwrap()
        .to_vec();
    assert_eq!(first, vec![1.0, 2.0, 3.0]);
    assert_eq!(calls.get(), 1);
    assert_eq!(cache.len(), 1);

    // Second request for the same family (parent order flipped) → cache hit.
    let second = cache
        .get_or_try_insert_with(0, &[2, 1], || {
            calls.set(calls.get() + 1);
            Ok(vec![9.0])
        })
        .unwrap()
        .to_vec();
    assert_eq!(second, vec![1.0, 2.0, 3.0]); // original cached value
    assert_eq!(calls.get(), 1); // not recomputed
    assert_eq!(cache.len(), 1);
}

#[test]
fn distinct_families_are_separate_entries() {
    let mut cache: FamilyCache<f64> = FamilyCache::new();
    cache
        .get_or_try_insert_with(0, &[1], || Ok(vec![1.0]))
        .unwrap();
    cache
        .get_or_try_insert_with(1, &[0], || Ok(vec![2.0]))
        .unwrap();
    assert_eq!(cache.len(), 2);
}

#[test]
fn get_reflects_insertion() {
    let mut cache: FamilyCache<f64> = FamilyCache::new();
    assert!(cache.is_empty());
    assert_eq!(cache.get(0, &[1]), None);
    cache
        .get_or_try_insert_with(0, &[1], || Ok(vec![5.0]))
        .unwrap();
    assert_eq!(cache.get(0, &[1]), Some(&[5.0][..]));
}

#[test]
fn compute_error_caches_nothing() {
    let mut cache: FamilyCache<f64> = FamilyCache::new();
    let result = cache.get_or_try_insert_with(0, &[1], || Err(BrcdError(BrcdErrorEnum::EmptyData)));
    assert_eq!(result.err(), Some(BrcdError(BrcdErrorEnum::EmptyData)));
    assert!(cache.is_empty());
}
