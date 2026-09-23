/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! A replaying stream pins the trait's shape: items in order, a cursor after each, then `None`
//! for good. Expected values are the literals the tape is built from.
//!
//! Corner cases (rows A to K): A an empty tape in `test_an_empty_stream_ends_at_once`; B a
//! one-event tape in the same test; D the position past the last event, polled twice, in
//! `test_a_stream_yields_in_order_then_ends`; every other row n/a.
use core::future::{Future, ready};
use deep_causality_context_store::utils_test::block_on;
use deep_causality_context_store::{ContextEvent, ContextEventItem, ContextEvents};
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq)]
struct ReplayError;

impl Display for ReplayError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ReplayError")
    }
}

/// A stream over a fixed list of events; the cursor is the position after each.
struct Replay {
    events: Vec<ContextEvent>,
    position: usize,
}

impl ContextEvents for Replay {
    type Error = ReplayError;
    type Cursor = usize;

    fn next(&mut self) -> impl Future<Output = ContextEventItem<Self::Cursor, Self::Error>> + Send {
        let item = self.events.get(self.position).cloned().map(|event| {
            self.position += 1;
            Ok((self.position, event))
        });
        ready(item)
    }
}

#[test]
fn test_a_stream_yields_in_order_then_ends() {
    let mut stream = Replay {
        events: vec![
            ContextEvent::NodeRetracted(1),
            ContextEvent::ContextRetracted(2),
        ],
        position: 0,
    };
    assert_eq!(
        block_on(stream.next()),
        Some(Ok((1, ContextEvent::NodeRetracted(1))))
    );
    assert_eq!(
        block_on(stream.next()),
        Some(Ok((2, ContextEvent::ContextRetracted(2))))
    );
    assert_eq!(block_on(stream.next()), None);
    assert_eq!(block_on(stream.next()), None);
    assert_eq!(ReplayError.to_string(), "ReplayError");
}

#[test]
fn test_an_empty_stream_ends_at_once() {
    let mut empty = Replay {
        events: vec![],
        position: 0,
    };
    assert_eq!(block_on(empty.next()), None);
    let mut one = Replay {
        events: vec![ContextEvent::NodeRetracted(4)],
        position: 0,
    };
    assert_eq!(
        block_on(one.next()),
        Some(Ok((1, ContextEvent::NodeRetracted(4))))
    );
    assert_eq!(block_on(one.next()), None);
}
