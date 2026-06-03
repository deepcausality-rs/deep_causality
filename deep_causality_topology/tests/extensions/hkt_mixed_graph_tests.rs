/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{CoMonad, Functor};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{MixedGraph, MixedGraphWitness};

fn graph_i32(values: Vec<i32>) -> MixedGraph<i32> {
    let n = values.len();
    let data = CausalTensor::new(values, vec![n]).unwrap();
    MixedGraph::new(n, data, 0).unwrap()
}

#[test]
fn functor_maps_payload_and_preserves_structure() {
    let mut g = graph_i32(vec![1, 2, 3]);
    g.add_arc(0, 1).unwrap();
    g.add_undirected(1, 2).unwrap();

    let mapped = MixedGraphWitness::fmap(g, |x| x * 10);
    assert_eq!(mapped.data().as_slice(), &[10, 20, 30]);
    // Edge structure is carried over unchanged.
    assert_eq!(mapped.num_edges(), 2);
    assert!(mapped.is_adjacent(0, 1));
    assert!(mapped.is_adjacent(1, 2));
}

#[test]
fn extract_returns_focused_payload() {
    let data = CausalTensor::new(vec![10, 20, 30], vec![3]).unwrap();
    let g = MixedGraph::new(3, data, 2).unwrap();
    assert_eq!(MixedGraphWitness::extract(&g), 30);
}

#[test]
fn extend_maps_neighborhood_aware_function() {
    // Sum of directed-arc parents' payloads at each focus.
    let mut g = graph_i32(vec![5, 7, 11]);
    g.add_arc(0, 1).unwrap(); // 0 → 1
    g.add_arc(0, 2).unwrap(); // 0 → 2

    let extended = MixedGraphWitness::extend(&g, |w: &MixedGraph<i32>| {
        let v = w.cursor();
        w.parents(v)
            .iter()
            .map(|&p| w.data().as_slice()[p])
            .sum::<i32>()
    });
    // node 0: no parents -> 0; node 1: parent {0}=5; node 2: parent {0}=5
    assert_eq!(extended.data().as_slice(), &[0, 5, 5]);
    assert_eq!(extended.cursor(), 0);
    assert_eq!(extended.num_edges(), 2);
}

#[test]
fn comonad_right_identity_extract_after_extend() {
    // extract(extend(w, f)) == f(w)  — the focus is preserved by extend.
    let mut g = graph_i32(vec![3, 4, 5]);
    g.add_arc(1, 0).unwrap();
    let f = |w: &MixedGraph<i32>| w.data().as_slice()[w.cursor()] + 100;
    let extended = MixedGraphWitness::extend(&g, f);
    // focus stays at 0; extract reads node 0's value = f(focus 0) = 3 + 100
    assert_eq!(extended.cursor(), 0);
    assert_eq!(MixedGraphWitness::extract(&extended), 103);
}

#[test]
fn comonad_right_identity_holds_for_nonzero_focus() {
    // extract(extend(w, f)) == f(w) must hold when the focus is not node 0.
    // This is the property that the cursor-preserving `extend` guarantees.
    let data = CausalTensor::new(vec![3, 4, 5], vec![3]).unwrap();
    let g = MixedGraph::new(3, data, 2).unwrap();
    let f = |w: &MixedGraph<i32>| w.data().as_slice()[w.cursor()] + 100;
    let extended = MixedGraphWitness::extend(&g, f);
    assert_eq!(extended.cursor(), 2);
    // f(w) reads the focused node (2) => 5 + 100; extract must reproduce it.
    assert_eq!(MixedGraphWitness::extract(&extended), 105);
}

#[test]
fn comonad_associativity_law() {
    // extend(extend(w, g), f) == extend(w, |w'| f(&extend(w', g)))
    let mut w = graph_i32(vec![5, 7, 11]);
    w.add_arc(0, 1).unwrap();
    w.add_arc(0, 2).unwrap();

    // g: sum of the focused node's directed-arc parents' payloads.
    let g = |w: &MixedGraph<i32>| {
        let v = w.cursor();
        w.parents(v)
            .iter()
            .map(|&p| w.data().as_slice()[p])
            .sum::<i32>()
    };
    // f: focused payload plus its first child's payload (0 if no child).
    let f = |w: &MixedGraph<i32>| {
        let v = w.cursor();
        let here = w.data().as_slice()[v];
        let child = w.children(v).first().map_or(0, |&c| w.data().as_slice()[c]);
        here + child
    };

    let lhs = MixedGraphWitness::extend(&MixedGraphWitness::extend(&w, g), f);
    let rhs = MixedGraphWitness::extend(&w, |wp: &MixedGraph<i32>| {
        f(&MixedGraphWitness::extend(wp, g))
    });

    assert_eq!(lhs.data().as_slice(), rhs.data().as_slice());
}

#[test]
fn comonad_duplicate_focuses_each_position() {
    // duplicate(w) yields a graph whose payload at p is `w` focused at p.
    let mut w = graph_i32(vec![2, 9, 4]);
    w.add_undirected(0, 1).unwrap();

    let dup: MixedGraph<MixedGraph<i32>> = MixedGraphWitness::duplicate(&w);
    assert_eq!(dup.num_vertices(), 3);
    for p in 0..3 {
        let inner = &dup.data().as_slice()[p];
        // Each inner copy is focused at its own position and carries `w`'s payload.
        assert_eq!(inner.cursor(), p);
        assert_eq!(inner.data().as_slice(), w.data().as_slice());
        // extract on the inner copy returns the payload at p.
        assert_eq!(MixedGraphWitness::extract(inner), w.data().as_slice()[p]);
    }
}

#[test]
fn comonad_left_identity_extend_extract_is_identity() {
    // extend(w, extract) reproduces the original payload at every node.
    let mut g = graph_i32(vec![2, 9, 4]);
    g.add_undirected(0, 1).unwrap();
    let rebuilt =
        MixedGraphWitness::extend(&g, |w: &MixedGraph<i32>| MixedGraphWitness::extract(w));
    assert_eq!(rebuilt.data().as_slice(), g.data().as_slice());
}
