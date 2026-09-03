/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The scoped fork-join map: order preservation, equivalence with the
//! sequential map, borrow-friendly closures, and edge sizes. The same
//! assertions hold with and without the `parallel` feature — run the suite
//! in both modes.

use deep_causality_par::scoped_map;

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
    let items: Vec<i64> = (0..4099).collect();
    let out = scoped_map(&items, |&x| x + 1);
    assert_eq!(out.len(), items.len());
    assert_eq!(out.first(), Some(&1));
    assert_eq!(out.last(), Some(&4099));
}
