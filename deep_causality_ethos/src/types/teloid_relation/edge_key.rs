/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::TeloidRelation;
use ultragraph::EdgeKey;

// The relation is the payload's data, not its identity: a teloid pair carries one relation, so
// the key is `()` and a second relation on a pair replaces the first.
impl EdgeKey for TeloidRelation {
    type Key = ();

    fn edge_key(&self) -> Self::Key {}
}
