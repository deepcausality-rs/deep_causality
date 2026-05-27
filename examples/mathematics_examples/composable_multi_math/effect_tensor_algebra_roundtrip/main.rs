/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Tensor <-> Algebra Round-Trip Inside the Causal Monad
//!
//! A vector starts as a `CausalTensor<FloatType>` of shape `[3]`. It is lifted
//! into `Cl(3,0)` as a pure vector, rotated by a Clifford rotor, then lowered
//! back to a tensor. A final tensor dot product confirms the norm was preserved.
//!
//! Each step is a `bind` on `CausalEffectPropagationProcess`. The carried
//! value type changes between steps: `CausalTensor` -> `CausalMultiVector` ->
//! `CausalTensor` -> `FloatType`. The monad threads them in one straight line.
//!
//! ## APIs Demonstrated
//! - Fluent `.bind(|value, state, context| ...)` across heterogeneous value types
//! - `CausalMultiVector::geometric_product`
//! - `EinSumOp::dot_prod` for the norm check

use deep_causality_haft::Pure;
use deep_causality_metric::Metric;
use deep_causality_multivector::CausalMultiVector;
use deep_causality_num::{Float106, RealField};
use deep_causality_tensor::{CausalTensor, EinSumOp, Tensor};
use mathematics_examples::effect_helpers::{
    Process, ProcessWitness, expect_value, fail, ok, print_log,
};

/// Switch this alias to `f32` for low precision, `f64` for standard precision,
/// or `Float106` for high precision.
pub type FloatType = Float106;

fn main() {
    println!("=== Tensor <-> Algebra Round-Trip Inside the Causal Monad ===");
    println!("Precision: {}\n", core::any::type_name::<FloatType>());

    let initial = CausalTensor::new(
        vec![
            FloatType::from(3.0),
            FloatType::from(4.0),
            FloatType::from(0.0),
        ],
        vec![3],
    )
    .unwrap();
    let initial_norm_sq: FloatType = initial
        .as_slice()
        .iter()
        .fold(FloatType::from(0.0), |acc, &v| acc + v * v);
    println!("Initial vector: {:?}", initial.as_slice());
    println!("Initial |v|^2 = {}\n", initial_norm_sq);

    // One straight-line monadic chain. The carried value type changes at every
    // step (Tensor -> MultiVector -> Tensor -> scalar); `bind` threads the
    // value, state, context, error, and log automatically. The closure receives
    // the upstream `EffectValue`; an error in any step short-circuits the rest.
    let result: Process<FloatType> = ProcessWitness::pure(initial)
        .bind(|v, _, _| lift_to_algebra(v.into_value().expect("initial tensor")))
        .bind(|v, _, _| rotate_in_xy(v.into_value().expect("lifted multivector")))
        .bind(|v, _, _| lower_to_tensor(v.into_value().expect("rotated multivector")))
        .bind(|v, _, _| norm_squared(v.into_value().expect("lowered tensor")));

    println!("Chain log:");
    print_log(&result.logs);

    match result.error {
        Some(e) => println!("\nChain errored: {}", e),
        None => {
            let final_norm_sq = expect_value(&result.value);
            let drift = (final_norm_sq - initial_norm_sq).abs();
            println!("\nFinal |v|^2 = {}", final_norm_sq);
            println!(
                "Round-trip drift = {} (should be at machine epsilon)",
                drift
            );
        }
    }
}

fn lift_to_algebra(v: CausalTensor<FloatType>) -> Process<CausalMultiVector<FloatType>> {
    let s = v.as_slice();
    if s.len() != 3 {
        return fail(format!("lift: expected length 3 vector, got {}", s.len()));
    }
    let zero = FloatType::from(0.0);
    // Cl(3,0) basis order: [1, e1, e2, e3, e12, e13, e23, e123]
    let coeffs = vec![zero, s[0], s[1], s[2], zero, zero, zero, zero];
    let mv = CausalMultiVector::new(coeffs, Metric::Euclidean(3)).unwrap();
    let msg = format!("lift: tensor {:?} -> multivector vector", s);
    ok(mv, msg)
}

fn rotate_in_xy(v: CausalMultiVector<FloatType>) -> Process<CausalMultiVector<FloatType>> {
    // 90-degree rotation in the e1^e2 plane.
    let metric = Metric::Euclidean(3);
    let theta = FloatType::pi() / FloatType::from(2.0);
    let half = theta / FloatType::from(2.0);
    let c = half.cos();
    let sn = half.sin();
    let zero = FloatType::from(0.0);
    let rotor =
        CausalMultiVector::new(vec![c, zero, zero, zero, -sn, zero, zero, zero], metric).unwrap();
    let rotor_rev =
        CausalMultiVector::new(vec![c, zero, zero, zero, sn, zero, zero, zero], metric).unwrap();
    let rotated = rotor.geometric_product(&v).geometric_product(&rotor_rev);
    ok(rotated, "rotate: 90 deg in e1^e2 plane")
}

fn lower_to_tensor(mv: CausalMultiVector<FloatType>) -> Process<CausalTensor<FloatType>> {
    let d = mv.data();
    let v = CausalTensor::new(vec![d[1], d[2], d[3]], vec![3]).unwrap();
    let msg = format!("lower: multivector -> tensor {:?}", v.as_slice());
    ok(v, msg)
}

fn norm_squared(v: CausalTensor<FloatType>) -> Process<FloatType> {
    let copy = v.clone();
    let ast = EinSumOp::dot_prod(v, copy);
    let result = match CausalTensor::ein_sum(&ast) {
        Ok(t) => t.as_slice()[0],
        Err(e) => return fail(format!("dot_prod failed: {:?}", e)),
    };
    ok(result, format!("norm^2 via tensor dot_prod = {}", result))
}
