/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{CoMonad, Functor};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::utils_tests::create_triangle_complex;
use deep_causality_topology::{Topology, TopologyWitness};
use std::sync::Arc;

#[test]
fn test_topology_functor() {
    let complex = Arc::new(create_triangle_complex());
    let data = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let topology = Topology::new(complex, 0, data, 0).unwrap();

    let mapped = TopologyWitness::fmap(topology, |x| x * 10.0);

    assert_eq!(mapped.data().as_slice(), &[10.0, 20.0, 30.0]);
}

#[test]
fn test_topology_extract() {
    let complex = Arc::new(create_triangle_complex());
    let data = CausalTensor::new(vec![10.0, 20.0, 30.0], vec![3]).unwrap();
    let topology = Topology::new(complex, 0, data, 2).unwrap(); // Cursor at 2

    let val = TopologyWitness::extract(&topology);
    assert_eq!(val, 30.0);
}

#[test]
fn test_topology_extend() {
    let complex = Arc::new(create_triangle_complex());
    let data = CausalTensor::new(vec![1.0, 1.0, 1.0], vec![3]).unwrap();
    let topology = Topology::new(complex, 0, data, 0).unwrap();

    // Extend: Value + Cursor
    let extended = TopologyWitness::extend(&topology, |w| {
        let val = TopologyWitness::extract(w);
        val + (w.cursor() as f64)
    });

    assert_eq!(extended.data().as_slice(), &[1.0, 2.0, 3.0]);
}
