/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::RelationKind;
use ultragraph_traits::EdgeKey;

impl EdgeKey for RelationKind {
    type Key = RelationKind;

    fn edge_key(&self) -> Self::Key {
        *self
    }
}
