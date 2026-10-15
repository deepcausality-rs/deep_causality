/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
// `ultragraph::EdgeKey` is a re-export of `ultragraph_traits::EdgeKey`, so an impl written
// against the one satisfies a bound written against the other. Both crates are named here on
// purpose: this is the seam a payload crate relies on.

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Kind(u8);

#[derive(Clone, Debug, Default)]
struct Payload(u8);

impl ultragraph_traits::EdgeKey for Payload {
    type Key = Kind;

    fn edge_key(&self) -> Self::Key {
        Kind(self.0)
    }
}

fn through_the_graph_crate<T: ultragraph::EdgeKey>(payload: &T) -> T::Key {
    payload.edge_key()
}

#[test]
fn test_one_trait_under_two_paths() {
    assert_eq!(through_the_graph_crate(&Payload(3)), Kind(3));
}
