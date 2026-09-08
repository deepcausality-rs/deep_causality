/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The scoped fork-join map: order preservation, equivalence with the
//! sequential map, borrow-friendly closures, edge sizes, and where the work
//! runs. Every assertion but the last test's holds with and without the
//! `parallel` feature — run the suite in both modes.

use deep_causality_par::scoped_map;
use std::collections::HashSet;
use std::thread;

#[test]
fn empty_slice_yields_empty_vec() {
    let out: Vec<i64> = scoped_map(&[] as &[i64], |x| x * 2);
    assert!(out.is_empty());
}

#[test]
fn single_element_maps_inline() {
    assert_eq!(scoped_map(&[21i64], |x| x * 2), vec![42]);
}

#[test]
fn order_is_preserved_and_matches_the_sequential_map() {
    let items: Vec<usize> = (0..1000).collect();
    let sequential: Vec<usize> = items.iter().map(|&x| x * x + 1).collect();
    let mapped = scoped_map(&items, |&x| x * x + 1);
    assert_eq!(mapped, sequential);
}

#[test]
fn the_closure_can_borrow_from_the_caller() {
    // The scoped threads borrow `table` from this stack frame — the property
    // a 'static-spawn API could not offer.
    let table = [10.0f64, 20.0, 30.0];
    let indices = [2usize, 0, 1, 2];
    let out = scoped_map(&indices, |&i| table[i] * 2.0);
    assert_eq!(out, vec![60.0, 20.0, 40.0, 60.0]);
}

#[test]
fn fallible_tasks_collect_like_a_sequential_map() {
    let items = [1i64, 2, 3, 4];
    let ok: Result<Vec<i64>, String> = scoped_map(&items, |&x| Ok(x * 2)).into_iter().collect();
    assert_eq!(ok.unwrap(), vec![2, 4, 6, 8]);

    let err: Result<Vec<i64>, String> = scoped_map(&items, |&x| {
        if x == 3 {
            Err("boom".to_string())
        } else {
            Ok(x)
        }
    })
    .into_iter()
    .collect();
    assert_eq!(err.unwrap_err(), "boom");
}

#[test]
fn more_items_than_cores_still_covers_every_element() {
    // 4099 is prime, so the last chunk is ragged on every core count. Comparing the whole vector
    // rather than its length and its two ends leaves no interior slot unobserved.
    let items: Vec<i64> = (0..4099).collect();
    let sequential: Vec<i64> = items.iter().map(|&x| x + 1).collect();
    assert_eq!(scoped_map(&items, |&x| x + 1), sequential);
}

#[test]
fn the_work_is_partitioned_into_one_chunk_per_core() {
    // Both paths return the same values by construction, so no value oracle can tell them apart:
    // an implementation that never spawns, or one that spawns a thread per element, passes every
    // other test in this file. Observe where each element was computed instead.
    let items: Vec<usize> = (0..4099).collect();
    let caller = thread::current().id();
    let ids = scoped_map(&items, |_| thread::current().id());
    assert_eq!(ids.len(), items.len());
    let distinct: HashSet<thread::ThreadId> = ids.iter().copied().collect();

    let threads = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .min(items.len());

    if cfg!(feature = "parallel") && threads > 1 {
        // One scoped thread per contiguous chunk, and the caller only joins. A live thread's id
        // is unique, so the distinct count is the chunk count exactly, not an estimate.
        let chunk = items.len().div_ceil(threads);
        assert_eq!(
            distinct.len(),
            items.len().div_ceil(chunk),
            "one thread per chunk"
        );
        // The contract the chunk size exists to keep: the fan-out is bounded by the core count,
        // however the partition is computed.
        assert!(
            distinct.len() <= threads,
            "spawned {} threads on {threads} cores",
            distinct.len()
        );
        assert!(
            !distinct.contains(&caller),
            "work ran on the calling thread"
        );
    } else {
        // Serial build, or a machine with one core: the inline map, on this thread.
        assert_eq!(distinct.len(), 1);
        assert!(distinct.contains(&caller));
    }
}
