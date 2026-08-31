/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `Topology::cup_product` delegates, and these tests pin what the delegation owes.
//!
//! This crate used to compute the cup product twice: the free [`cup_product`], generic over
//! `K: CellularComplex` with `K::CellType: SplittableCell`, and a second, simplicial-only
//! implementation on `Topology` extracting the Alexander–Whitney front and back faces by hand.
//! Neither called the other. These tests were written to pin that the two agreed, and having
//! established it, the duplicate was retired: the method now delegates.
//!
//! **So the first test below is no longer an agreement test.** With one implementation left, it
//! checks the wrapping the method adds around the shared body: the result's grade, its length, and
//! its values. It is kept because the wrapping is real code and because a regression there would
//! otherwise be silent.
//!
//! The tests after it cover the contracts that *changed* when the method began delegating. The
//! grade tightening is here; the tolerance loosening is in
//! `tests/types/topology/topology_tests.rs`, where the complex it needs is already built.
//!
//! A third delta was expected and turned out not to exist. The retired implementation reached
//! `.expect("Data/Skeleton mismatch")` on a cochain whose length did not match its skeleton, but
//! `Topology::new` validates exactly that, so the panic was unreachable through the public
//! constructor and there is no behaviour to pin.

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::utils_tests::{pseudo_cochain, tetrahedron};
use deep_causality_topology::{ChainComplex, Cochain, Topology, TopologyErrorEnum, cup_product};
use std::sync::Arc;

/// Builds a `Topology` cochain of `degree` over `complex`.
fn topology_cochain(
    complex: &Arc<deep_causality_topology::SimplicialComplex<f64>>,
    degree: usize,
    values: Vec<f64>,
) -> Topology<f64, f64> {
    let len = values.len();
    Topology::new(
        complex.clone(),
        degree,
        CausalTensor::new(values, vec![len]).unwrap(),
        0,
    )
    .unwrap()
}

/// Every degree pair a tetrahedron admits, both ways round.
#[test]
fn test_the_method_wraps_the_shared_body_faithfully() {
    let complex = Arc::new(tetrahedron());

    // All ten admissible pairs, not the six this test covered while it was pinning two
    // implementations against each other. A tetrahedron has max_dim 3, so every pair summing to
    // three or less is in range.
    for (p, q) in [
        (0usize, 0usize),
        (0, 1),
        (1, 0),
        (1, 1),
        (0, 2),
        (2, 0),
        (0, 3),
        (3, 0),
        (1, 2),
        (2, 1),
    ] {
        let np = complex.num_cells(p);
        let nq = complex.num_cells(q);
        assert!(np > 0 && nq > 0, "degree {p} or {q} has no cells");

        let a = pseudo_cochain(np, 11 + p as u64);
        let b = pseudo_cochain(nq, 23 + q as u64);

        let free = cup_product(
            complex.as_ref(),
            &Cochain::from_values(&a, p),
            &Cochain::from_values(&b, q),
        )
        .unwrap_or_else(|e| panic!("free cup_product at ({p}, {q}): {e}"));

        let method = topology_cochain(&complex, p, a)
            .cup_product(&topology_cochain(&complex, q, b))
            .unwrap_or_else(|e| panic!("Topology::cup_product at ({p}, {q}): {e}"));

        assert_eq!(method.grade(), p + q, "the result must have grade p + q");
        assert_eq!(
            free.degree(),
            p + q,
            "the free form must agree on the grade"
        );
        assert_eq!(
            free.values(),
            method.data().as_slice(),
            "the method's wrapping altered the values at ({p}, {q})"
        );
    }
}

#[test]
fn test_an_out_of_range_grade_now_errors_instead_of_returning_zeros() {
    // The delegation's first behaviour change. The retired implementation returned `Ok` with a
    // zero-filled cochain when p + q exceeded the complex's dimension, reporting a cochain in a
    // degree the complex does not have as a successful computation. A tetrahedron has max_dim 3,
    // so 2 + 2 is out of range.
    let complex = Arc::new(tetrahedron());
    let n2 = complex.num_cells(2);
    let a = topology_cochain(&complex, 2, pseudo_cochain(n2, 5));
    let b = topology_cochain(&complex, 2, pseudo_cochain(n2, 7));

    let err = a.cup_product(&b).expect_err("2 + 2 exceeds max_dim 3");
    assert!(matches!(err.0, TopologyErrorEnum::InvalidGradeOperation(_)));
}

#[test]
fn test_operands_on_different_complexes_are_still_rejected() {
    // The one precondition the free function cannot state, since it takes a single complex. This
    // is the method's own check and survived the delegation.
    let one = Arc::new(tetrahedron());
    let two = Arc::new(tetrahedron());
    let n1 = one.num_cells(1);
    let a = topology_cochain(&one, 1, pseudo_cochain(n1, 11));
    let b = topology_cochain(&two, 1, pseudo_cochain(n1, 13));

    let err = a
        .cup_product(&b)
        .expect_err("two equal-but-distinct complexes must be rejected");
    assert!(matches!(err.0, TopologyErrorEnum::GenericError(_)));
}
