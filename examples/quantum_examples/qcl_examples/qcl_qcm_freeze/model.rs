/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The model: a two-node causal graph `0 → 1` and its Choi–Jamiołkowski factor store, both
//! factors on the shared Hilbert leg `0`. Configuration only; the stages run in `main.rs`.

use crate::{C, FloatType};
use deep_causality::utils_test::test_utils;
use deep_causality::{BaseCausaloid, CausableGraph, CausaloidGraph};
use deep_causality_num::lift;
use deep_causality_num_complex::Complex;
use deep_causality_quantum::{FactorSupports, ProcessFactors};
use deep_causality_tensor::CausalTensor;

fn c(re: f64, im: f64) -> C {
    Complex::new(lift(re), lift(im))
}

fn mat(data: Vec<C>) -> CausalTensor<C> {
    CausalTensor::new(data, vec![2, 2]).expect("a 2 × 2 matrix")
}

/// Pauli-X.
pub fn sigma_x() -> CausalTensor<C> {
    mat(vec![c(0.0, 0.0), c(1.0, 0.0), c(1.0, 0.0), c(0.0, 0.0)])
}

/// Pauli-Z.
pub fn sigma_z() -> CausalTensor<C> {
    mat(vec![c(1.0, 0.0), c(0.0, 0.0), c(0.0, 0.0), c(-1.0, 0.0)])
}

/// `diag(a, b)`, which commutes with any other diagonal factor.
pub fn diagonal(a: f64, b: f64) -> CausalTensor<C> {
    mat(vec![c(a, 0.0), c(0.0, 0.0), c(0.0, 0.0), c(b, 0.0)])
}

/// A two-node graph `0 → 1`, frozen or dynamic.
///
/// The causaloids come from the engine's test utilities, which fix their value type at `f64`;
/// that scalar belongs to the graph's classical causaloids, not to the quantum factors, and is
/// unrelated to `FloatType`.
pub fn two_node_graph(frozen: bool) -> CausaloidGraph<BaseCausaloid<f64, bool>> {
    let mut g = CausaloidGraph::new(0);
    let n0 = g
        .add_causaloid(test_utils::get_test_causaloid_deterministic(0))
        .expect("add node 0");
    let n1 = g
        .add_causaloid(test_utils::get_test_causaloid_deterministic(1))
        .expect("add node 1");
    g.add_edge(n0, n1).expect("edge 0 → 1");
    if frozen {
        g.freeze();
    }
    g
}

/// The factor store and support registry for two factors on the shared leg `0`.
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
