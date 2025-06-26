/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use ultragraph::prelude::*;

use crate::benchmarks::data::Data;

pub fn build_linear_graph(k: usize) -> UltraGraph<Data> {
    let mut g = ultragraph::new_with_matrix_storage::<Data>(k);
    let d = Data::default();

    let root_index = g.add_root_node(d);
    let mut previous_idx = root_index;

    for _ in 0..k {
        // add a new causaloid and set current idx to it
        let data = Data::default();
        let current_idx = g.add_node(data);

        // link current causaloid to previos causaloid
        g.add_edge(previous_idx, current_idx)
            .expect("Failed to add edge");

        previous_idx = current_idx;
    }

    g
}
