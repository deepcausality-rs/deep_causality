/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use ultragraph_traits::EdgeKey;

/// A payload keyed by a kind, carrying data the key ignores.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Kind {
    A,
    B,
}

#[derive(Clone, Debug, PartialEq)]
struct Edge {
    kind: Kind,
    weight: u32,
}

impl EdgeKey for Edge {
    type Key = Kind;

    fn edge_key(&self) -> Self::Key {
        self.kind
    }
}

/// A caller generic over the trait reads the key through the associated type.
fn key_of<T: EdgeKey>(payload: &T) -> T::Key {
    payload.edge_key()
}

#[test]
fn test_edge_key_returns_the_identifying_part() {
    let edge = Edge {
        kind: Kind::B,
        weight: 7,
    };
    assert_eq!(key_of(&edge), Kind::B);
}

#[test]
fn test_edge_key_ignores_the_data() {
    let light = Edge {
        kind: Kind::A,
        weight: 1,
    };
    let heavy = Edge {
        kind: Kind::A,
        weight: 1_000,
    };
    assert_eq!(light.edge_key(), heavy.edge_key());
    assert_ne!(light, heavy);
}

#[test]
fn test_keys_are_ordered() {
    // The bound `Key: Ord` is what a frozen graph sorts by; a caller can rely on it.
    fn sorted<T: EdgeKey>(a: &T, b: &T) -> bool {
        a.edge_key() <= b.edge_key()
    }
    let a = Edge {
        kind: Kind::A,
        weight: 9,
    };
    let b = Edge {
        kind: Kind::B,
        weight: 0,
    };
    assert!(sorted(&a, &b));
    assert!(!sorted(&b, &a));
}
