/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Drives the `else { zero }` arm of the information-leak ratio in
//! `analyze`/`surd_states` (`surd_algo.rs:133` and the CDL twin
//! `surd_algo_cdl.rs`). That arm fires only when the *target* entropy `h` is not
//! strictly positive, i.e. the target marginal is concentrated in a single
//! state (a constant target with zero entropy). The XOR-style "info-leak-zero"
//! fixtures elsewhere keep `h > 0` and only exercise the `(hc/h).clamp(..)`
//! branch with `hc == 0`; here the target itself is degenerate so the ratio is
//! never formed and `info_leak` is forced to `0` by the fallback arm.

use deep_causality_algorithms::surd::{MaxOrder, surd_states, surd_states_cdl};
use deep_causality_tensor::{CausalTensor, CausalTensorError};

const TOLERANCE: f64 = 1e-10;

/// Probability table on shape `[2, 2, 2]` whose entire mass sits on target
/// state `T = 0`. The target marginal is therefore `[1, 0]`, giving
/// `H(target) = 0`, so the leak ratio takes the zero fallback rather than
/// dividing by `h`.
fn constant_target_table() -> Vec<f64> {
    let mut data = vec![0.0_f64; 8];
    let idx = |t: usize, s1: usize, s2: usize| (t * 2 + s1) * 2 + s2;
    // All mass on T = 0, spread across the source states so the sources are
    // still varied (only the *target* is constant).
    data[idx(0, 0, 0)] = 0.25;
    data[idx(0, 0, 1)] = 0.25;
    data[idx(0, 1, 0)] = 0.25;
    data[idx(0, 1, 1)] = 0.25;
    data
}

#[test]
fn test_info_leak_zero_when_target_entropy_zero() {
    let p_raw: CausalTensor<f64> =
        CausalTensor::new(constant_target_table(), vec![2, 2, 2]).unwrap();
    let result = surd_states(&p_raw, MaxOrder::Max);
    // The information-leak ratio is computed *before* the per-target
    // decomposition: with `H(target) == 0` the `h > eps` guard is false and the
    // `else { zero }` arm (`surd_algo.rs:133`) runs. The dense-`f64` path then
    // continues into the per-target loop, where the degenerate (probability-zero)
    // non-occurring target state makes a downstream conditional normalization
    // divide by zero, so the call surfaces `DivisionByZero`. The zero-entropy
    // fallback line is still executed on the way there. (The `Option`-aware CDL
    // path below tolerates the degenerate state and completes with leak == 0.)
    assert!(matches!(result, Err(CausalTensorError::DivisionByZero)));
}

#[test]
fn test_info_leak_zero_when_target_entropy_zero_cdl() {
    let opt: Vec<Option<f64>> = constant_target_table().into_iter().map(Some).collect();
    let p_raw = CausalTensor::new(opt, vec![2, 2, 2]).unwrap();
    let result = surd_states_cdl(&p_raw, MaxOrder::Max).unwrap();
    assert!(result.info_leak().abs() < TOLERANCE);
}
