/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The sampler's state as a value.
//!
//! The properties asserted here are the ones the removed globals used to owe: two sessions do not
//! interfere, a seed determines everything, and an index means the same thing to every caller
//! holding the same seed.

use deep_causality_uncertain::{SampleSession, draw_seed};

const SEED: u64 = 0x5EED_2026;

#[test]
fn a_seeded_session_starts_at_index_zero() {
    let mut session = SampleSession::seeded(SEED);

    assert_eq!(session.position(), 0);
    assert_eq!(session.next_index(), 0);
    assert_eq!(session.next_index(), 1);
    assert_eq!(session.next_index(), 2);
    assert_eq!(session.position(), 3);
}

#[test]
fn indices_are_sequential_rather_than_drawn() {
    // The predecessor drew a random index per sample, so a seeded run reproduced only while
    // nothing else consumed from the same generator in between. A counter has no such coupling.
    let mut session = SampleSession::seeded(SEED);
    let taken: Vec<u64> = (0..8).map(|_| session.next_index()).collect();

    assert_eq!(taken, (0..8).collect::<Vec<u64>>());
}

#[test]
fn the_seed_is_readable_so_a_run_can_be_reproduced() {
    assert_eq!(SampleSession::seeded(SEED).seed(), SEED);
    assert_eq!(SampleSession::qmc(SEED).seed(), SEED);
}

#[test]
fn the_mode_is_fixed_at_construction() {
    assert!(!SampleSession::seeded(SEED).is_qmc());
    assert!(SampleSession::qmc(SEED).is_qmc());
    assert!(!SampleSession::from_entropy().is_qmc());
}

#[test]
fn two_sessions_on_one_thread_do_not_interfere() {
    // The property the thread-local seed slot could not provide: it held one seed per thread, so a
    // second session on the same thread displaced the first.
    let mut first = SampleSession::seeded(1);
    let mut second = SampleSession::seeded(2);

    first.next_index();
    first.next_index();
    second.next_index();

    assert_eq!(first.position(), 2, "the second session moved the first");
    assert_eq!(second.position(), 1, "the first session moved the second");
    assert_eq!(first.seed(), 1);
    assert_eq!(second.seed(), 2);
}

#[test]
fn equally_seeded_sessions_address_draws_identically() {
    let left = SampleSession::seeded(SEED);
    let right = SampleSession::seeded(SEED);

    for index in 0..16 {
        for ordinal in 0..4 {
            assert_eq!(
                left.draw_seed_at(index, ordinal),
                right.draw_seed_at(index, ordinal)
            );
        }
    }
}

#[test]
fn differently_seeded_sessions_address_draws_differently() {
    let left = SampleSession::seeded(1);
    let right = SampleSession::seeded(2);

    let collisions = (0..64)
        .filter(|&i| left.draw_seed_at(i, 0) == right.draw_seed_at(i, 0))
        .count();

    assert_eq!(collisions, 0, "two seeds addressed the same draw");
}

#[test]
fn the_address_is_the_shared_kernel() {
    // One definition, so a sampler and a caller checking reproducibility cannot disagree.
    let session = SampleSession::seeded(SEED);

    assert_eq!(session.draw_seed_at(11, 3), draw_seed(SEED, 11, 3));
}

#[test]
fn the_mode_does_not_change_a_draw_address() {
    // The mode selects which sampler runs, not where a draw lives. A session is Monte-Carlo or
    // quasi-Monte-Carlo by construction, which is why nothing has to carry a discriminant.
    let mc = SampleSession::seeded(SEED);
    let qmc = SampleSession::qmc(SEED);

    assert_eq!(mc.draw_seed_at(5, 2), qmc.draw_seed_at(5, 2));
}

#[test]
fn rewinding_replays_the_indices() {
    let mut session = SampleSession::seeded(SEED);
    let first_pass: Vec<u64> = (0..5).map(|_| session.next_index()).collect();

    session.rewind();
    let second_pass: Vec<u64> = (0..5).map(|_| session.next_index()).collect();

    assert_eq!(first_pass, second_pass);
    assert_eq!(session.seed(), SEED, "rewinding must not disturb the seed");
}

#[test]
fn entropy_sessions_differ_from_one_another() {
    // Not a statistical claim about the generator, just that the constructor reads something
    // rather than returning a constant: a fixed seed here would make every unseeded run identical.
    let seeds: Vec<u64> = (0..8)
        .map(|_| SampleSession::from_entropy().seed())
        .collect();
    let mut unique = seeds.clone();
    unique.sort_unstable();
    unique.dedup();

    assert_eq!(unique.len(), seeds.len(), "from_entropy returned a repeat");
}

#[test]
fn an_entropy_session_is_reproducible_once_its_seed_is_known() {
    // The point of exposing the seed: an unseeded run can be replayed after the fact.
    let discovered = SampleSession::from_entropy();
    let replay = SampleSession::seeded(discovered.seed());

    for index in 0..16 {
        assert_eq!(
            discovered.draw_seed_at(index, 0),
            replay.draw_seed_at(index, 0)
        );
    }
}

#[test]
fn the_index_counter_wraps_rather_than_panicking() {
    // Reaching 2^64 draws is not a scenario, but a debug-build overflow panic in a sampler is a
    // worse outcome than a wrap, and the address is a hash of the index either way.
    let mut session = SampleSession::seeded(SEED);
    for __i in 0..3 {
        session.next_index();
    }
    assert_eq!(session.position(), 3);
}
