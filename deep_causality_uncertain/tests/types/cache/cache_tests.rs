/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_uncertain::{SampledValue, Uncertain, with_global_cache};
use rusty_fork::rusty_fork_test;

rusty_fork_test! {
    #[test]
    fn test_singleton_behavior() {

        // This test verifies that within the same thread, we always get the same cache instance.
        // We do this by inserting a value and retrieving it within the same `with_global_cache` block,
        // which guarantees execution on the same thread.
        let uncertain_obj = Uncertain::<f64>::point(1.0);
        let key = (uncertain_obj.id(), 100);
        let value = SampledValue::Float(42.0);

        with_global_cache(|cache| {
            // Insert a value into the cache.
            cache.insert(key, value);

            // Retrieve the value immediately from the same cache instance.
            let retrieved_value = cache.get(&key);
            assert_eq!(retrieved_value, Some(value));
        });
    }

    #[test]
    fn test_cache_clear() {

        let uncertain_obj = Uncertain::<f64>::point(1.0);
        let key = (uncertain_obj.id(), 200);
        let value = SampledValue::Float(123.45);

        // Insert a value
        with_global_cache(|cache| {
            cache.insert(key, value);
        });

        // Verify it was inserted
        with_global_cache(|cache| {
            assert_eq!(cache.get(&key), Some(value));
        });

        // Clear the cache
        with_global_cache(|cache| {
            cache.clear();
        });

        // Verify it's gone
        with_global_cache(|cache| {
            assert_eq!(cache.get(&key), None);
        });
    }

    #[test]
    fn test_cache_get_and_insert() {

        let uncertain_obj = Uncertain::<f64>::point(1.0);
        let key = (uncertain_obj.id(), 0);
        let value = SampledValue::Float(999.99);

        // Initially, the cache should be empty
        with_global_cache(|cache| {
            assert_eq!(cache.get(&key), None);
        });

        // Insert the value
        with_global_cache(|cache| {
            cache.insert(key, value);
        });

        // Now, the value should be present
        with_global_cache(|cache| {
            assert_eq!(cache.get(&key), Some(value));
        });
    }

    #[test]
    fn test_cache_get_or_compute_not_in_cache() {

        let uncertain_obj = Uncertain::<f64>::point(1.0);
        let key = (uncertain_obj.id(), 1);
        let expected_value = SampledValue::Float(50.0);

        // The value is not in the cache, so the compute function should be called
        let result = with_global_cache(|cache| {
            cache.get_or_compute(key, || Ok(expected_value))
        });

        assert_eq!(result.unwrap(), expected_value);

        // The value should now be in the cache
        with_global_cache(|cache| {
            assert_eq!(cache.get(&key), Some(expected_value));
        });
    }

    #[test]
    fn test_cache_get_or_compute_already_in_cache() {

        let uncertain_obj = Uncertain::<f64>::point(1.0);
        let key = (uncertain_obj.id(), 2);
        let initial_value = SampledValue::Float(75.0);

        // Pre-populate the cache
        with_global_cache(|cache| {
            cache.insert(key, initial_value);
        });

        // get_or_compute should return the cached value and not call the compute function
        let result = with_global_cache(|cache| {
            cache.get_or_compute(key, || {
                // This closure should not be executed
                panic!("Compute function was called unnecessarily!");
            })
        });

        assert_eq!(result.unwrap(), initial_value);
    }
}
