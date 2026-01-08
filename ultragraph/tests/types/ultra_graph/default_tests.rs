/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use ultragraph::*;

#[test]
fn test_graph_state_default() {
    let gs = GraphState::<(), ()>::default();
    if let GraphState::Dynamic(g) = gs {
        assert!(g.is_empty());
        assert_eq!(g.number_nodes(), 0);
        assert_eq!(g.number_edges(), 0);
    } else {
        panic!("Default GraphState should be Dynamic");
    }
}
