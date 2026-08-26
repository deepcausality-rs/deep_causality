/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! This crate computes the cup product twice, and these tests pin that the two agree.
//!
//! The free [`cup_product`] is generic over `K: ChainComplex` with `K::CellType: SplittableCell`,
//! and takes its cochains apart into a slice and a degree each. `Topology<T>::cup_product` is a
//! second, independent implementation: simplicial-only, extracting the Alexander–Whitney front and
//! back faces by hand, and taking its cochains as whole `Topology<T>` values that already carry
//! their grade.
//!
//! `Simplex` implements `SplittableCell`, so the generic function already covers everything the
//! method does. Neither calls the other. Until these tests, nothing said they had to agree, and a
//! fix applied to one would have left the other behind.

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::utils_tests::{pseudo_cochain, tetrahedron};
use deep_causality_topology::{ChainComplex, Topology, cup_product};
use std::sync::Arc;

/// Every degree pair a tetrahedron admits, both ways round.
///
/// Measured before this test was written: the two agree exactly, to the bit, at every pair below.
/// So the assertion is equality rather than a tolerance.
#[test]
fn test_the_two_cup_product_implementations_agree() {
    let complex = Arc::new(tetrahedron());

    for (p, q) in [(0usize, 0usize), (0, 1), (1, 0), (1, 1), (0, 2), (2, 0)] {
        let np = complex.num_cells(p);
        let nq = complex.num_cells(q);
        assert!(np > 0 && nq > 0, "degree {p} or {q} has no cells");

        let a = pseudo_cochain(np, 11 + p as u64);
        let b = pseudo_cochain(nq, 23 + q as u64);

        let free = cup_product(complex.as_ref(), &a, p, &b, q)
            .unwrap_or_else(|e| panic!("free cup_product at ({p}, {q}): {e}"));

        let ta = Topology::new(
            complex.clone(),
            p,
            CausalTensor::new(a, vec![np]).unwrap(),
            0,
        )
        .unwrap();
        let tb = Topology::new(
            complex.clone(),
            q,
            CausalTensor::new(b, vec![nq]).unwrap(),
            0,
        )
        .unwrap();
        let method = ta
            .cup_product(&tb)
            .unwrap_or_else(|e| panic!("Topology::cup_product at ({p}, {q}): {e}"));

        assert_eq!(
            method.grade(),
            p + q,
            "the method's result must have grade p + q"
        );
        assert_eq!(
            free.len(),
            method.data().as_slice().len(),
            "the two disagree on the size of the result at ({p}, {q})"
        );
        assert_eq!(
            free,
            method.data().as_slice().to_vec(),
            "the two implementations disagree at ({p}, {q})"
        );
    }
}
