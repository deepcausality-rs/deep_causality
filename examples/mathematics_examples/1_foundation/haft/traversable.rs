/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Traversable Example
//!
//! `Traversable::sequence` turns a structure inside out: `Vec<Option<T>>` becomes
//! `Option<Vec<T>>`, and `Vec<Result<T, E>>` becomes `Result<Vec<T>, E>`. The inner type
//! supplies the `Applicative`, which decides what "all of them" means -- `None` or the first
//! `Err` collapses the whole traversal.
//!
//! That is the atomic-batch operation: give me every record, or tell me which one is missing.

use deep_causality_haft::{OptionWitness, ResultWitness, Traversable, VecWitness};
use std::fmt::Debug;

fn main() {
    // 1. Atomic batch retrieval: all the users, or none.
    let results_ok: Vec<Option<User>> = vec![1, 2, 3].into_iter().map(fetch_user).collect();
    let raw_ok = results_ok.clone();
    let atomic_batch_ok = VecWitness::sequence::<User, OptionWitness>(results_ok);
    print_atomic_ok(&raw_ok, &atomic_batch_ok);
    assert!(atomic_batch_ok.is_some());

    // One missing user collapses the whole batch to None.
    let results_missing: Vec<Option<User>> = vec![1, 404, 3].into_iter().map(fetch_user).collect();
    let raw_missing = results_missing.clone();
    let atomic_batch_missing = VecWitness::sequence::<User, OptionWitness>(results_missing);
    print_atomic_missing(&raw_missing, &atomic_batch_missing);
    assert_eq!(atomic_batch_missing, None);

    // 2. Fail-fast validation. The inner applicative is Result, so the first Err wins.
    let validated_ok: Vec<Result<i32, String>> =
        vec![100, 200, 50].into_iter().map(validate_tx).collect();
    let block_ok = VecWitness::sequence::<i32, ResultWitness<String>>(validated_ok);

    let validated_invalid: Vec<Result<i32, String>> =
        vec![100, -50, 200].into_iter().map(validate_tx).collect();
    let block_invalid = VecWitness::sequence::<i32, ResultWitness<String>>(validated_invalid);

    print_validation(&block_ok, &block_invalid);
    assert_eq!(block_ok, Ok(vec![100, 200, 50]));
    assert_eq!(block_invalid, Err("Negative amount: -50".to_string()));

    // 3. The flip runs the other way too: OptionWitness is Traversable over Result.
    let nested: Option<Result<i32, String>> = Some(Ok(7));
    let flipped = OptionWitness::sequence::<i32, ResultWitness<String>>(nested);
    print_flip(&flipped);
    assert_eq!(flipped, Ok(Some(7)));
}

/// Fetches a user, where id 404 stands for "not found".
fn fetch_user(id: u32) -> Option<User> {
    if id == 404 {
        None
    } else {
        Some(User {
            id,
            name: format!("User_{id}"),
        })
    }
}

/// Rejects a negative transaction amount.
fn validate_tx(amount: i32) -> Result<i32, String> {
    if amount < 0 {
        Err(format!("Negative amount: {amount}"))
    } else {
        Ok(amount)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct User {
    id: u32,
    name: String,
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_atomic_ok<R: Debug, A: Debug>(raw: &R, atomic: &A) {
    println!("=== DeepCausality HKT: Traversable (sequence) ===\n");
    println!("--- 1. Atomic Batch Retrieval (All or Nothing) ---");
    println!("Vec<Option<User>>: {raw:?}");
    println!("sequence -> Option<Vec<User>>: {atomic:#?}");
}

fn print_atomic_missing<R: Debug, A: Debug>(raw: &R, atomic: &A) {
    println!("\nWith one missing: {raw:?}");
    println!("sequence -> {atomic:?}");
}

fn print_validation<T: Debug>(ok: &T, invalid: &T) {
    println!("\n--- 2. Fail-Fast Validation (Result) ---");
    println!("All valid:   {ok:?}");
    println!("One invalid: {invalid:?}");
}

fn print_flip<T: Debug>(flipped: &T) {
    println!("\n--- 3. The Same Flip, Other Way Round ---");
    println!("Some(Ok(7)) sequenced -> {flipped:?}");
}
