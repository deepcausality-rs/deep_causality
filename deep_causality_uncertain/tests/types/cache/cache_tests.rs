/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_uncertain::{
    GlobalSampleCache, SampledValue, UncertainError, with_global_cache,
};
use std::sync::Arc;

// Helper to reset the global cache for isolated tests
fn reset_global_cache() {
    with_global_cache(|cache| cache.clear());
}

#[test]
fn test_get_global_cache_returns_same_instance_on_same_thread() {
    reset_global_cache();

    let mut ptr1: *const GlobalSampleCache = std::ptr::null();
    let mut ptr2: *const GlobalSampleCache = std::ptr::null();

    with_global_cache(|cache1| {
        ptr1 = cache1;
    });

    with_global_cache(|cache2| {
        ptr2 = cache2;
    });

    assert!(!ptr1.is_null());
    assert_eq!(ptr1, ptr2);
}

#[test]
fn test_cache_get_and_insert() {
    reset_global_cache();
    with_global_cache(|cache| {
        let key_float = (1, 1_u64);
        let value_float = SampledValue::Float(123.45);

        let key_bool = (2, 2_u64);
        let value_bool = SampledValue::Bool(true);

        // Test initial get (should be None)
        assert_eq!(cache.get(&key_float), None);
        assert_eq!(cache.get(&key_bool), None);

        // Test insert
        cache.insert(key_float, value_float);
        cache.insert(key_bool, value_bool);

        // Test get after insert (should be Some)
        assert_eq!(cache.get(&key_float), Some(value_float));
        assert_eq!(cache.get(&key_bool), Some(value_bool));

        // Test updating a value
        let new_value_float = SampledValue::Float(999.99);
        cache.insert(key_float, new_value_float);
        assert_eq!(cache.get(&key_float), Some(new_value_float));
    });
}

#[test]
fn test_cache_get_or_compute_not_in_cache() {
    reset_global_cache();
    with_global_cache(|cache| {
        let key = (3, 3_u64);
        let mut compute_count = 0;

        let computed_value = cache
            .get_or_compute(key, || {
                compute_count += 1;
                Ok(SampledValue::Float(50.0))
            })
            .unwrap();

        assert_eq!(computed_value, SampledValue::Float(50.0));
        assert_eq!(compute_count, 1); // compute_fn should have been called
        assert_eq!(cache.get(&key), Some(SampledValue::Float(50.0))); // Value should be cached
    });
}

#[test]
fn test_cache_get_or_compute_already_in_cache() {
    reset_global_cache();
    with_global_cache(|cache| {
        let key = (4, 4_u64);
        let mut compute_count = 0;

        // Insert value directly first
        cache.insert(key, SampledValue::Bool(false));

        let computed_value = cache
            .get_or_compute(key, || {
                compute_count += 1;
                Ok(SampledValue::Bool(true)) // This should not be called
            })
            .unwrap();

        assert_eq!(computed_value, SampledValue::Bool(false)); // Should return cached value
        assert_eq!(compute_count, 0); // compute_fn should NOT have been called
    });
}

#[test]
fn test_cache_get_or_compute_compute_fn_returns_error() {
    reset_global_cache();
    with_global_cache(|cache| {
        let key = (5, 5_u64);

        let error_result = cache.get_or_compute(key, || {
            Err(UncertainError::SamplingError("Test error".to_string()))
        });

        assert!(error_result.is_err());
        assert_eq!(
            error_result.unwrap_err().to_string(),
            "Sampling error: Test error"
        );
        assert_eq!(cache.get(&key), None); // Error should not be cached
    });
}

#[test]
fn test_cache_clear() {
    reset_global_cache();
    with_global_cache(|cache| {
        let key1 = (6, 6_u64);
        let value1 = SampledValue::Float(1.0);
        cache.insert(key1, value1);
        assert_eq!(cache.get(&key1), Some(value1));

        let key2 = (7, 7_u64);
        let value2 = SampledValue::Bool(false);
        cache.insert(key2, value2);
        assert_eq!(cache.get(&key2), Some(value2));

        cache.clear();

        assert_eq!(cache.get(&key1), None);
        assert_eq!(cache.get(&key2), None);
    });
}

#[test]
fn test_sampled_value_debug_clone_copy() {
    let float_val = SampledValue::Float(12.34);
    let bool_val = SampledValue::Bool(true);

    // Test Debug
    assert_eq!(format!("{:?}", float_val), "Float(12.34)");
    assert_eq!(format!("{:?}", bool_val), "Bool(true)");

    // Test Clone
    let cloned_float = float_val.clone();
    let cloned_bool = bool_val.clone();
    assert_eq!(cloned_float, float_val);
    assert_eq!(cloned_bool, bool_val);

    // Test Copy (by assignment)
    let copied_float = float_val;
    let copied_bool = bool_val;
    assert_eq!(copied_float, float_val); // float_val is still valid after copy
    assert_eq!(copied_bool, bool_val); // bool_val is still valid after copy
}

// Test concurrent access using a shared cache instance.
#[test]
fn test_cache_concurrent_access() {
    // Create a new, dedicated cache instance wrapped in Arc for sharing across threads.
    let cache = Arc::new(GlobalSampleCache::new());
    let num_threads = 10;
    let num_inserts_per_thread = 100;

    let mut handles = vec![];

    for i in 0..num_threads {
        let cache_clone = Arc::clone(&cache);
        let handle = std::thread::spawn(move || {
            for j in 0..num_inserts_per_thread {
                let key = (i, j as u64);
                let value = SampledValue::Float((i * num_inserts_per_thread + j) as f64);
                cache_clone.insert(key, value);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Verify all values are present in the single shared cache.
    let mut expected_count = 0;
    for i in 0..num_threads {
        for j in 0..num_inserts_per_thread {
            let key = (i, j as u64);
            let expected_value = SampledValue::Float((i * num_inserts_per_thread + j) as f64);
            assert_eq!(cache.get(&key), Some(expected_value));
            expected_count += 1;
        }
    }

    println!(
        "Successfully inserted and retrieved {} items concurrently.",
        expected_count
    );
}
