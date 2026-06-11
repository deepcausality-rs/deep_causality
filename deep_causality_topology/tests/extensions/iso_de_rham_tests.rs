/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Law test for the `DeRhamSharpIso` Tier-2 witness (`extensions::iso_de_rham`):
//! the exact round-trip law holds on the constant-field carrier, where the
//! de Rham/sharp pair is exactly inverse. Order-based round-trip tests for
//! smooth fields live with the operator tests in
//! `tests/types/manifold/de_rham_tests.rs`.

use deep_causality_num::iso::witness::test_support::assert_witness_iso_round_trip;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    ChainComplex, CubicalReggeGeometry, DeRhamSharpIso, LatticeComplex, Manifold,
};

fn unit_manifold(lattice: LatticeComplex<2, f64>) -> Manifold<LatticeComplex<2, f64>, f64> {
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let metric: CubicalReggeGeometry<2, f64> = CubicalReggeGeometry::unit();
    Manifold::from_cubical_with_metric(lattice, data, metric, 0)
}

#[test]
fn iso_witness_round_trip_law_on_constants() {
    // The Tier-2 witness satisfies the exact round-trip law on the
    // constant-field carrier (where ♭ and ♯ are exactly inverse).
    let manifold = unit_manifold(LatticeComplex::<2, f64>::square_torus(3));
    let complex = manifold.complex();
    let n0 = complex.num_cells(0);
    let n1 = complex.num_cells(1);

    let vertex_side = CausalTensor::new(vec![2.5; 2 * n0], vec![2 * n0]).unwrap();
    let edge_side = CausalTensor::new(vec![2.5; n1], vec![n1]).unwrap();

    assert_witness_iso_round_trip::<DeRhamSharpIso<2, f64>, _, _>(
        (manifold.clone(), vertex_side),
        (manifold, edge_side),
    );
}
