/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::ContextEvent;

/// One item of a stream: the cursor after an event and the event, the error that ended the stream,
/// or `None` when it has ended.
pub type ContextEventItem<Cursor, Error> = Option<Result<(Cursor, ContextEvent), Error>>;

/// A stream of events in the order a store committed them.
///
/// Declared here rather than taken from a crate, because the standard library has no stable stream
/// trait and this crate has no dependencies. It is the shape every stream trait reduces to: one
/// asynchronous `next`. A caller holding a stream crate wraps it in a few lines.
pub trait ContextEvents {
    type Error: core::fmt::Debug + core::fmt::Display;

    /// Where a subscription resumes from. Opaque to the caller.
    type Cursor: Clone + Send;

    /// The next event with the cursor after it, `None` when the stream has ended, or the error
    /// that ended it.
    fn next(&mut self) -> impl Future<Output = ContextEventItem<Self::Cursor, Self::Error>> + Send;
}
