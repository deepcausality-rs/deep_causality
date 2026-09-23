/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{ContextEvent, ContextEvents, ContextSnapshot, ContextStorage};

/// A store that also reports changes as they commit and accepts a host's changes in order.
///
/// Requires [`ContextStorage`]: a stream is a way of keeping a hydrated context current, and what
/// a context is comes from the storage trait. A backend that cannot stream implements the base
/// trait alone and is complete; nothing here has a default body that pretends otherwise.
///
/// # Scope of a subscription
///
/// A subscription delivers the changes to the container its slice names and to the containers
/// that container referenced when the subscription began, and nothing else, for the life of the
/// stream. A container created or attached afterwards is reached by a new `subscribe` whose slice
/// names it, resumed from the cursor the host has reached.
pub trait ContextStorageStream: ContextStorage {
    /// Where a subscription resumes from. Opaque to the caller.
    type Cursor: Clone + Send;

    /// The stream a subscription yields.
    type Events: ContextEvents<Cursor = Self::Cursor, Error = Self::Error> + Send;

    /// The context as of a consistent point, and every change to it from that point on. The two
    /// are one call so no change can fall between the snapshot and the first event. With a
    /// cursor, the snapshot is the context as of that cursor and the stream continues from it.
    fn subscribe(
        &self,
        spec: &Self::Slice,
        from: Option<Self::Cursor>,
    ) -> impl Future<Output = Result<(ContextSnapshot, Self::Events), Self::Error>> + Send;

    /// Performs the operation an event names, under the same refusals, and returns the cursor
    /// after it. Refuses the report-only variants `ContextCreated`, `NodeEntered` and `NodeLeft`.
    fn apply(
        &self,
        event: &ContextEvent,
    ) -> impl Future<Output = Result<Self::Cursor, Self::Error>> + Send;

    /// Performs the operations a sequence of events names, in order, as one transaction, and
    /// returns the cursor after them. Refused whole if any event would be.
    fn apply_batch(
        &self,
        events: &[ContextEvent],
    ) -> impl Future<Output = Result<Self::Cursor, Self::Error>> + Send;
}
