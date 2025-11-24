/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{OptionWitness, ResultWitness, Traversable, VecWitness};

// ============================================================================
// Domain Logic: Batch Operations
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
struct User {
    id: u32,
    name: String,
}

fn main() {
    println!("=== DeepCausality HKT: Batch Aggregation (Traversable) ===\n");

    // ------------------------------------------------------------------------
    // Concept: Traversable (Flipping Structure)
    //
    // ENGINEERING VALUE:
    // When processing a batch of items, you often end up with a "List of Results"
    // (Vec<Result<T, E>>).
    //
    // However, for an atomic operation, you often want a "Result of a List"
    // (Result<Vec<T>, E>). i.e., "Give me all the data, or fail if any is missing."
    //
    // `sequence` performs this transformation automatically. It implements the
    // "Fail-Fast" pattern for batch processing.
    // ------------------------------------------------------------------------

    println!("--- 1. Atomic Batch Retrieval (All or Nothing) ---");

    // Scenario: Fetching Users by ID.
    // Some IDs exist, some might not.
    let fetch_user = |id: u32| -> Option<User> {
        if id == 404 {
            None // User not found
        } else {
            Some(User {
                id,
                name: format!("User_{}", id),
            })
        }
    };

    // Case A: All users exist
    let batch_ids_ok = vec![1, 2, 3];
    // Map fetch over IDs -> Vec<Option<User>>
    let results_ok: Vec<Option<User>> = batch_ids_ok.into_iter().map(fetch_user).collect();
    println!("Raw Results (OK): {:?}", results_ok);

    // Sequence: Vec<Option<User>> -> Option<Vec<User>>
    type OptionMonad = OptionWitness;
    let atomic_batch_ok: Option<Vec<User>> = VecWitness::sequence::<User, OptionMonad>(results_ok);

    println!("Atomic Batch (OK): {:#?}", atomic_batch_ok);
    assert!(atomic_batch_ok.is_some());

    // Case B: One user is missing
    let batch_ids_missing = vec![1, 404, 3];
    let results_missing: Vec<Option<User>> =
        batch_ids_missing.into_iter().map(fetch_user).collect();
    println!("\nRaw Results (Missing): {:?}", results_missing);

    // Sequence: One None causes the whole batch to be None
    let atomic_batch_missing: Option<Vec<User>> =
        VecWitness::sequence::<User, OptionMonad>(results_missing);

    println!("Atomic Batch (Missing): {:?}", atomic_batch_missing);
    assert_eq!(atomic_batch_missing, None);

    println!("\n--- 2. Fail-Fast Validation (Result) ---");

    // Scenario: Validating a list of transactions.
    // If ANY transaction is invalid, the whole block is rejected.
    let validate_tx = |amount: i32| -> Result<i32, String> {
        if amount < 0 {
            Err(format!("Negative amount: {}", amount))
        } else {
            Ok(amount)
        }
    };

    // Case A: All valid
    let tx_batch_ok = vec![100, 200, 50];
    let validation_results_ok: Vec<Result<i32, String>> =
        tx_batch_ok.into_iter().map(validate_tx).collect();

    type ResultMonad = ResultWitness<String>;
    let block_ok: Result<Vec<i32>, String> =
        VecWitness::sequence::<i32, ResultMonad>(validation_results_ok);

    println!("Block Validation (OK): {:?}", block_ok);
    assert!(block_ok.is_ok());

    // Case B: One invalid
    let tx_batch_invalid = vec![100, -50, 200];
    let validation_results_invalid: Vec<Result<i32, String>> =
        tx_batch_invalid.into_iter().map(validate_tx).collect();

    // Sequence: The first Err aborts the sequence and is returned
    let block_invalid: Result<Vec<i32>, String> =
        VecWitness::sequence::<i32, ResultMonad>(validation_results_invalid);

    println!("Block Validation (Invalid): {:?}", block_invalid);
    assert_eq!(block_invalid, Err("Negative amount: -50".to_string()));
}
