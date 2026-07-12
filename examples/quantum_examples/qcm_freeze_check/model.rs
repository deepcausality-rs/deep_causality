/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The quantum causal model: a two-node causal graph and its Choi–Jamiołkowski
//! factor store. Configuration only — no execution — so the freeze scenarios in
//! `main.rs` read cleanly.

use crate::constants::{C, FloatType};
use deep_causality::utils_test::test_utils;
use deep_causality::{BaseCausaloid, CausableGraph, CausaloidGraph};
use deep_causality_num_complex::Complex;
use deep_causality_quantum::{FactorSupports, ProcessFactors};
use deep_causality_tensor::CausalTensor;

fn c(re: FloatType, im: FloatType) -> C {
    Complex::new(re, im)
}

fn mat(data: Vec<C>) -> CausalTensor<C> {
    CausalTensor::new(data, vec![2, 2]).unwrap()
}

/// Pauli-X.
pub fn sigma_x() -> CausalTensor<C> {
    mat(vec![c(0.0, 0.0), c(1.0, 0.0), c(1.0, 0.0), c(0.0, 0.0)])
}

/// Pauli-Z.
pub fn sigma_z() -> CausalTensor<C> {
    mat(vec![c(1.0, 0.0), c(0.0, 0.0), c(0.0, 0.0), c(-1.0, 0.0)])
}

/// A diagonal operator `diag(a, b)` — commutes with any other diagonal factor.
pub fn diagonal(a: FloatType, b: FloatType) -> CausalTensor<C> {
    mat(vec![c(a, 0.0), c(0.0, 0.0), c(0.0, 0.0), c(b, 0.0)])
}

/// A fresh two-node causal graph `0 → 1` whose nodes each carry one qubit.
pub fn two_node_graph() -> CausaloidGraph<BaseCausaloid<FloatType, bool>> {
    let mut g = CausaloidGraph::new(0);
    let n0 = g
        .add_causaloid(test_utils::get_test_causaloid_deterministic(0))
        .expect("add node 0");
    let n1 = g
        .add_causaloid(test_utils::get_test_causaloid_deterministic(1))
        .expect("add node 1");
    g.add_edge(n0, n1).expect("edge 0 → 1");
    g
}

/// The node-keyed factor store and the single-qubit support registry for the
/// two factors, both living on the shared Hilbert leg `0`.
pub fn factors_on_shared_leg(
    factor0: CausalTensor<C>,
    factor1: CausalTensor<C>,
) -> (ProcessFactors<FloatType>, FactorSupports) {
    let mut factors = ProcessFactors::new();
    factors.insert(0, factor0);
    factors.insert(1, factor1);

    let mut supports = FactorSupports::new();
    supports.declare(0, &[0]);
    supports.declare(1, &[0]);

    (factors, supports)
}
