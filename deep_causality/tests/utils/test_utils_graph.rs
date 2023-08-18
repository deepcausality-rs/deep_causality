// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality::prelude::{CausalityError, Causaloid, CausaloidGraph, Dataoid, IdentificationValue, NumericalValue, Spaceoid, SpaceTempoid, Tempoid};
use deep_causality::protocols::causable_graph::graph::CausableGraph;

const SMALL: usize = 9;
// const MEDIUM: usize = 1_00;
// const LARGE: usize = 1_000;

type CausalGraph<'l> = CausaloidGraph<Causaloid<'l, Dataoid, Spaceoid, Tempoid, SpaceTempoid>>;


fn get_test_causaloid<'l>()
    -> Causaloid<'l, Dataoid, Spaceoid, Tempoid, SpaceTempoid>
{
    let id: IdentificationValue = 1;
    let description = "tests whether data exceeds threshold of 0.55";
    fn causal_fn(obs: NumericalValue) -> Result<bool, CausalityError> {
        if obs.is_sign_negative() {
            return Err(CausalityError("Observation is negative".into()));
        }

        let threshold: NumericalValue = 0.55;
        if !obs.ge(&threshold) {
            Ok(false)
        } else {
            Ok(true)
        }
    }

    Causaloid::new(id, causal_fn, description)
}

pub fn get_small_linear_graph_and_data<'l>()
    -> (CausalGraph<'l>, [f64; SMALL + 1])
{ // Builds a linear graph: root -> a -> b -> c
    (build_linear_graph(SMALL), generate_sample_data())
}

pub fn build_linear_graph<'l>(
    k: usize
)
    -> CausalGraph<'l>
{   // Builds a linear graph: root -> a -> b -> c
    let mut g = CausaloidGraph::new();

    let root_causaloid = get_test_causaloid();
    let root_index = g.add_root_causaloid(root_causaloid);

    let mut previous_idx = root_index;

    for _ in 0..k {
        // add a new causaloid and set current idx to it
        let causaloid = get_test_causaloid();
        let current_idx = g.add_causaloid(causaloid);

        // link current causaloid to previos causaloid
        g.add_edge(previous_idx, current_idx).expect("Failed to add edge");

        previous_idx = current_idx;
    }

    g
}

pub fn get_small_multi_cause_graph_and_data<'l>()
    -> (CausalGraph<'l>, [f64; 4 + 1])
{   // Builds a multi-layer cause graph:
    (build_multi_cause_graph(), generate_sample_data())
}

fn build_multi_cause_graph<'l>()
    -> CausalGraph<'l>
{
    // Builds a multi cause graph:
    //  root
    //  / \
    //  A B
    //  \ /
    //   C

    let mut g = CausaloidGraph::new();

    // Add root causaloid
    let root_causaloid = get_test_causaloid();
    let root_index = g.add_root_causaloid(root_causaloid);

    // Add causaloid A
    let causaloid = get_test_causaloid();
    let idx_a = g.add_causaloid(causaloid);

    // Link causaloid A to root causaloid
    g.add_edge(root_index, idx_a).expect("Failed to add edge between root and A");

    // Add causaloid B
    let causaloid = get_test_causaloid();
    let idx_b = g.add_causaloid(causaloid);

    // Link causaloid B to root causaloid
    g.add_edge(root_index, idx_b).expect("Failed to add edge between root and B");

    // Add causaloid C
    let causaloid = get_test_causaloid();
    let idx_c = g.add_causaloid(causaloid);

    // Link causaloid C to A
    g.add_edge(idx_a, idx_c).expect("Failed to add edge between A and C");

    // Link causaloid C to B
    g.add_edge(idx_b, idx_c).expect("Failed to add edge between C and B");

    g
}

pub fn get_small_multi_layer_cause_graph_and_data<'l>()
    -> (CausalGraph<'l>, [f64; 8 + 1])
{   // Builds a multi-layer cause graph:
    (build_multi_layer_cause_graph(), generate_sample_data())
}

fn build_multi_layer_cause_graph<'l>()
    -> CausalGraph<'l>
{
    // Builds a multi-layer cause graph:
    //    root
    //  /   |  \
    //  A   B   C
    // /\  /\  /\
    //D   E   F  G

    let mut g = CausaloidGraph::new();

    // Add root causaloid
    let root_causaloid = get_test_causaloid();
    let root_index = g.add_root_causaloid(root_causaloid);

    // ### First layer ### //

    // Add causaloid A
    let causaloid = get_test_causaloid();
    let idx_a = g.add_causaloid(causaloid);
    // Link causaloid A to root causaloid
    g.add_edge(root_index, idx_a).expect("Failed to add edge between root and A");

    // Add causaloid B
    let causaloid = get_test_causaloid();
    let idx_b = g.add_causaloid(causaloid);
    // Link causaloid B to root causaloid
    g.add_edge(root_index, idx_b).expect("Failed to add edge between root and B");

    // Add causaloid C
    let causaloid = get_test_causaloid();
    let idx_c = g.add_causaloid(causaloid);
    // Link causaloid C  to root causaloid
    g.add_edge(root_index, idx_c).expect("Failed to add edge between root and C");

    // ### Second layer ### //

    // Add causaloid D
    let causaloid = get_test_causaloid();
    let idx_d = g.add_causaloid(causaloid);
    // Link causaloid D  to A
    g.add_edge(idx_a, idx_d).expect("Failed to add edge between D and A");

    // Add causaloid E
    let causaloid = get_test_causaloid();
    let idx_e = g.add_causaloid(causaloid);
    // Link causaloid E  to A
    g.add_edge(idx_a, idx_e).expect("Failed to add edge between A and E");
    // Link causaloid E  to B
    g.add_edge(idx_b, idx_e).expect("Failed to add edge between B and E");

    // Add causaloid F
    let causaloid = get_test_causaloid();
    let idx_f = g.add_causaloid(causaloid);
    // Link causaloid F  to B
    g.add_edge(idx_b, idx_f).expect("Failed to add edge between B and F");
    // Link causaloid F  to C
    g.add_edge(idx_c, idx_f).expect("Failed to add edge between C and F");

    // Add causaloid G
    let causaloid = get_test_causaloid();
    let idx_g = g.add_causaloid(causaloid);
    // Link causaloid G  to C
    g.add_edge(idx_c, idx_g).expect("Failed to add edge between A and C");

    g
}

pub fn get_left_imbalanced_cause_graph<'l>()
    -> (CausalGraph<'l>, [f64; 6 + 1])
{   // Builds a multi-layer cause graph:
    (build_left_imbalanced_cause_graph(), generate_sample_data())
}

fn build_left_imbalanced_cause_graph<'l>()
    -> CausaloidGraph<Causaloid<'l, Dataoid, Spaceoid, Tempoid, SpaceTempoid>>
{
    // Builds a multi-layer cause graph:
    //    root
    //  /   |  \
    //  A   B   C
    // /\
    //D  E

    let mut g = CausaloidGraph::new();

    // Add root causaloid
    let root_causaloid = get_test_causaloid();
    let root_index = g.add_root_causaloid(root_causaloid);

    // ### First layer ### //

    // Add causaloid A
    let causaloid = get_test_causaloid();
    let idx_a = g.add_causaloid(causaloid);
    // Link causaloid A to root causaloid
    g.add_edge(root_index, idx_a).expect("Failed to add edge between root and A");

    // Add causaloid B
    let causaloid = get_test_causaloid();
    let idx_b = g.add_causaloid(causaloid);
    // Link causaloid B to root causaloid
    g.add_edge(root_index, idx_b).expect("Failed to add edge between root and B");

    // Add causaloid C
    let causaloid = get_test_causaloid();
    let idx_c = g.add_causaloid(causaloid);
    // Link causaloid C  to root causaloid
    g.add_edge(root_index, idx_c).expect("Failed to add edge between root and C");

    // ### Second layer ### //

    // Add causaloid D
    let causaloid = get_test_causaloid();
    let idx_d = g.add_causaloid(causaloid);
    // Link causaloid D  to A
    g.add_edge(idx_a, idx_d).expect("Failed to add edge between A and E");

    // Add causaloid E
    let causaloid = get_test_causaloid();
    let idx_e = g.add_causaloid(causaloid);
    // Link causaloid E  to A
    g.add_edge(idx_a, idx_e).expect("Failed to add edge between A and B");

    g
}

pub fn get_right_imbalanced_cause_graph<'l>()
    -> (CausalGraph<'l>, [f64; 6 + 1])
{   // Builds a multi-layer cause graph:
    (build_right_imbalanced_cause_graph(), generate_sample_data())
}

fn build_right_imbalanced_cause_graph<'l>()
    -> CausalGraph<'l>
{
    // Builds a multi-layer cause graph:
    //    root
    //  /   |  \
    //  A   B   C
    //          /\
    //         D  E

    let mut g = CausaloidGraph::new();

    // Add root causaloid
    let root_causaloid = get_test_causaloid();
    let root_index = g.add_root_causaloid(root_causaloid);

    // ### First layer ### //

    // Add causaloid A
    let causaloid = get_test_causaloid();
    let idx_a = g.add_causaloid(causaloid);
    // Link causaloid A to root causaloid
    g.add_edge(root_index, idx_a).expect("Failed to add edge between rootCause and A");

    // Add causaloid B
    let causaloid = get_test_causaloid();
    let idx_b = g.add_causaloid(causaloid);
    // Link causaloid B to root causaloid
    g.add_edge(root_index, idx_b).expect("Failed to add edge between rootCause and B");

    // Add causaloid C
    let causaloid = get_test_causaloid();
    let idx_c = g.add_causaloid(causaloid);
    // Link causaloid C  to root causaloid
    g.add_edge(root_index, idx_c).expect("Failed to add edge between root and C");

    // ### Second layer ### //

    // Add causaloid D
    let causaloid = get_test_causaloid();
    let idx_d = g.add_causaloid(causaloid);
    // Link causaloid D  to C
    g.add_edge(idx_c, idx_d).expect("Failed to add edge between D to C");

    // Add causaloid E
    let causaloid = get_test_causaloid();
    let idx_e = g.add_causaloid(causaloid);
    // Link causaloid E  to C
    g.add_edge(idx_c, idx_e).expect("Failed to add edge between c and e");

    g
}

// Generates a fixed sized array with sample data
pub fn generate_sample_data<const N: usize>()
    -> [f64; N]
{
    [0.99; N]
}
