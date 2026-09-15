/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The address of one draw.

use deep_causality_uncertain::{SampleIndex, SampleSession};
use std::collections::HashSet;

#[test]
fn an_address_carries_the_seed_and_the_index() {
    let session = SampleSession::seeded(0xABCD);
    let at = SampleIndex::at(&session, 7);

    assert_eq!(at.seed(), 0xABCD);
    assert_eq!(at.index(), 7);
}

/// Equality is on both halves, so neither can drift without being noticed.
#[test]
fn equality_is_on_both_halves() {
    let a = SampleIndex::at(&SampleSession::seeded(1), 1);
    assert_eq!(a, SampleIndex::at(&SampleSession::seeded(1), 1));
    assert_ne!(
        a,
        SampleIndex::at(&SampleSession::seeded(2), 1),
        "seed differs"
    );
    assert_ne!(
        a,
        SampleIndex::at(&SampleSession::seeded(1), 2),
        "index differs"
    );
}

/// Naming an address does not advance the session — that is what makes two quantities addressable
/// at the *same* index.
#[test]
fn naming_an_address_does_not_advance_the_session() {
    let session = SampleSession::seeded(5);
    let first = SampleIndex::at(&session, 0);
    let second = SampleIndex::at(&session, 0);
    assert_eq!(first, second);
}

#[test]
fn a_sequence_walks_zero_to_n_in_order() {
    let session = SampleSession::seeded(9);
    let walked: Vec<u64> = SampleIndex::sequence(&session, 5)
        .map(|a| a.index())
        .collect();
    assert_eq!(walked, vec![0, 1, 2, 3, 4]);

    let seeds: HashSet<u64> = SampleIndex::sequence(&session, 5)
        .map(|a| a.seed())
        .collect();
    assert_eq!(
        seeds,
        HashSet::from([9]),
        "every address in a sequence shares the seed"
    );
}

/// A sequence of length zero yields nothing rather than one address or an error.
#[test]
fn an_empty_sequence_yields_nothing() {
    let session = SampleSession::seeded(9);
    assert_eq!(SampleIndex::sequence(&session, 0).count(), 0);
}

/// The extremes of the index range are addresses like any other.
#[test]
fn the_extremes_of_the_range_are_addressable() {
    let session = SampleSession::seeded(u64::MAX);
    let top = SampleIndex::at(&session, u64::MAX);
    let bottom = SampleIndex::at(&SampleSession::seeded(0), 0);

    assert_eq!(top.seed(), u64::MAX);
    assert_eq!(top.index(), u64::MAX);
    assert_eq!(bottom.seed(), 0);
    assert_eq!(bottom.index(), 0);
    assert_ne!(top, bottom);
}

/// Addresses are hashable, so they can key a memo — and distinct addresses hash apart.
#[test]
fn addresses_are_hashable_and_distinct() {
    let session = SampleSession::seeded(3);
    let set: HashSet<SampleIndex> = SampleIndex::sequence(&session, 64).collect();
    assert_eq!(set.len(), 64, "64 addresses must be 64 distinct keys");
}

#[test]
fn an_address_is_copy_and_debuggable() {
    let at = SampleIndex::at(&SampleSession::seeded(1), 2);
    let copied = at;
    assert_eq!(at, copied);
    assert!(format!("{at:?}").contains("SampleIndex"));
}
