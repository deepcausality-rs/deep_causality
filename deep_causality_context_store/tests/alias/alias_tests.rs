/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Expected values are the literals passed through; the width is compared to `u64` by
//! `size_of`, which the alias declaration pins.
//!
//! Corner cases (rows A to K): all n/a (an alias has no inputs).
use deep_causality_context_store::{ContextId, ContextoidId, IdentificationValue};

fn takes_context_id(id: ContextId) -> IdentificationValue {
    id
}

fn takes_contextoid_id(id: ContextoidId) -> IdentificationValue {
    id
}

#[test]
fn test_aliases_share_one_width() {
    let width: IdentificationValue = 7;
    assert_eq!(takes_context_id(width), 7);
    assert_eq!(takes_contextoid_id(width), 7);
    assert_eq!(size_of::<ContextId>(), size_of::<u64>());
    assert_eq!(size_of::<ContextoidId>(), size_of::<u64>());
}
