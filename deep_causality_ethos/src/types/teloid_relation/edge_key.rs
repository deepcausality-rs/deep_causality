/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::TeloidRelation;
use ultragraph::EdgeKey;

// The relation is the payload's data, not its identity: the key is `()`, so a teloid pair
// carries one relation. `TeloidGraph` refuses the other relation on a pair that already
// carries one.
impl EdgeKey for TeloidRelation {
    type Key = ();

    fn edge_key(&self) -> Self::Key {}
}
