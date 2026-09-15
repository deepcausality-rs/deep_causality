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

use deep_causality_algebra::Real;
use deep_causality_haft::Pure;
use deep_causality_metric::Metric;
use deep_causality_multivector::CausalMultiVector;
use deep_causality_num::lift;
use deep_causality_tensor::{CausalTensor, EinSumOp, Tensor};
use mathematics_examples::effect_helpers::{Process, ProcessWitness, StepLog, fail, ok, print_log};

/// `f64` is the right precision here: only two rotation steps, not a long
/// chain. Float106 yields no observable gain.
pub type FloatType = f64;

fn main() {
    print_header();

    let initial = CausalTensor::new(
        vec![lift::<FloatType>(1.0), lift::<FloatType>(0.0)],
        vec![2],
    )
    .expect("two components in a rank-1 shape of 2");
    print_initial(initial.as_slice());

    // predict -> correct -> verify, threaded through one monadic chain. The
    // value stays a `CausalTensor`, but predict uses a tensor matrix-multiply
    // and correct uses a Clifford rotor; both co-exist in the same chain. A
    // NaN in verify short-circuits the rest without manual error plumbing.
    let result: Process<CausalTensor<FloatType>> = ProcessWitness::pure(initial)
        .bind(|v, _, _| predict(v.into_value().expect("initial state")))
        .bind(|v, _, _| correct(v.into_value().expect("predicted state")))
        .bind(|v, _, _| verify(v.into_value().expect("corrected state")));

    print_chain_log(result.logs());
    print_outcome(
        result.error().map(|e| e.to_string()),
        result.value_cloned().as_ref().map(|t| t.as_slice()),
    );
}

fn deg_to_rad(deg: FloatType) -> FloatType {
    deg * FloatType::pi() / lift::<FloatType>(180.0)
}

fn rotation_matrix_2d(theta: FloatType) -> CausalTensor<FloatType> {
    let c = theta.cos();
    let s = theta.sin();
    CausalTensor::new(vec![c, -s, s, c], vec![2, 2]).expect("four components in a 2x2 shape")
}

/// Predict: `x' = F x`, with `F` a 10-degree rotation standing in for a model step.
///
/// The body propagates with `?`; this wrapper turns a returned error into the chain's own
/// error channel, which is what `bind` short-circuits on.
fn predict(state: CausalTensor<FloatType>) -> Process<CausalTensor<FloatType>> {
    match predict_step(state) {
        Ok((flat, msg)) => ok(flat, msg),
        Err(e) => fail(e),
    }
}

fn predict_step(
    state: CausalTensor<FloatType>,
) -> Result<(CausalTensor<FloatType>, String), String> {
    let f = rotation_matrix_2d(deg_to_rad(lift::<FloatType>(10.0)));
    // mat_mul expects [m,n] x [n,k], so the state reshapes from [2] to [2,1].
    let x_col = CausalTensor::new(state.as_slice().to_vec(), vec![2, 1])
        .map_err(|e| format!("predict: reshaping the state failed: {e:?}"))?;
    let predicted = CausalTensor::ein_sum(&EinSumOp::mat_mul(f, x_col))
        .map_err(|e| format!("predict mat_mul failed: {e:?}"))?;
    let flat = CausalTensor::new(predicted.as_slice().to_vec(), vec![2])
        .map_err(|e| format!("predict: flattening the result failed: {e:?}"))?;
    let msg = format!("predict: rotated by 10 deg -> {:?}", flat.as_slice());
    Ok((flat, msg))
}

/// Correct: a further -3 degrees, applied as a Clifford rotor rather than a matrix.
fn correct(state: CausalTensor<FloatType>) -> Process<CausalTensor<FloatType>> {
    match correct_step(state) {
        Ok((new_state, msg)) => ok(new_state, msg),
        Err(e) => fail(e),
    }
}

fn correct_step(
    state: CausalTensor<FloatType>,
) -> Result<(CausalTensor<FloatType>, String), String> {
    let metric = Metric::Euclidean(2);
    let theta = deg_to_rad(lift::<FloatType>(-3.0));
    let half = theta / lift::<FloatType>(2.0);
    let (c, sn) = (half.cos(), half.sin());
    let zero = lift::<FloatType>(0.0);

    let blade = |coeffs: Vec<FloatType>| {
        CausalMultiVector::new(coeffs, metric)
            .map_err(|e| format!("correct: building a Cl(2,0) element failed: {e:?}"))
    };
    let rotor = blade(vec![c, zero, zero, -sn])?;
    let rotor_rev = blade(vec![c, zero, zero, sn])?;

    let s = state.as_slice();
    let v = blade(vec![zero, s[0], s[1], zero])?;
    let rotated = rotor.geometric_product(&v).geometric_product(&rotor_rev);
    let d = rotated.data();

    let new_state = CausalTensor::new(vec![d[1], d[2]], vec![2])
        .map_err(|e| format!("correct: rebuilding the state failed: {e:?}"))?;
    let msg = format!("correct: -3 deg rotor -> {:?}", new_state.as_slice());
    Ok((new_state, msg))
}

fn verify(state: CausalTensor<FloatType>) -> Process<CausalTensor<FloatType>> {
    if state.as_slice().iter().any(|v| v.is_nan()) {
        return fail("verify: NaN detected");
    }
    ok(state, "verify: state is finite")
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("=== Predict / Correct / Verify Inside the Causal Monad ===");
    println!("Precision: {}\n", core::any::type_name::<FloatType>());
}

fn print_initial(state: &[FloatType]) {
    println!("Initial state x = {state:?}\n");
}

fn print_chain_log(logs: &StepLog) {
    println!("Chained log:");
    print_log(logs);
}

fn print_outcome(error: Option<String>, final_state: Option<&[FloatType]>) {
    match error {
        Some(e) => println!("\nPipeline errored: {e}"),
        None => println!(
            "\nFinal state: {:?}",
            final_state.expect("a chain with no error carries a value")
        ),
    }
}
