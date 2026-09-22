/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{ContextId, ContextRecord, ContextoidId, ContextoidRecord, RelationRecord};

/// One change to a store. The same type flows both ways: a store reports it, and a host requests
/// it.
///
/// Each variant is one storage operation by the same name, except `NodeEntered` and `NodeLeft`,
/// which are a view's answer moving with no operation behind it. A membership event names the
/// context it concerns and, when it adds a node, carries the node's record, so a subscriber
/// applies it to a hydrated context without having seen any earlier event. The event carries no
/// time: the order of events is the stream's cursor.
#[derive(Debug, Clone, PartialEq)]
pub enum ContextEvent {
    ContextCreated(ContextRecord),
    ContextRetracted(ContextId),
    NodeCreated(ContextoidRecord),
    NodeRetracted(ContextoidId),
    EdgeCreated(RelationRecord),
    EdgeRetracted {
        from: ContextoidId,
        to: ContextoidId,
    },
    NodeLinked {
        context: ContextId,
        node: ContextoidRecord,
    },
    NodeUnlinked {
        context: ContextId,
        node: ContextoidId,
    },
    ContextAttached {
        context: ContextId,
        extra: ContextRecord,
    },
    ContextDetached {
        context: ContextId,
        extra: ContextId,
    },
    /// A node a view now holds because a rule began reading it. The node itself is unchanged.
    NodeEntered {
        context: ContextId,
        node: ContextoidRecord,
    },
    /// A node a view no longer holds because no rule reads it. The node itself is unchanged.
    NodeLeft {
        context: ContextId,
        node: ContextoidId,
    },
}
