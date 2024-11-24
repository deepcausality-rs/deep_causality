// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use dcl_data_structures::ring_buffer::prelude::*;

#[test]
fn test_min_cursor_sequence_empty() {
    let sequences: Vec<AtomicSequence> = vec![];
    assert_eq!(min_cursor_sequence(&sequences), 0);
}

#[test]
fn test_min_cursor_sequence_single() {
    let sequences = vec![AtomicSequence::from(42)];
    assert_eq!(min_cursor_sequence(&sequences), 42);
}

#[test]
fn test_min_cursor_sequence_multiple() {
    let sequences = vec![
        AtomicSequence::from(10),
        AtomicSequence::from(5),
        AtomicSequence::from(15),
    ];
    assert_eq!(min_cursor_sequence(&sequences), 5);
}

#[test]
fn test_min_cursor_sequence_same_values() {
    let sequences = vec![
        AtomicSequence::from(7),
        AtomicSequence::from(7),
        AtomicSequence::from(7),
    ];
    assert_eq!(min_cursor_sequence(&sequences), 7);
}

#[test]
fn test_min_cursor_sequence_zero() {
    let sequences = vec![
        AtomicSequence::from(0),
        AtomicSequence::from(1),
        AtomicSequence::from(2),
    ];
    assert_eq!(min_cursor_sequence(&sequences), 0);
}
