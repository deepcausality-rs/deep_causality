/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! A tape backend pins the streaming trait's shape through the supertrait. Expected values are
//! the literals the tape returns.
//!
//! Corner cases (rows A to K): A an empty batch in `test_apply_and_apply_batch_return_a_cursor`;
//! every other row is the in-memory backend's (group 5) and is n/a for a shape test.
use core::future::{Future, ready};
use deep_causality_context_store::utils_test::block_on;
use deep_causality_context_store::{
    ContextEvent, ContextEventItem, ContextEvents, ContextId, ContextRecord, ContextSnapshot,
    ContextStorage, ContextStorageStream, ContextoidId, ContextoidRecord, IdReserve,
    RelationRecord,
};
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq)]
struct TapeError;

impl Display for TapeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "TapeError")
    }
}

struct TapeEvents {
    left: Vec<ContextEvent>,
    cursor: usize,
}

impl ContextEvents for TapeEvents {
    type Error = TapeError;
    type Cursor = usize;

    fn next(&mut self) -> impl Future<Output = ContextEventItem<Self::Cursor, Self::Error>> + Send {
        let item = self.left.pop().map(|event| {
            self.cursor += 1;
            Ok((self.cursor, event))
        });
        ready(item)
    }
}

/// A backend whose stream replays a fixed tape and whose writes count the events applied.
struct Tape;

impl ContextStorage for Tape {
    type Error = TapeError;
    type Slice = ContextId;

    fn reserve(&self, n: usize) -> impl Future<Output = Result<IdReserve, Self::Error>> + Send {
        ready(Ok(IdReserve::new((1..=n as ContextoidId).collect())))
    }
    fn create_context(
        &self,
        _name: &str,
    ) -> impl Future<Output = Result<ContextId, Self::Error>> + Send {
        ready(Ok(1))
    }
    fn retract_context(
        &self,
        _context: ContextId,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        ready(Ok(()))
    }
    fn create_node(
        &self,
        _nodes: &[ContextoidRecord],
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        ready(Ok(()))
    }
    fn retract_node(
        &self,
        _node: ContextoidId,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        ready(Ok(()))
    }
    fn create_edge(
        &self,
        _edges: &[RelationRecord],
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        ready(Ok(()))
    }
    fn retract_edge(
        &self,
        _from: ContextoidId,
        _to: ContextoidId,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        ready(Ok(()))
    }
    fn link(
        &self,
        _context: ContextId,
        _nodes: &[ContextoidId],
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        ready(Ok(()))
    }
    fn unlink(
        &self,
        _context: ContextId,
        _nodes: &[ContextoidId],
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        ready(Ok(()))
    }
    fn attach(
        &self,
        _context: ContextId,
        _extra: ContextId,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        ready(Ok(()))
    }
    fn detach(
        &self,
        _context: ContextId,
        _extra: ContextId,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        ready(Ok(()))
    }
    fn lookup(
        &self,
        ids: &[ContextoidId],
    ) -> impl Future<Output = Result<Vec<Option<ContextoidRecord>>, Self::Error>> + Send {
        ready(Ok(vec![None; ids.len()]))
    }
    fn hydrate(
        &self,
        spec: &Self::Slice,
    ) -> impl Future<Output = Result<ContextSnapshot, Self::Error>> + Send {
        ready(Ok(ContextSnapshot::new(
            ContextRecord::new(*spec, "tape".to_string()),
            vec![],
            vec![],
            vec![],
        )))
    }
}

impl ContextStorageStream for Tape {
    type Cursor = usize;
    type Events = TapeEvents;

    fn subscribe(
        &self,
        spec: &Self::Slice,
        from: Option<Self::Cursor>,
    ) -> impl Future<Output = Result<(ContextSnapshot, Self::Events), Self::Error>> + Send {
        let snapshot = ContextSnapshot::new(
            ContextRecord::new(*spec, "tape".to_string()),
            vec![],
            vec![],
            vec![],
        );
        let events = TapeEvents {
            left: vec![ContextEvent::ContextRetracted(*spec)],
            cursor: from.unwrap_or(0),
        };
        ready(Ok((snapshot, events)))
    }

    fn apply(
        &self,
        _event: &ContextEvent,
    ) -> impl Future<Output = Result<Self::Cursor, Self::Error>> + Send {
        ready(Ok(1))
    }

    fn apply_batch(
        &self,
        events: &[ContextEvent],
    ) -> impl Future<Output = Result<Self::Cursor, Self::Error>> + Send {
        ready(Ok(events.len()))
    }
}

fn assert_send<F: Send>(_: &F) {}

/// Compiles only through the supertrait: a streaming backend is also a storage, so both a
/// streaming operation and a storage operation are reachable from one bound.
fn through_both_traits<S: ContextStorageStream + Sync>(storage: &S) -> (S::Cursor, usize)
where
    S::Slice: Sync,
{
    assert_send(&storage.apply(&ContextEvent::NodeRetracted(1)));
    assert_send(&storage.apply_batch(&[]));
    let cursor = block_on(storage.apply_batch(&[
        ContextEvent::NodeRetracted(1),
        ContextEvent::NodeRetracted(2),
    ]))
    .unwrap();
    let reserved = block_on(storage.reserve(3)).unwrap().remaining();
    (cursor, reserved)
}

#[test]
fn test_apply_and_apply_batch_return_a_cursor() {
    assert_eq!(block_on(Tape.apply(&ContextEvent::NodeRetracted(1))), Ok(1));
    assert_eq!(block_on(Tape.apply_batch(&[])), Ok(0));
    assert_eq!(through_both_traits(&Tape), (2, 3));
}

#[test]
fn test_subscribe_returns_snapshot_and_stream() {
    let (snapshot, mut events) = block_on(Tape.subscribe(&5, Some(3))).unwrap();
    assert_eq!(snapshot.context().id(), 5);
    assert_eq!(snapshot.context().name(), "tape");
    let first = events.next();
    assert_send(&first);
    assert_eq!(
        block_on(first),
        Some(Ok((4, ContextEvent::ContextRetracted(5))))
    );
    assert_eq!(block_on(events.next()), None);
    assert_eq!(TapeError.to_string(), "TapeError");
}
