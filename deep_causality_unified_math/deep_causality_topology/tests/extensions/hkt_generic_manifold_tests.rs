/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `GenericManifoldWitness<K>` over arbitrary `ChainComplex`.
//!
//! After the precision-as-parameter refactor, `GenericManifoldWitness<K>` exposes its
//! functional surface as inherent methods (see `extensions/hkt_manifold/mod.rs`). The
//! simplicial path is covered by `hkt_manifold_tests.rs`; this file covers the cubical
//! path.

use deep_causality_haft::Functor;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{GenericManifoldWitness, LatticeComplex, Manifold};

#[test]
fn generic_manifold_witness_fmap_doubles_field_on_lattice_complex() {
    let complex = LatticeComplex::<2, f64>::open([3, 3]);
    let data = CausalTensor::from_vec(vec![1.0f64, 2.0, 3.0, 4.0], &[4]);
    let manifold: Manifold<LatticeComplex<2, f64>, f64> = Manifold::from_cubical(complex, data, 0);

    let doubled = <GenericManifoldWitness<LatticeComplex<2, f64>> as Functor<
        GenericManifoldWitness<LatticeComplex<2, f64>>,
    >>::fmap(manifold, |x: f64| x * 2.0);

    let v: Vec<f64> = doubled.data().as_slice().to_vec();
    assert_eq!(v, vec![2.0, 4.0, 6.0, 8.0]);
}
