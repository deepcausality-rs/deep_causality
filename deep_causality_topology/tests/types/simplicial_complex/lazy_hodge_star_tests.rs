/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the lazy lumped-mass Hodge ⋆ population.

use deep_causality_topology::{Simplex, SimplicialComplex, Skeleton};

#[test]
fn test_lumped_mass_gives_a_vertexless_cell_zero_volume() {
    // A simplex with no vertices spans nothing, so its Cayley-Menger volume is
    // zero. Its lumped-mass dual volume is then zero as well, and a zero is not
    // stored in the sparse diagonal.
    let skeletons = vec![Skeleton::new(0, vec![Simplex::new(vec![])])];
    let complex: SimplicialComplex<f64> =
        SimplicialComplex::with_geometry(skeletons, Vec::new(), Vec::new(), Vec::new(), 2);

    let ops = complex
        .hodge_star_operators()
        .expect("a zero-dimensional complex has no top-volume degeneracy to reject");

    assert_eq!(ops.len(), 1);
    assert_eq!(ops[0].shape(), (1, 1));
    assert!(
        ops[0].values().is_empty(),
        "a cell of zero volume carries no lumped mass"
    );
}

#[test]
fn test_lumped_mass_vertex_dual_volume_is_the_incident_edge_length_halved() {
    // A single edge of length 3 between two vertices. Each endpoint's dual
    // volume is the incident primal volume divided by max_dim + 1 = 2, so both
    // vertex entries are 1.5, and the top-grade entry is 1 / 3.
    let skeletons = vec![
        Skeleton::new(0, vec![Simplex::new(vec![0]), Simplex::new(vec![1])]),
        Skeleton::new(1, vec![Simplex::new(vec![0, 1])]),
    ];
    let coords = vec![0.0, 0.0, 3.0, 0.0];
    let complex: SimplicialComplex<f64> =
        SimplicialComplex::with_geometry(skeletons, Vec::new(), Vec::new(), coords, 2);

    let ops = complex.hodge_star_operators().expect("edge length 3 > 0");

    assert_eq!(ops.len(), 2);
    assert_eq!(ops[0].shape(), (2, 2));
    assert_eq!(ops[0].values(), &[1.5, 1.5]);
    assert_eq!(ops[1].shape(), (1, 1));
    assert!((ops[1].values()[0] - 1.0 / 3.0).abs() < 1e-12);
}
