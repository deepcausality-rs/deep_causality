/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{
    BTreeMapWitness, BoxWitness, Functor, HKT, HashMapWitness, LinkedListWitness, OptionWitness,
    ResultWitness, VecDequeWitness, VecWitness,
};
use std::collections::{BTreeMap, HashMap, LinkedList, VecDeque};
use std::fmt::Debug;

// ============================================================================
// Domain Logic: Data Anonymization
// ============================================================================

// A generic function that applies a transformation to ANY container that implements Functor.
//
// ENGINEERING VALUE:
// In a large system, data is stored in various structures (Vec, HashMap, Option, etc.).
// You often need to apply the SAME business logic (e.g., "Anonymize PII") to all of them.
//
// Instead of writing `anonymize_vec`, `anonymize_map`, `anonymize_option`, you write
// ONE generic function `process_batch` that works on ALL of them.
// This drastically reduces code duplication and ensures consistent behavior.
fn process_batch<F, T, U>(container: F::Type<T>, transform: impl FnMut(T) -> U) -> F::Type<U>
where
    F: Functor<F> + HKT,
{
    F::fmap(container, transform)
}

fn main() {
    print_header();

    // The transformation: mask any ID above 1000, keep the rest.
    let anonymize_id = |id: i32| -> String {
        if id > 1000 {
            "****".to_string()
        } else {
            id.to_string()
        }
    };

    // 1. Linear collections. One generic function serves all three.
    let user_ids_vec = vec![101, 5000, 42, 9999];
    let original_vec = user_ids_vec.clone();
    let masked_vec = process_batch::<VecWitness, _, _>(user_ids_vec, anonymize_id);
    let masked_list =
        process_batch::<LinkedListWitness, _, _>(LinkedList::from([101, 5000]), anonymize_id);
    let masked_deque =
        process_batch::<VecDequeWitness, _, _>(VecDeque::from([9999, 42]), anonymize_id);
    print_linear(&original_vec, &masked_vec, &masked_list, &masked_deque);
    assert_eq!(masked_vec, vec!["101", "****", "42", "****"]);

    // 2. Key-value stores. A map's Functor maps the value and preserves the key.
    let mut session_map = HashMap::new();
    session_map.insert("session_1", 101);
    session_map.insert("session_2", 5000);
    let original_map = session_map.clone();

    let mut ordered_map = BTreeMap::new();
    ordered_map.insert(1, 5000);
    ordered_map.insert(2, 101);

    let masked_map = process_batch::<HashMapWitness<&str>, _, _>(session_map, anonymize_id);
    let masked_btree = process_batch::<BTreeMapWitness<i32>, _, _>(ordered_map, anonymize_id);
    print_maps(&original_map, &masked_map, &masked_btree);

    // 3. Contexts: a value that may be missing, a fetch that may fail, a boxed value.
    let masked_opt = process_batch::<OptionWitness, _, _>(Some(5000), anonymize_id);
    let fetch_result: Result<i32, &str> = Ok(5000);
    let masked_res = process_batch::<ResultWitness<&str>, _, _>(fetch_result, anonymize_id);
    let masked_box = process_batch::<BoxWitness, _, _>(Box::new(5000), anonymize_id);
    print_contexts(&masked_opt, &masked_res, &masked_box);

    assert_eq!(masked_opt, Some("****".to_string()));
    assert_eq!(masked_res, Ok("****".to_string()));
    assert_eq!(masked_box, Box::new("****".to_string()));
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("=== DeepCausality HKT: Batch Data Processing ===\n");
}

fn print_linear<O: Debug, V: Debug, L: Debug, D: Debug>(
    original: &O,
    masked_vec: &V,
    masked_list: &L,
    masked_deque: &D,
) {
    println!("--- 1. Processing Linear Collections (Vec, List, Deque) ---");
    println!("Original Vec: {original:?}");
    println!("Masked Vec:   {masked_vec:?}");
    println!("Masked List:  {masked_list:?}");
    println!("Masked Deque: {masked_deque:?}");
}

fn print_maps<O: Debug, M: Debug, B: Debug>(original: &O, masked_map: &M, masked_btree: &B) {
    println!("\n--- 2. Processing Key-Value Stores (HashMap, BTreeMap) ---");
    println!("Original HashMap: {original:?}");
    println!("Masked HashMap:   {masked_map:?}");
    println!("Masked BTreeMap:  {masked_btree:?}");
}

fn print_contexts<O: Debug, R: Debug, B: Debug>(opt: &O, res: &R, boxed: &B) {
    println!("\n--- 3. Processing Contexts (Option, Result, Box) ---");
    println!("Masked Option: {opt:?}");
    println!("Masked Result: {res:?}");
    println!("Masked Box:    {boxed:?}");
}
