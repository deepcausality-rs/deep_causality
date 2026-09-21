/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::{Identifiable, IdentificationValue};

/// Stands in for a context-side implementor such as `Contextoid`.
struct ContextNode {
    id: IdentificationValue,
}

/// Stands in for a causal-side implementor such as `Causaloid`.
struct CausalUnit {
    id: IdentificationValue,
}

impl Identifiable for ContextNode {
    fn id(&self) -> IdentificationValue {
        self.id
    }
}

impl Identifiable for CausalUnit {
    fn id(&self) -> IdentificationValue {
        self.id
    }
}

/// The reason the trait lives in core: one bound accepts implementors from either layer.
fn read_id<T: Identifiable>(item: &T) -> IdentificationValue {
    item.id()
}

#[test]
fn test_identifiable_accepts_either_layer() {
    let context_side = ContextNode { id: 7 };
    let causal_side = CausalUnit { id: 42 };

    assert_eq!(read_id(&context_side), 7);
    assert_eq!(read_id(&causal_side), 42);
}

#[test]
fn test_identifiable_returns_identification_value() {
    let node = ContextNode {
        id: IdentificationValue::MAX,
    };

    let id: IdentificationValue = node.id();
    assert_eq!(id, u64::MAX);
}
