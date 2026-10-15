/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_ethos::TeloidRelation;
use ultragraph::EdgeKey;

// Compiles only when the key type is `()`.
fn unit_key<T: EdgeKey<Key = ()>>(payload: &T) {
    payload.edge_key()
}

#[test]
fn test_both_relations_share_the_unit_key() {
    unit_key(&TeloidRelation::Inherits);
    unit_key(&TeloidRelation::Defeats);
    let () = TeloidRelation::Inherits.edge_key();
    let () = TeloidRelation::Defeats.edge_key();
}
