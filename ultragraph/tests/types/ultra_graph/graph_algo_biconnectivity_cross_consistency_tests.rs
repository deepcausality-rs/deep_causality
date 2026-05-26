/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Cross-API consistency tests for the biconnectivity trio.
//!
//! Spec invariants (see `openspec/changes/add-biconnected-components`):
//!   I1. v is articulation iff v appears in >= 2 biconnected components.
//!   I2. {u, v} is a bridge iff some biconnected component has vertex set
//!       exactly {u, v}.

use std::collections::{HashMap, HashSet};
use ultragraph::{GraphMut, StructuralGraphAlgorithms, UltraGraphWeighted};

struct Fixture {
    name: &'static str,
    nodes: usize,
    edges: Vec<(usize, usize)>,
}

fn fixtures() -> Vec<Fixture> {
    vec![
        Fixture {
            name: "path-5",
            nodes: 5,
            edges: vec![(0, 1), (1, 2), (2, 3), (3, 4)],
        },
        Fixture {
            name: "triangle",
            nodes: 3,
            edges: vec![(0, 1), (1, 2), (2, 0)],
        },
        Fixture {
            name: "bow-tie",
            nodes: 5,
            edges: vec![(0, 1), (1, 2), (2, 0), (2, 3), (3, 4), (4, 2)],
        },
        Fixture {
            name: "two-cycles-bridged",
            nodes: 6,
            edges: vec![(0, 1), (1, 2), (2, 0), (2, 3), (3, 4), (4, 5), (5, 3)],
        },
        Fixture {
            name: "star",
            nodes: 5,
            edges: vec![(0, 1), (0, 2), (0, 3), (0, 4)],
        },
        Fixture {
            name: "tree",
            nodes: 4,
            edges: vec![(0, 1), (1, 2), (1, 3)],
        },
        Fixture {
            name: "k4",
            nodes: 4,
            edges: vec![(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)],
        },
        Fixture {
            name: "nested-cycles",
            nodes: 4,
            edges: vec![(0, 1), (1, 2), (2, 0), (1, 3), (2, 3)],
        },
        Fixture {
            name: "disconnected",
            nodes: 6,
            edges: vec![(0, 1), (1, 2), (2, 0), (3, 4)],
        },
        Fixture {
            name: "with-self-loops",
            nodes: 5,
            edges: vec![(0, 1), (1, 2), (2, 0), (2, 3), (3, 4), (4, 2), (2, 2)],
        },
        Fixture {
            name: "isolated-only",
            nodes: 5,
            edges: vec![],
        },
        Fixture {
            name: "empty",
            nodes: 0,
            edges: vec![],
        },
    ]
}

fn freeze(fix: &Fixture) -> UltraGraphWeighted<i32, ()> {
    let mut g = UltraGraphWeighted::<i32, ()>::new();
    for i in 0..fix.nodes {
        g.add_node(i as i32).unwrap();
    }
    for &(a, b) in &fix.edges {
        g.add_edge(a, b, ()).unwrap();
    }
    g.freeze();
    g
}

#[test]
fn articulation_iff_in_two_or_more_biconnected_components() {
    for fix in fixtures() {
        let g = freeze(&fix);
        let aps: HashSet<usize> = g.articulation_points().unwrap().into_iter().collect();
        let comps = g.biconnected_components().unwrap();

        let mut counts: HashMap<usize, usize> = HashMap::new();
        for comp in &comps {
            for &v in comp {
                *counts.entry(v).or_insert(0) += 1;
            }
        }
        let derived: HashSet<usize> = counts
            .into_iter()
            .filter_map(|(v, c)| if c >= 2 { Some(v) } else { None })
            .collect();
        assert_eq!(
            aps, derived,
            "fixture {}: articulation_points disagrees with biconnected_components",
            fix.name
        );
    }
}

#[test]
fn bridge_iff_two_vertex_biconnected_component() {
    for fix in fixtures() {
        let g = freeze(&fix);
        let bridges: HashSet<(usize, usize)> = g.bridges().unwrap().into_iter().collect();
        let comps = g.biconnected_components().unwrap();

        let derived: HashSet<(usize, usize)> = comps
            .iter()
            .filter(|c| c.len() == 2)
            .map(|c| (c[0], c[1])) // components are sorted ascending
            .collect();
        assert_eq!(
            bridges, derived,
            "fixture {}: bridges disagrees with biconnected_components",
            fix.name
        );
    }
}
