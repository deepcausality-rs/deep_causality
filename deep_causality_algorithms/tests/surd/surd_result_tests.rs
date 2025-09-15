/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::surd::SurdResult;
use deep_causality_data_structures::CausalTensor;
use std::collections::HashMap;

// Helper function to create a default SurdResult for testing
fn create_test_surd_result() -> SurdResult<i32> {
    let mut redundant_info = HashMap::new();
    redundant_info.insert(vec![1, 2], 0.5);

    let mut synergistic_info = HashMap::new();
    synergistic_info.insert(vec![3, 4], 0.8);

    let mut mutual_info = HashMap::new();
    mutual_info.insert(vec![5, 6], 1.2);

    let info_leak = 0.1;

    let mut causal_redundant_states = HashMap::new();
    causal_redundant_states.insert(
        vec![1],
        CausalTensor::new(vec![1], vec![1]).expect("Failed to create CausalTensor"),
    );

    let mut causal_unique_states = HashMap::new();
    causal_unique_states.insert(
        vec![2],
        CausalTensor::new(vec![2], vec![1]).expect("Failed to create CausalTensor"),
    );

    let mut causal_synergistic_states = HashMap::new();
    causal_synergistic_states.insert(
        vec![3],
        CausalTensor::new(vec![3], vec![1]).expect("Failed to create CausalTensor"),
    );

    let mut non_causal_redundant_states = HashMap::new();
    non_causal_redundant_states.insert(
        vec![4],
        CausalTensor::new(vec![4], vec![1]).expect("Failed to create CausalTensor"),
    );

    let mut non_causal_unique_states = HashMap::new();
    non_causal_unique_states.insert(
        vec![5],
        CausalTensor::new(vec![5], vec![1]).expect("Failed to create CausalTensor"),
    );

    let mut non_causal_synergistic_states = HashMap::new();
    non_causal_synergistic_states.insert(
        vec![6],
        CausalTensor::new(vec![6], vec![1]).expect("Failed to create CausalTensor"),
    );

    SurdResult::new(
        redundant_info,
        synergistic_info,
        mutual_info,
        info_leak,
        causal_redundant_states,
        causal_unique_states,
        causal_synergistic_states,
        non_causal_redundant_states,
        non_causal_unique_states,
        non_causal_synergistic_states,
    )
}

#[test]
fn test_constructor_and_all_getters() {
    let result = create_test_surd_result();

    // Test aggregate info getters
    assert_eq!(*result.redundant_info(), {
        let mut map = HashMap::new();
        map.insert(vec![1, 2], 0.5);
        map
    });
    assert_eq!(*result.synergistic_info(), {
        let mut map = HashMap::new();
        map.insert(vec![3, 4], 0.8);
        map
    });
    assert_eq!(*result.mutual_info(), {
        let mut map = HashMap::new();
        map.insert(vec![5, 6], 1.2);
        map
    });
    assert_eq!(result.info_leak(), 0.1);

    // Test causal states getters
    assert_eq!(result.causal_redundant_states().len(), 1);
    assert_eq!(
        result
            .causal_redundant_states()
            .get(&vec![1])
            .unwrap()
            .data(),
        &[1]
    );

    assert_eq!(result.causal_unique_states().len(), 1);
    assert_eq!(
        result.causal_unique_states().get(&vec![2]).unwrap().data(),
        &[2]
    );

    assert_eq!(result.causal_synergistic_states().len(), 1);
    assert_eq!(
        result
            .causal_synergistic_states()
            .get(&vec![3])
            .unwrap()
            .data(),
        &[3]
    );

    // Test non-causal states getters
    assert_eq!(result.non_causal_redundant_states().len(), 1);
    assert_eq!(
        result
            .non_causal_redundant_states()
            .get(&vec![4])
            .unwrap()
            .data(),
        &[4]
    );

    assert_eq!(result.non_causal_unique_states().len(), 1);
    assert_eq!(
        result
            .non_causal_unique_states()
            .get(&vec![5])
            .unwrap()
            .data(),
        &[5]
    );

    assert_eq!(result.non_causal_synergistic_states().len(), 1);
    assert_eq!(
        result
            .non_causal_synergistic_states()
            .get(&vec![6])
            .unwrap()
            .data(),
        &[6]
    );
}

#[test]
fn test_display_implementation() {
    let result = create_test_surd_result();
    let display_str = format!("{}", result);

    assert!(display_str.contains("--- SURD Decomposition Result ---"));
    assert!(display_str.contains("Aggregate Redundant Info: {[1, 2]: 0.5}"));
    assert!(display_str.contains("Aggregate Synergistic Info: {[3, 4]: 0.8}"));
    assert!(display_str.contains("Aggregate Mutual Info: {[5, 6]: 1.2}"));
    assert!(display_str.contains("Information Leak: 0.1"));
    assert!(display_str.contains("--- State-Dependent Maps ---"));
    assert!(display_str.contains("Causal Redundant States: [[1]]"));
    assert!(display_str.contains("Causal Unique States: [[2]]"));
    assert!(display_str.contains("Causal Synergistic States: [[3]]"));
    assert!(display_str.contains("Non-Causal Redundant States: [[4]]"));
    assert!(display_str.contains("Non-Causal Unique States: [[5]]"));
    assert!(display_str.contains("Non-Causal Synergistic States: [[6]]"));
}

#[test]
fn test_debug_implementation() {
    let result = create_test_surd_result();
    let debug_str = format!("{:?}", result);

    assert!(debug_str.starts_with("SurdResult"));
    assert!(debug_str.contains("redundant_info: {[1, 2]: 0.5}"));
    assert!(debug_str.contains("synergistic_info: {[3, 4]: 0.8}"));
    assert!(debug_str.contains("mutual_info: {[5, 6]: 1.2}"));
    assert!(debug_str.contains("info_leak: 0.1"));

    // Note: The exact debug output of CausalTensor might vary.
    // We check for key components.
    assert!(debug_str.contains("causal_redundant_states: {[1]: CausalTensor"));
    assert!(debug_str.contains("shape: [1]"));
    assert!(debug_str.contains("data: [1]"));

    assert!(debug_str.contains("causal_unique_states: {[2]: CausalTensor"));
    assert!(debug_str.contains("data: [2]"));

    assert!(debug_str.contains("causal_synergistic_states: {[3]: CausalTensor"));
    assert!(debug_str.contains("data: [3]"));

    assert!(debug_str.contains("non_causal_redundant_states: {[4]: CausalTensor"));
    assert!(debug_str.contains("data: [4]"));

    assert!(debug_str.contains("non_causal_unique_states: {[5]: CausalTensor"));
    assert!(debug_str.contains("data: [5]"));

    assert!(debug_str.contains("non_causal_synergistic_states: {[6]: CausalTensor"));
    assert!(debug_str.contains("data: [6]"));
}

#[test]
fn test_display_error_propagation() {
    use std::fmt::{self, Write};

    // A mock writer that is designed to fail on any write operation.
    struct FailingWriter;

    impl Write for FailingWriter {
        fn write_str(&mut self, _s: &str) -> fmt::Result {
            Err(fmt::Error)
        }
    }

    let surd_result = create_test_surd_result();
    let mut failing_writer = FailingWriter;

    // Attempt to write the SurdResult to the failing writer.
    // The `?` operator in the Display impl should propagate the error.
    let result = write!(&mut failing_writer, "{}", surd_result);

    // Assert that the write operation failed.
    assert!(result.is_err());
}
