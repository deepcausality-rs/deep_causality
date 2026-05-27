/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Predict-Correct Pipeline Inside a Causal Monad
//!
//! A 2-vector state is threaded through a `predict -> correct -> verify` chain.
//! Each step is a `Monad::bind` on `CausalEffectPropagationProcess`. The
//! predict step uses a tensor matrix-multiply; the correct step uses a
//! Clifford rotor; the verify step short-circuits on NaN.
//!
//! Tensor and algebra co-exist inside the same monadic chain. The error path
//! and the step log are handled by the monad, not by manual plumbing.
//!
//! ## APIs Demonstrated
//! - `CausalEffectPropagationProcessWitness::pure` and fluent `.bind()`
//! - `CausalTensor::ein_sum` with `EinSumOp::mat_mul`
//! - `CausalMultiVector::geometric_product`

use deep_causality_haft::Pure;
use deep_causality_metric::Metric;
use deep_causality_multivector::CausalMultiVector;
use deep_causality_num::RealField;
use deep_causality_tensor::{CausalTensor, EinSumOp, Tensor};
use mathematics_examples::effect_helpers::{
    Process, ProcessWitness, expect_value, fail, ok, print_log,
};

/// `f64` is the right precision here: only two rotation steps, not a long
/// chain. Float106 yields no observable gain.
pub type FloatType = f64;

fn main() {
    println!("=== Predict / Correct / Verify Inside the Causal Monad ===");
    println!("Precision: {}\n", core::any::type_name::<FloatType>());

    let initial =
        CausalTensor::new(vec![FloatType::from(1.0), FloatType::from(0.0)], vec![2]).unwrap();
    println!("Initial state x = {:?}\n", initial.as_slice());

    // predict -> correct -> verify, threaded through one monadic chain. The
    // value stays a `CausalTensor`, but predict uses a tensor matrix-multiply
    // and correct uses a Clifford rotor; both co-exist in the same chain. A
    // NaN in verify short-circuits the rest without manual error plumbing.
    let result: Process<CausalTensor<FloatType>> = ProcessWitness::pure(initial)
        .bind(|v, _, _| predict(v.into_value().expect("initial state")))
        .bind(|v, _, _| correct(v.into_value().expect("predicted state")))
        .bind(|v, _, _| verify(v.into_value().expect("corrected state")));

    println!("Chained log:");
    print_log(&result.logs);

    match result.error {
        Some(e) => println!("\nPipeline errored: {}", e),
        None => {
            let final_state = expect_value(&result.value);
            println!("\nFinal state: {:?}", final_state.as_slice());
        }
    }
}

fn deg_to_rad(deg: FloatType) -> FloatType {
    deg * FloatType::pi() / FloatType::from(180.0)
}

fn rotation_matrix_2d(theta: FloatType) -> CausalTensor<FloatType> {
    let c = theta.cos();
    let s = theta.sin();
    CausalTensor::new(vec![c, -s, s, c], vec![2, 2]).unwrap()
}

fn predict(state: CausalTensor<FloatType>) -> Process<CausalTensor<FloatType>> {
    // Predict: x' = F x. F is a 10-degree rotation as a stand-in for a model step.
    let f = rotation_matrix_2d(deg_to_rad(FloatType::from(10.0)));
    // mat_mul expects shape [m,n] x [n,k]. Reshape state from [2] to [2,1].
    let x_col = CausalTensor::new(state.as_slice().to_vec(), vec![2, 1]).unwrap();
    let ast = EinSumOp::mat_mul(f, x_col);
    let predicted = match CausalTensor::ein_sum(&ast) {
        Ok(t) => t,
        Err(e) => return fail(format!("predict mat_mul failed: {:?}", e)),
    };
    let flat = CausalTensor::new(predicted.as_slice().to_vec(), vec![2]).unwrap();
    let msg = format!("predict: rotated by 10 deg -> {:?}", flat.as_slice());
    ok(flat, msg)
}

fn correct(state: CausalTensor<FloatType>) -> Process<CausalTensor<FloatType>> {
    // Correct: rotate by an additional -3 degrees via Clifford rotor.
    let metric = Metric::Euclidean(2);
    let theta = deg_to_rad(FloatType::from(-3.0));
    let half = theta / FloatType::from(2.0);
    let c = half.cos();
    let sn = half.sin();
    let zero = FloatType::from(0.0);
    let rotor = CausalMultiVector::new(vec![c, zero, zero, -sn], metric).unwrap();
    let rotor_rev = CausalMultiVector::new(vec![c, zero, zero, sn], metric).unwrap();

    let s = state.as_slice();
    let v = CausalMultiVector::new(vec![zero, s[0], s[1], zero], metric).unwrap();
    let rotated = rotor.geometric_product(&v).geometric_product(&rotor_rev);
    let d = rotated.data();

    let new_state = CausalTensor::new(vec![d[1], d[2]], vec![2]).unwrap();
    let msg = format!("correct: -3 deg rotor -> {:?}", new_state.as_slice());
    ok(new_state, msg)
}

fn verify(state: CausalTensor<FloatType>) -> Process<CausalTensor<FloatType>> {
    if state.as_slice().iter().any(|v| v.is_nan()) {
        return fail("verify: NaN detected");
    }
    ok(state, "verify: state is finite")
}
